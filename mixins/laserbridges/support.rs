// Game-side glue for Laser Bridges & Doors: the four laser sources and the
// beam blocks they write.
//
// Injected into `crate::game` as `laserbridges_state`; it needs the game, so it
// is not part of the standalone library (only `runtime.rs` is).
//
// Upstream recomputes a source in `neighborChanged` and `onPlace`, with the
// level in hand. At the pin the neighbour and block-change hooks hold only a
// `GameLevel`, which cannot call `native_set_block`, so the port queues the
// source and applies it at `level_tick_post`, where the level is in place:
// `getBestNeighborSignal` -> the `powered` state -> `generateBridge`. A source
// reacts one server tick late, then settles (`native_set_block` refuses an
// unchanged state, so the second pass writes nothing).
//
// `block_changed` carries the old and the new state, so a removed or replaced
// source clears the beam it left in the same tick, without a per-position memory
// that a world reload would lose.

use std::collections::BTreeMap;
use std::sync::Mutex;

use horizon_core::geometry::Direction;
use horizon_core::position::BlockPos;
use horizon_world::level::Dimension as LevelDimension;
use horizon_world::level::{PlacementContext, flags};
use horizon_world::registry::{BlockRegistry, BlockStateId};

use crate::game::laserbridges::{
    Beam, Cell, Clicked, Face, Facing, MAX_LENGTH, Write, placement, plan,
};

/// One source block: the laser block it writes, and whether it hangs on a
/// support (upstream's `AbstractAttachedSourceBlock`; the full-block sources
/// have no `canSurvive`).
#[derive(Clone, Copy)]
struct Source {
    beam: &'static str,
    attached: bool,
}

/// `ModRegistry.RegisterSetup`: each source and the laser block it generates.
const SOURCES: [(&str, Source); 4] = [
    (
        "laserbridges:laser_source_block",
        Source {
            beam: "laserbridges:laser_block_powered",
            attached: true,
        },
    ),
    (
        "laserbridges:laser_source_fullblock",
        Source {
            beam: "laserbridges:laser_block_powered",
            attached: false,
        },
    ),
    (
        "laserbridges:laser_fence_source_block",
        Source {
            beam: "laserbridges:laser_fence_powered",
            attached: true,
        },
    ),
    (
        "laserbridges:laser_fence_source_fullblock",
        Source {
            beam: "laserbridges:laser_fence_powered",
            attached: false,
        },
    ),
];

/// `LaserBridgesConfig.blocks_cut_through_by_lasers`, minus the three airs,
/// which the caller classifies as [`Cell::Air`].
const CUTTABLE: [&str; 5] = [
    "minecraft:short_grass",
    "minecraft:tall_grass",
    "minecraft:fern",
    "minecraft:large_fern",
    "minecraft:dead_bush",
];

/// A source position's pending work for the next level tick.
#[derive(Clone, Copy, Default)]
struct Job {
    /// Read the redstone input and regenerate.
    evaluate: bool,
    /// Clear the beam a removed (or replaced) source left: its geometry and the
    /// laser block it wrote.
    clear: Option<(Beam, &'static str)>,
}

#[derive(Default)]
struct State {
    jobs: BTreeMap<(String, [i32; 3]), Job>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn state() -> std::sync::MutexGuard<'static, Option<State>> {
    STATE
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

/// A parsed source state.
#[derive(Clone, Copy)]
struct Parsed {
    source: Source,
    beam: Beam,
    powered: bool,
}

fn parse(registry: &BlockRegistry, state: BlockStateId) -> Option<Parsed> {
    let state = registry.state(state)?;
    let (_, source) = SOURCES.iter().find(|(id, _)| *id == &*state.name)?;
    let face = Face::from_name(state.properties.get("face")?)?;
    let facing = Facing::from_name(state.properties.get("facing")?)?;
    let colour = state.properties.get("color")?.parse::<u8>().ok()?;
    let powered = state
        .properties
        .get("powered")
        .is_some_and(|value| value == "true");
    Some(Parsed {
        source: *source,
        beam: Beam {
            face,
            facing,
            colour,
        },
        powered,
    })
}

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

/// `AbstractSourceBlock.getStateForPlacement`: the attachment and the facing
/// come from the click; an attached source also needs a sturdy support behind
/// it (`FaceAttachedHorizontalDirectionalBlock.canSurvive`), while the
/// full-block sources accept any placement.
#[expect(
    private_interfaces,
    reason = "the hook helpers are `pub` for the bundle and only named inside this crate, which owns the game types",
)]
pub fn on_state_for_placement(
    level: &crate::game::GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    let Some(parsed) = parse(level.registry, state) else {
        return;
    };
    if parsed.source.attached
        && !level.behaviours.is_face_sturdy(
            level
                .world
                .block(context.hit_pos.x, context.hit_pos.y, context.hit_pos.z),
            context.face,
        )
    {
        *answer = Some(None);
        return;
    }
    let (face, facing) = placement(clicked(context.face), context.yaw);
    if let Some(new) = with_properties(
        level.registry,
        state,
        &[("face", face.name()), ("facing", facing.name())],
    ) {
        *answer = Some(Some(new));
    }
}

