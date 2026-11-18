//! Laser Bridges & Doors — the one `impl Hooks` at the crate root.
//!
//! Upstream (Laser Bridges & Doors 6.3, MIT) recomputes a source in `neighborChanged`/`onPlace`
//! with the level in hand: `getBestNeighborSignal` → the `powered` state → `generateBridge`. Two of
//! those moments read-only here — a block part's `neighbor_changed` and the global `block_changed`
//! answer only a read-only level — so the port queues the source and applies it in
//! [`Hooks::level_tick_post`], where the level of the tick is writable. A source therefore reacts one
//! server tick late and then settles (`set_block` refuses an unchanged state, so the second pass
//! writes nothing). `block_changed` carries the old state, so a removed or replaced source clears
//! the beam it left in the same tick, without per-position memory a world reload would lose.
//!
//! The beam itself — its length, its line, what stops it, and the state a placement takes — lives in
//! [`runtime`], byte-identical to the retired bundle's `runtime.rs` and tested by `cargo test`
//! without the game. This module is only the glue between it and the SDK.
use std::collections::BTreeMap;
use std::sync::{Mutex, MutexGuard};

use mod_api::{
    self, BlockChangedEvent, BlockFace, BlockMetadata, BlockPart, BlockState, BlockUpdateFlags,
    Direction, Flow, Hooks, Host, HostBlockStates, HostSide, InteractionResult, LevelHost,
    LevelRef, LevelTickPostEvent, LevelTicksHost, NeighborChangedEvent, Registrar,
    RegistrationContext, ServerHost, StateConstraint, StateForPlacementEvent, SupportType,
    UseItemOnBlockEvent, UsePhase,
};

pub mod runtime;

use runtime::{Beam, Cell, Clicked, Face, Facing, MAX_LENGTH, Write, placement, plan};

/// One source block: the laser block it writes, and whether it hangs on a support (upstream's
/// `AbstractAttachedSourceBlock`; the full-block sources have no `canSurvive`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Source {
    /// The source's id, which is also its block item's id.
    id: &'static str,
    /// The laser block its beam writes.
    beam: &'static str,
    /// Whether it needs a sturdy face behind it when it is placed.
    attached: bool,
}

/// `ModRegistry.RegisterSetup`: each source and the laser block it generates.
const SOURCES: [Source; 4] = [
    Source {
        id: "laserbridges:laser_source_block",
        beam: "laserbridges:laser_block_powered",
        attached: true,
    },
    Source {
        id: "laserbridges:laser_source_fullblock",
        beam: "laserbridges:laser_block_powered",
        attached: false,
    },
    Source {
        id: "laserbridges:laser_fence_source_block",
        beam: "laserbridges:laser_fence_powered",
        attached: true,
    },
    Source {
        id: "laserbridges:laser_fence_source_fullblock",
        beam: "laserbridges:laser_fence_powered",
        attached: false,
    },
];

/// `LaserBridgesConfig.blocks_cut_through_by_lasers`, minus the three airs, which
/// [`classify`] handles as [`Cell::Air`].
const CUTTABLE: [&str; 5] = [
    "minecraft:short_grass",
    "minecraft:tall_grass",
    "minecraft:fern",
    "minecraft:large_fern",
    "minecraft:dead_bush",
];

/// The source with this block (or block item) id.
fn source_of(id: &str) -> Option<Source> {
    SOURCES.iter().copied().find(|source| source.id == id)
}

/// Whether this block item is one of the two attached sources, the only ones upstream refuses to
/// place without a sturdy face behind them.
fn attached_item(id: &str) -> bool {
    source_of(id).is_some_and(|source| source.attached)
}

