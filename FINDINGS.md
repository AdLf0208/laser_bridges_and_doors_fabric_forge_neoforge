# Laser Bridges & Doors — upstream analysis and slice plan

Ported from **Laser Bridges & Doors 6.3** (MIT, `Mars-The-Planet/Laser-Bridges-And-Doors`),
read at commit `792de13e5acb5ba1f650a1b1de46bf7b43c6a620` (branch `1.21.1`, `gradle.properties`
`version=6.3`), matching the ATM10 jar `laserbridges-1.21.1-neoforge-6.3.jar`
(SHA-1 `6f3ec7b8…`). Behaviour only; the port is a clean-room Rust rewrite. Provenance and the
carry list: [`_shared/carry-work/laserbridges/SOURCE.md`](../../_shared/carry-work/laserbridges/SOURCE.md).

Class **content**, tier **machines** (ledger row). This slice is the block-state/redstone half of
that tier: it needs no block entity, so the machine-shaped part (a receiver's redstone output, the
dispenser dye path) is the deferred half.

## The source and its beam (`ISourceBlock.generateBridge`)

The mod has four source blocks, built from two behaviours crossed with two laser blocks:

| id | class | emits |
|---|---|---|
| `laser_source_block` | `AbstractAttachedSourceBlock` (needs a sturdy support) | `laser_block_powered` |
| `laser_source_fullblock` | `AbstractSourceBlock` (full block, no support check) | `laser_block_powered` |
| `laser_fence_source_block` | `AbstractAttachedSourceBlock` | `laser_fence_powered` |
| `laser_fence_source_fullblock` | `AbstractSourceBlock` | `laser_fence_powered` |

State set (`ILaserGeneratingBlock`, `AbstractSourceBlock.createBlockStateDefinition`):
`FACING` (`HORIZONTAL_FACING`, 4 values), `FACE` (`ATTACH_FACE`: `floor`/`wall`/`ceiling`),
`POWERED` (boolean), `COLOR` (0–15, a dye id). Defaults: `FACE=wall`, `FACING=north`,
`POWERED=false`, `COLOR=0`.

`generateBridge(world, power, pos, state)` (`source/ISourceBlock.java`):

1. `length = (int)(max_length / 15f * power)` — `max_length` defaults to 15, so the length is the
   redstone strength.
2. It walks `i = 1 .. max_length-1` (an **exclusive** upper bound), one cell per step from the
   source: `FACE=floor` → `+y`, `FACE=ceiling` → `-y`, `FACE=wall` → `facing` (east `+x`,
   west `-x`, south `+z`, north `-z`).
3. For each cell: stop if the cell is neither the beam block in **exactly** the wanted state
   (same colour, face and facing) nor a block in `blocks_cut_through_by_lasers`
   (`air`, `cave_air`, `void_air`, `short_grass`, `tall_grass`, `fern`, `large_fern`,
   `dead_bush`).
4. A cuttable cell is destroyed (`destroyBlock(pos, true)`, so it drops) when powered, then
   beamed over.
5. While `i < length`, place the laser block (colour + face + facing); afterwards, remove a laser
   block of the same kind found there (the shrink/clear tail).
6. The sounds `laserbridges:on` / `off` play at the source, 0.3 volume, 0.5 pitch, on every
   generation.

Consequences the port keeps:

- a full signal of 15 fills **14** cells (1..14); a signal of 1 fills **none**;
- unpowered, the walk clears every laser block of its state up to the first solid block (the loop
  still runs to `max_length-1` because air is "cuttable");