fn queue(dimension: &str, pos: [i32; 3], job: Job) {
    let mut guard = state();
    let state = guard.get_or_insert_with(State::default);
    let entry = state.jobs.entry((dimension.to_owned(), pos)).or_default();
    entry.evaluate |= job.evaluate;
    if job.clear.is_some() {
        entry.clear = job.clear;
    }
}

/// A neighbour changed under `pos`: a source's redstone input may have moved.
#[expect(
    private_interfaces,
    reason = "the hook helpers are `pub` for the bundle and only named inside this crate, which owns the game types",
)]
pub fn on_neighbor_changed(
    level: &crate::game::GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    state: BlockStateId,
) {
    if parse(level.registry, state).is_none() {
        return;
    }
    queue(
        dimension_name(dimension),
        [pos.x, pos.y, pos.z],
        Job {
            evaluate: true,
            clear: None,
        },
    );
}

/// A block changed: a new source is evaluated, and the beam of a source that
/// went away is cleared. Upstream does both in `onPlace`/`onRemove`; the port
/// does them at the next level tick.
#[expect(
    private_interfaces,
    reason = "the hook helpers are `pub` for the bundle and only named inside this crate, which owns the game types",
)]
pub fn on_block_changed(
    level: &crate::game::GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    old: BlockStateId,
    new: BlockStateId,
) {
    if old == new {
        return;
    }
    let before = parse(level.registry, old);
    let after = parse(level.registry, new);
    let cleared = before.map(|parsed| (parsed.beam, parsed.source.beam));
    // A source replaced by another kind of source clears its old beam too
    // (upstream runs `onRemove` before `onPlace`); the same source type only
    // needs re-evaluating.
    let clear = match (cleared, after) {
        (Some((beam, block)), Some(after)) if after.source.beam != block => Some((beam, block)),
        (Some(_), Some(_)) => None,
        (cleared, None) => cleared,
        (None, Some(_)) => None,
    };
    let job = Job {
        evaluate: after.is_some(),
        clear,
    };
    if job.evaluate || job.clear.is_some() {
        queue(
            dimension_name(dimension),
            [pos.x, pos.y, pos.z],
            job,
        );
    }
}

/// `level_tick_post`: read each queued source's redstone input, set its
/// `powered` state and generate (or clear) its beam.
#[expect(
    private_interfaces,
    reason = "the hook helpers are `pub` for the bundle and only named inside this crate, which owns the game types",
)]
pub fn on_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    let ready: Vec<([i32; 3], Job)> = {
        let mut guard = state();
        let state = guard.get_or_insert_with(State::default);
        let keyed: Vec<(String, [i32; 3])> = state
            .jobs
            .keys()
            .filter(|(dim, _)| dim == dimension)
            .cloned()
            .collect();
        let mut ready = Vec::with_capacity(keyed.len());
        for key in keyed {
            if let Some(job) = state.jobs.remove(&key) {
                ready.push((key.1, job));
            }
        }
        ready
    };
    for (pos, job) in ready {
        if let Some((beam, block)) = job.clear {
            generate(game, dimension, pos, beam, block, 0);
        }
        if !job.evaluate {
            continue;
        }
        let signal = game.native_neighbor_signal(dimension, pos);
        let power = u8::try_from(signal).unwrap_or(0).min(15);
        let prepared = {
            let registry = &game.pack.assets.registry;
            let Some(current) = block_at(game, pos) else {
                continue;
            };
            let Some(parsed) = parse(registry, current) else {
                continue;
            };
            let powered = if power > 0 { "true" } else { "false" };
            let new = if parsed.powered == (power > 0) {
                None
            } else {
                with_properties(registry, current, &[("powered", powered)])
            };
            (parsed, new)
        };
        let (parsed, new) = prepared;
        if let Some(new) = new {
            game.native_set_block(dimension, pos, new, flags::UPDATE_ALL);
        }
        generate(game, dimension, pos, parsed.beam, parsed.source.beam, power);
    }
}