/// What one block change means for the beams: nothing, a new source to evaluate, an old source's
/// beam to clear, or both when one kind of source replaced another.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Change {
    /// No source at either end: other blocks are none of the beam's business.
    Nothing,
    /// A source is there now (placed, or replaced by one of the same kind).
    Evaluate,
    /// A source went away; the beam its geometry left is still in the world.
    Clear(&'static str),
    /// One kind of source replaced another: clear the old beam, then build the new one.
    Replace(&'static str),
}

/// `onPlace` and `onRemove` over one change: a new source is evaluated, a removed one drops its
/// beam, and a source replaced by the other kind does both (upstream runs `onRemove` first).
fn change(old: Option<Source>, new: Option<Source>) -> Change {
    match (old, new) {
        (Some(before), Some(after)) if before.beam != after.beam => Change::Replace(before.beam),
        (Some(_), Some(_)) => Change::Evaluate,
        (Some(before), None) => Change::Clear(before.beam),
        (None, Some(_)) => Change::Evaluate,
        (None, None) => Change::Nothing,
    }
}

/// The runtime face for a clicked [`Direction`].
fn clicked(face: Direction) -> Clicked {
    match face {
        Direction::Up => Clicked::Up,
        Direction::Down => Clicked::Down,
        Direction::North => Clicked::North,
        Direction::South => Clicked::South,
        Direction::East => Clicked::East,
        Direction::West => Clicked::West,
    }
}

/// The [`Direction`] a clicked face's unit normal names.
fn direction([x, y, z]: [i32; 3]) -> Option<Direction> {
    Some(match (x, y, z) {
        (0, -1, 0) => Direction::Down,
        (0, 1, 0) => Direction::Up,
        (0, 0, -1) => Direction::North,
        (0, 0, 1) => Direction::South,
        (-1, 0, 0) => Direction::West,
        (1, 0, 0) => Direction::East,
        _ => return None,
    })
}

/// The SDK face of a [`Direction`].
fn block_face(direction: Direction) -> BlockFace {
    match direction {
        Direction::Down => BlockFace::Down,
        Direction::Up => BlockFace::Up,
        Direction::North => BlockFace::North,
        Direction::South => BlockFace::South,
        Direction::West => BlockFace::West,
        Direction::East => BlockFace::East,
    }
}

/// A parsed source state: which source, the beam its geometry describes, and whether it is powered.
struct Parsed {
    source: Source,
    beam: Beam,
    powered: bool,
}

/// The source a state spells, with its `face`/`facing`/`color`/`powered` read as the sidecar
/// constants the retired glue read from the registry.
fn parse(metadata: &dyn BlockMetadata, state: BlockState) -> Option<Parsed> {
    let source = source_of(metadata.name(state.block()).ok()?)?;
    let face = Face::from_name(state.property(metadata, "face").ok()??)?;
    let facing = Facing::from_name(state.property(metadata, "facing").ok()??)?;
    // `ISourceBlock.COLOR` is a dye id, 0 to 15; a state that spells anything else is not ours.
    let colour = state
        .property(metadata, "color")
        .ok()??
        .parse::<u8>()
        .ok()?;
    let powered = state.property(metadata, "powered").ok().flatten() == Some("true");
    Some(Parsed {
        source,
        beam: Beam {
            face,
            facing,
            colour,
        },
        powered,
    })
}

/// A source position's pending work for the next level tick.
#[derive(Clone, Copy, Default)]
struct Job {
    /// Read the redstone input and regenerate.
    evaluate: bool,
    /// Clear the beam a removed (or replaced) source left: the old source state (its geometry) and
    /// the laser block it wrote.
    clear: Option<(BlockState, &'static str)>,
}

/// The queue: one job per (dimension, position), merged as hooks report it.
#[derive(Default)]
struct Jobs {
    by_position: BTreeMap<(String, [i32; 3]), Job>,
}

/// The queue lives for the session. `Hooks` is `Send + Sync`, so it is a `Mutex`, and a block part
/// has no route to the mod value the macro boxes: the queue is module state, as the retired
/// bundle's was.
static JOBS: Mutex<Option<Jobs>> = Mutex::new(None);

fn jobs() -> MutexGuard<'static, Option<Jobs>> {
    JOBS.lock().expect("laserbridges job queue lock poisoned")
}

/// Queue `job` for `pos` in `dimension`; two hooks reporting the same position merge.
fn queue(dimension: &str, pos: [i32; 3], job: Job) {
    let mut guard = jobs();
    let jobs = guard.get_or_insert_with(Jobs::default);
    let entry = jobs
        .by_position
        .entry((dimension.to_owned(), pos))
        .or_default();
    entry.evaluate |= job.evaluate;
    if job.clear.is_some() {
        entry.clear = job.clear;
    }
}

/// Take every job queued for `dimension`, leaving the other dimensions' alone.
fn take(dimension: &str) -> Vec<([i32; 3], Job)> {
    let mut guard = jobs();
    let Some(jobs) = guard.as_mut() else {
        return Vec::new();
    };
    let keyed: Vec<(String, [i32; 3])> = jobs
        .by_position
        .keys()
        .filter(|(dim, _)| dim == dimension)
        .cloned()
        .collect();
    let mut ready = Vec::with_capacity(keyed.len());
    for key in keyed {
        if let Some(job) = jobs.by_position.remove(&key) {
            ready.push((key.1, job));
        }
    }
    ready
}

/// `ISourceBlock.generateBridge`: walk the line from the source, placing the laser block while
/// inside `length` and clearing the ones already there beyond it.
fn generate(
    host: &mut dyn ServerHost,
    level: LevelRef,
    origin: [i32; 3],
    beam: Beam,
    beam_id: &str,
    power: u8,
) {
    let Some(air) = host.block_state_id("minecraft:air") else {
        return;
    };
    let target;
    let block;
    let writes = {
        let facts: &dyn Host = &*host;
        let Some(state) = beam_state(facts, beam_id, beam) else {
            return;
        };
        target = state;
        block = target.block();
        plan(beam, power, MAX_LENGTH, |at| {
            classify(level, facts, origin, beam, at, target)
        })
    };
    for write in writes {
        let at = match write {
            Write::Cut { at } | Write::Place { at } | Write::Clear { at } => at,
        };
        let pos = offset(origin, beam.offset(at));
        match write {
            Write::Cut { .. } => {
                // `Level.destroyBlock(pos, true)`: the cut plant drops.
                let _ = level.destroy_block(host, pos, true, None, false);
            }
            Write::Place { .. } => {
                if level.block_state(&*host, pos) != Some(target) {
                    let _ = level.set_block(host, pos, target, update_flags());
                }
            }
            Write::Clear { .. } => {
                if level
                    .block_state(&*host, pos)
                    .is_some_and(|current| current.block() == block)
                {
                    let _ = level.set_block(host, pos, air, update_flags());
                }
            }
        }
    }
}

/// This beam's laser-block state: its block, and the beam's `face`, `facing` and `color` over the
/// block's own defaults.
fn beam_state(facts: &dyn Host, beam_id: &str, beam: Beam) -> Option<BlockState> {
    let block = facts.block_key(beam_id)?;
    let colour = beam.colour.to_string();
    let constraints = [
        StateConstraint::new("face", beam.face.name()),
        StateConstraint::new("facing", beam.facing.name()),
        StateConstraint::new("color", colour),
    ];
    BlockState::of(&facts.block_states(), block, &constraints).ok()
}

/// What `generateBridge` finds in one cell of the line.
fn classify(
    level: LevelRef,
    facts: &dyn Host,
    origin: [i32; 3],
    beam: Beam,
    at: i32,
    target: BlockState,
) -> Cell {
    let pos = offset(origin, beam.offset(at));
    let Some(current) = level.block_state(facts, pos) else {
        // A cell in an unloaded chunk stops the line, as a solid block does.
        return Cell::Solid;
    };
    if current == target {
        return Cell::Same;
    }
    let states = facts.block_states();
    let Ok(name) = states.name(current.block()) else {
        return Cell::Solid;
    };
    match name {
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" => Cell::Air,
        cuttable if CUTTABLE.contains(&cuttable) => Cell::Cuttable,
        _ => Cell::Solid,
    }
}

/// `Level.setBlock`'s `UPDATE_ALL`: neighbours and clients are told.
fn update_flags() -> BlockUpdateFlags {
    BlockUpdateFlags::NEIGHBORS | BlockUpdateFlags::CLIENTS
}

fn offset([x, y, z]: [i32; 3], [dx, dy, dz]: [i32; 3]) -> [i32; 3] {
    [x + dx, y + dy, z + dz]
}

fn xyz(pos: mod_api::BlockPos) -> [i32; 3] {
    [pos.x, pos.y, pos.z]
}

/// A source block's own part: where a placement points it, and its redstone input moving.
struct SourcePart;

#[mod_api::horizon_part]
impl BlockPart for SourcePart {
    /// `AbstractSourceBlock.getStateForPlacement`: a vertical click mounts the source on the floor
    /// or ceiling and a horizontal one on the wall, emitting away from it. The attached sources'
    /// support requirement is [`Hooks::use_item_on_block`]: this moment's level is read-only and has
    /// no sturdy-face query, while the use moment has the whole host.
    fn state_for_placement(
        &self,
        level: &dyn LevelHost,
        event: &mut StateForPlacementEvent<'_>,
    ) -> Option<Option<BlockState>> {
        let (face, facing) = placement(clicked(event.context.face), event.context.yaw);
        let face = level.state_with_property(event.state, "face", face.name())?;
        let state = level.state_with_property(face, "facing", facing.name())?;
        Some(Some(state))
    }

    /// A neighbour changed under this source: its redstone input may have moved, so queue it for
    /// the next level tick (this level's ticks schedule, but cannot replace blocks).
    fn neighbor_changed(&self, _level: &mut dyn LevelTicksHost, event: &mut NeighborChangedEvent) {
        queue(
            event.dimension.key().id(),
            xyz(event.pos),
            Job {
                evaluate: true,
                clear: None,
            },
        );
    }
}

#[derive(Default)]
struct Mod;

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Attach the source part to the four sources. The part is per block, so the game asks it only
    /// for these blocks and no method has to check which block it was asked about.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        for source in SOURCES {
            let key = registrar.block_key(source.id);
            registrar.block_part(key, SourcePart);
        }
    }

    /// An authoritative server committed a changed state: evaluate a new source, and clear the beam
    /// a removed or replaced one left. Applied at the next level tick, where the level writes.
    fn block_changed(&self, level: &dyn LevelHost, event: &mut BlockChangedEvent) {
        if event.old == event.new {
            return;
        }
        let old = level.state_block(event.old).and_then(source_of);
        let new = level.state_block(event.new).and_then(source_of);
        let (evaluate, clear) = match change(old, new) {
            Change::Nothing => return,
            Change::Evaluate => (true, None),
            Change::Clear(beam) => (false, Some(beam)),
            Change::Replace(beam) => (true, Some(beam)),
        };
        queue(
            event.dimension.key().id(),
            xyz(event.pos),
            Job {
                evaluate,
                clear: clear.map(|beam| (event.old, beam)),
            },
        );
    }

    /// After a level finished its tick pass, with that level in place: read each queued source's
    /// signal, set its `powered` state and generate (or clear) its beam.
    fn level_tick_post(&self, host: HostSide<'_>, event: &mut LevelTickPostEvent) {
        let HostSide::Server(host) = host else {
            return;
        };
        let level = event.level;
        let Some(dimension) = level.dimension().id(&*host) else {
            return;
        };
        for (pos, job) in take(&dimension) {
            if let Some((old, beam_id)) = job.clear {
                let beam = {
                    let facts: &dyn Host = &*host;
                    let states = facts.block_states();
                    parse(&states, old).map(|parsed| parsed.beam)
                };
                if let Some(beam) = beam {
                    generate(host, level, pos, beam, beam_id, 0);
                }
            }
            if !job.evaluate {
                continue;
            }
            let Some(current) = level.block_state(&*host, pos) else {
                continue;
            };
            let parsed = {
                let facts: &dyn Host = &*host;
                let states = facts.block_states();
                parse(&states, current)
            };
            let Some(parsed) = parsed else {
                continue;
            };
            let signal = level.best_neighbor_signal(&*host, pos).unwrap_or(0);
            let power = u8::try_from(signal).unwrap_or(0).min(15);
            if parsed.powered != (power > 0) {
                let powered = if power > 0 { "true" } else { "false" };
                let new = {
                    let facts: &dyn Host = &*host;
                    let states = facts.block_states();
                    current.with(&states, "powered", powered).ok()
                };
                if let Some(new) = new {
                    let _ = level.set_block(host, pos, new, update_flags());
                }
            }
            generate(host, level, pos, parsed.beam, parsed.source.beam, power);
        }
    }

    /// An attached source needs a sturdy face behind it (`FaceAttachedHorizontalDirectionalBlock`'s
    /// `canSurvive`). Upstream refuses the placement and keeps the stack; the pre-block phase of the
    /// click is where that can still be done, because the placement part's level has no sturdy-face
    /// query.
    fn use_item_on_block(&self, host: HostSide<'_>, event: &mut UseItemOnBlockEvent<'_>) -> Flow {
        if event.phase != UsePhase::ItemBeforeBlock {
            return Flow::Continue;
        }
        let HostSide::Server(host) = host else {
            return Flow::Continue;
        };
        let click = event.click;
        let Some(stack) = click.stack else {
            return Flow::Continue;
        };
        if !attached_item(&stack.item().id(&*host)) {
            return Flow::Continue;
        }
        let Some(face) = direction(click.hit.face) else {
            return Flow::Continue;
        };
        let Some(level) = host.level(click.dimension.key().id()) else {
            return Flow::Continue;
        };
        if level.block_face_sturdy(&*host, click.pos, block_face(face), SupportType::Full)
            != Some(false)
        {
            return Flow::Continue;
        }
        // `getStateForPlacement` answers null upstream: nothing is placed and the stack stays.
        *event.cancellation_result = InteractionResult::Fail;
        Flow::Cancel
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SOURCES`' four ids, for the tests below.
    const BRIDGE: &str = "laserbridges:laser_source_block";
    const BRIDGE_FULL: &str = "laserbridges:laser_source_fullblock";
    const FENCE: &str = "laserbridges:laser_fence_source_block";
    const FENCE_FULL: &str = "laserbridges:laser_fence_source_fullblock";

    #[test]
    fn each_source_names_the_laser_block_it_writes() {
        let bridge = source_of(BRIDGE).expect("the wall bridge is a source");
        assert_eq!(bridge.beam, "laserbridges:laser_block_powered");
        assert!(bridge.attached);
        let full = source_of(BRIDGE_FULL).expect("the full block bridge is a source");
        assert_eq!(full.beam, "laserbridges:laser_block_powered");
        assert!(!full.attached);
        assert_eq!(
            source_of(FENCE).map(|source| source.beam),
            Some("laserbridges:laser_fence_powered")
        );
        assert_eq!(
            source_of(FENCE_FULL).map(|source| source.beam),
            Some("laserbridges:laser_fence_powered")
        );
    }

    #[test]
    fn a_laser_block_and_an_unknown_id_are_not_sources() {
        assert_eq!(source_of("laserbridges:laser_block_powered"), None);
        assert_eq!(source_of("minecraft:stone"), None);
        assert_eq!(source_of(""), None);
    }

    #[test]
    fn only_the_two_wall_sources_need_a_support() {
        assert!(attached_item(BRIDGE));
        assert!(attached_item(FENCE));
        assert!(!attached_item(BRIDGE_FULL));
        assert!(!attached_item(FENCE_FULL));
        assert!(!attached_item("laserbridges:laser_block_powered"));
    }

    #[test]
    fn a_placed_source_is_evaluated_and_a_removed_one_clears_its_beam() {
        let bridge = source_of(BRIDGE).expect("a source");
        assert_eq!(
            change(None, Some(bridge)),
            Change::Evaluate,
            "a fresh source is evaluated"
        );
        assert_eq!(
            change(Some(bridge), Some(bridge)),
            Change::Evaluate,
            "the same kind re-evaluates without clearing (upstream's onPlace)"
        );
        assert_eq!(
            change(Some(bridge), None),
            Change::Clear("laserbridges:laser_block_powered"),
            "a removed source clears the beam it left"
        );
        assert_eq!(change(None, None), Change::Nothing);
    }

    #[test]
    fn a_source_replaced_by_the_other_kind_clears_the_old_beam_and_builds_the_new_one() {
        let bridge = source_of(BRIDGE).expect("a source");
        let fence = source_of(FENCE).expect("a source");
        assert_eq!(
            change(Some(bridge), Some(fence)),
            Change::Replace("laserbridges:laser_block_powered"),
            "the old bridge beam is cleared and the fence beam built in the same tick"
        );
        assert_eq!(
            change(Some(fence), Some(bridge)),
            Change::Replace("laserbridges:laser_fence_powered")
        );
    }

    #[test]
    fn a_vertical_click_mounts_on_the_floor_or_ceiling_and_a_horizontal_one_on_the_wall() {
        assert_eq!(clicked(Direction::Up), Clicked::Up);
        assert_eq!(clicked(Direction::Down), Clicked::Down);
        assert_eq!(clicked(Direction::North), Clicked::North);
        assert_eq!(clicked(Direction::South), Clicked::South);
        assert_eq!(clicked(Direction::East), Clicked::East);
        assert_eq!(clicked(Direction::West), Clicked::West);
    }

    #[test]
    fn a_face_normal_names_its_direction_and_a_non_axis_is_refused() {
        assert_eq!(direction([0, 1, 0]), Some(Direction::Up));
        assert_eq!(direction([0, -1, 0]), Some(Direction::Down));
        assert_eq!(direction([0, 0, -1]), Some(Direction::North));
        assert_eq!(direction([0, 0, 1]), Some(Direction::South));
        assert_eq!(direction([-1, 0, 0]), Some(Direction::West));
        assert_eq!(direction([1, 0, 0]), Some(Direction::East));
        assert_eq!(direction([1, 1, 0]), None);
    }

    #[test]
    fn every_direction_has_the_sdk_face_of_the_same_name() {
        for (direction, face) in [
            (Direction::Down, BlockFace::Down),
            (Direction::Up, BlockFace::Up),
            (Direction::North, BlockFace::North),
            (Direction::South, BlockFace::South),
            (Direction::West, BlockFace::West),
            (Direction::East, BlockFace::East),
        ] {
            assert_eq!(block_face(direction), face, "{direction:?}");
        }
    }
}