- a solid block stops the line and leaves anything beyond it alone;
- a laser block of **another** colour stops the line too (upstream's exact-state equality), so a
  colour change mid-line strands the far cells;
- two sources facing each other: the second source is a solid block, so the first stops there.

## Redstone and lifecycle (`ISourceBlock`, `AbstractSourceBlock`, `AbstractAttachedSourceBlock`)

- `neighborChanged`: `flag = state.POWERED`; if `flag != hasNeighborSignal(pos)` then a powered
  source schedules a 0-tick block tick, and an unpowered one sets `POWERED=true` and generates with
  **`getBestNeighborSignal`** (the strength, not just a boolean); if the flag already matches and it
  is powered, it schedules a 0-tick tick (so a strength change re-generates).
- `tick`: if powered and still powered, generate with the current strength; if the signal is gone,
  set `POWERED=false` and generate with 0.
- `onPlace`: the same rising-edge logic as `neighborChanged`, with flags 2.
- `onRemove`: plays `minecraft:metal_break` and generates with 0 (clears the beam).
- `useItemOn` with a `DyeItem`: consumes one dye and sets `COLOR` (the receiver does the same).
- `getShape` is the `Block.box` table; the powered forms **add** the beam's 1/16-thick plane to the
  source's own cell.

### Collision — the part the slice is judged on

The laser blocks' `getShape` (`Constants`, `LaserBridgeBlock`, `LaserFenceBlock`), in 1/16 units:

| block | floor (axis x / z) | ceiling (x / z) | wall |
|---|---|---|---|
| `laser_block_powered` | `[8,0,0,8,16,16]` / `[0,0,8,16,16,8]` | `[8,0,0,8,14,16]` / `[0,0,8,16,14,8]` | `[0,8,0,16,8,16]` |
| `laser_fence_powered` | `[0,0,8,16,16,8]` / `[8,0,0,8,16,16]` | `[0,0,8,16,14,8]` / `[8,0,0,8,14,16]` | `[8,0,0,8,16,16]` north/south, `[0,0,8,16,16,8]` east/west |

So a **wall-mounted bridge source** builds a horizontal plate at 8 px (its top is exactly half a
block above the course) — that is the walkable "bridge"; a **floor or ceiling source** builds a
thin vertical wall — the "door" / "fence". Vanilla's `getCollisionShape` defaults to `getShape`, so
the plate is collidable: that is the whole point of the mod.

## Receivers (`IReceiverBlock`) — deferred

`laser_attached_receiver` / `laser_attached_receiver_fence` look at the cell in front of them: if it
is a laser block or a **powered** source, they set `POWERED=true` and copy its colour, and they are
`isSignalSource` with `getSignal = POWERED ? 15 : 0` (0 on their own facing). `onRemove` notifies
their neighbours. That is a small block-state slice of its own; it is not in this slice.

## What this slice ships, and the shape it takes (machines tier)

`horizon/docs/modpacks.md`: `machines` = content + use/place/break/tick hooks, block entities,
menus, capabilities. This slice implements the **content + block-state/redstone** half:

- **Content** (`.hmod`): the four sources and two lasers, their states, shapes, hardness, light,
  models, blockstates, textures, sounds, lang, loot, the pickaxe tag, six recipes, a creative tab.
- **Native** (`.hnmod`): the beam model as one pure, unit-tested `runtime.rs`
  (`length`, `offset`, `plan`, `placement`, `horizontal`) plus a support module that reads the
  redstone signal and writes the line.
- **Hooks** (all stamped at `1d63e0a5`, API 411): `state_for_placement` (1), `neighbor_changed` (6),
  `block_changed` (7), `level_tick_post` (23), with `native_set_block` (24) and
  `native_neighbor_signal` (96) from the tick hook.

### Why the port defers to `level_tick_post`

Upstream regenerates inside `neighborChanged`/`onPlace`, with the level in hand. At the pin
`native_neighbor_changed` and `native_block_changed` are `&self` on `GameLevel`
(`crates/horizon-client/src/game/native_hooks.rs:1207-1218`), which
"[cannot call `native_set_block`]" (`docs/native-mods.md:245`), so the port queues the source and
applies it at `native_level_tick_post` (API 23), where the level is in place. Cost: a source reacts
one server tick late; a second pass writes nothing because `native_set_block` refuses an unchanged
state, so the beam settles in one tick.

`block_changed` carries the **old** state as well as the new one, so a removed (or replaced) source
clears the beam it left in the same tick, with no per-position memory to lose across a reload.

### Findings at the pin (recorded for the notes)

- `native_neighbor_changed` gives the state but not a mutable level; the queue is the only way to a
  write from it.
- A **redstone wire cannot power these blocks**: `wire_signal` returns 0 for `Direction::Down`
  (`crates/horizon-world/src/level/redstone.rs:126-133`), so a wire above a source does nothing; and
  a wire cannot sit on the source either, because the source's declared thin outline is not sturdy.
  A weighted pressure plate beside it does report its own `power`, which is how the e2e reaches a
  signal of 7.
- A content block's `light` is per block, not per state (`crates/horizon-mods/src/content.rs:77-99`),
  so the attached sources' `powered`-dependent light becomes a constant 10.
- A run of `setblock` through `ctl command` lands **one server tick later**; a read in the same
  breath still sees the old block (this cost one e2e iteration).

## Gaps (this slice)

1. **Receivers** (`laser_attached_receiver`, `laser_attached_receiver_fence`): blocks, redstone
   output, colour copy — not shipped.
2. **Dyeing**: `useItemOn` with a `DyeItem` sets `COLOR`; the state and the render tint exist in
   content, but no hook sets the colour yet, so every beam is colour 0.
   The client tint handler (`RegisterColorHandlersEvent.Block` in `LaserBridges.java`) has no
   Horizon equivalent here, so the carried models' `tintindex: 0` leaves the texture untinted.
3. **Sounds**: `laserbridges:on`/`off` are carried but not played (the pin's
   `native_play_sound` takes vanilla sound events).
4. **The dispenser dye behaviour** (`CommonClass.init`'s `DispenseItemBehavior` for dyes) is not
   ported.
5. **Config**: `max_length` (15) and `blocks_cut_through_by_lasers` are constants in `runtime.rs` /
   the support module; upstream's `config/laserbridges.json` (Deimos) is not read.
6. **Explosions/pistons**: a beam block is unbreakable (bedrock template), and a piston that moves a
   source is not hooked; upstream has no piston behaviour either, but a moved source would leave its
   beam behind here.
7. **A colour change mid-line** strands the far cells (upstream's exact-state equality, kept).
8. **`/setblock` of a source at a position whose chunk is unloaded**: the queued job is dropped with
   the state (upstream has no such case; `world.loaded` gates the read).