/// `ISourceBlock.generateBridge`: walk the line from the source, placing the
/// laser block while inside `length`, and clearing the ones already there
/// beyond it.
fn generate(
    game: &mut crate::game::Game<'_>,
    dimension: &str,
    origin: [i32; 3],
    beam: Beam,
    block: &str,
    power: u8,
) {
    let colour = beam.colour.to_string();
    let Some(target) = beam_state(game, block, beam, &colour) else {
        return;
    };
    let writes = {
        let registry = &game.pack.assets.registry;
        plan(beam, power, MAX_LENGTH, |at| {
            classify(game, registry, offset(origin, beam.offset(at)), target)
        })
    };
    let Some(air) = game.pack.assets.registry.default_state("minecraft:air") else {
        return;
    };
    for write in writes {
        let at = match write {
            Write::Cut { at } | Write::Place { at } | Write::Clear { at } => at,
        };
        let pos = offset(origin, beam.offset(at));
        match write {
            Write::Cut { .. } => {
                // `Level.destroyBlock(pos, true)`: the cut plant drops.
                let _ = game.native_machine_destroy_block(pos, None, false);
            }
            Write::Place { .. } => {
                if block_at(game, pos) != Some(target) {
                    game.native_set_block(dimension, pos, target, flags::UPDATE_ALL);
                }
            }
            Write::Clear { .. } => {
                let clears = block_at(game, pos)
                    .and_then(|current| game.pack.assets.registry.state(current))
                    .is_some_and(|current| &*current.name == block);
                if clears {
                    game.native_set_block(dimension, pos, air, flags::UPDATE_ALL);
                }
            }
        }
    }
}

/// This beam's laser-block state: its block, the beam's `face`, `facing` and
/// `color`, and the block's own default for anything else.
fn beam_state(
    game: &crate::game::Game<'_>,
    block: &str,
    beam: Beam,
    colour: &str,
) -> Option<BlockStateId> {
    let registry = &game.pack.assets.registry;
    let base = registry.default_state(block)?;
    with_properties(
        registry,
        base,
        &[
            ("face", beam.face.name()),
            ("facing", beam.facing.name()),
            ("color", colour),
        ],
    )
}

/// The state with the named properties replaced.
fn with_properties(
    registry: &BlockRegistry,
    state: BlockStateId,
    properties: &[(&str, &str)],
) -> Option<BlockStateId> {
    let mut descriptor = registry.state(state)?.clone();
    for (name, value) in properties {
        descriptor
            .properties
            .insert((*name).to_owned(), (*value).to_owned());
    }
    registry.resolve(&descriptor).ok()
}

/// What `generateBridge` finds in one cell.
fn classify(
    game: &crate::game::Game<'_>,
    registry: &BlockRegistry,
    pos: [i32; 3],
    target: BlockStateId,
) -> Cell {
    let Some(current) = block_at(game, pos) else {
        // A cell in an unloaded chunk stops the line, as a solid block does.
        return Cell::Solid;
    };
    if current == target {
        return Cell::Same;
    }
    let Some(state) = registry.state(current) else {
        return Cell::Solid;
    };
    match &*state.name {
        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air" => Cell::Air,
        name if CUTTABLE.contains(&name) => Cell::Cuttable,
        _ => Cell::Solid,
    }
}

fn offset([x, y, z]: [i32; 3], [dx, dy, dz]: [i32; 3]) -> [i32; 3] {
    [x + dx, y + dy, z + dz]
}

fn dimension_name(dimension: LevelDimension) -> &'static str {
    match dimension {
        LevelDimension::Overworld => "minecraft:overworld",
        LevelDimension::Nether => "minecraft:the_nether",
        LevelDimension::End => "minecraft:the_end",
    }
}

/// The block at `pos`, or `None` outside the world or in an unloaded chunk.
fn block_at(game: &crate::game::Game<'_>, [x, y, z]: [i32; 3]) -> Option<BlockStateId> {
    let world = &game.world;
    if !(-30_000_000..30_000_000).contains(&x)
        || !(-30_000_000..30_000_000).contains(&z)
        || !(world.shape.min_y..world.shape.max_y()).contains(&y)
        || !world.loaded(x, z)
    {
        return None;
    }
    Some(world.block(x, y, z))
}
