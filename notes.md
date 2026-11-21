# Laser Bridges & Doors — port notes

Ported from **Laser Bridges & Doors 6.3** (`Mars-The-Planet/Laser-Bridges-And-Doors`, **MIT**),
branch `1.21.1`, commit `792de13e5acb5ba1f650a1b1de46bf7b43c6a620` (2026-10-31). Minecraft Java
1.21.1 NeoForge, the exact ATM10 jar `laserbridges-1.21.1-neoforge-6.3.jar` (SHA-1
`6f3ec7b8e77450ed0a0865e3ca646ab41464adf3`); provenance in
[`_shared/carry-work/laserbridges/SOURCE.md`](../_shared/carry-work/laserbridges/SOURCE.md).
The upstream Java was read for behaviour only; this is a clean-room Rust rewrite.

**Licence: MIT → rank 0.** Local-only until the maintainer approves publishing (ARCHITECTURE.md §8).
Queue: core mechanics, no required ATM10 parent.

**Class + tier:** `content` · `machines` (the ledger row). Both slices are the block-state/redstone
half of the tier: no block entity is needed, and the receivers' redstone output is now shipped as a
block part (`block_signal` / `is_signal_source`), so what remains machine-shaped is the dispenser dye
path and the dye interaction, both deferred rather than pretended.

**Status: slice 2 implemented and verified in-game** — the two receivers
(`laser_attached_receiver`, `laser_attached_receiver_fence`) now ship beside slice 1's four sources
and two beams: they power on the laser block (or powered source) in front of them, copy its colour,
emit light only while powered, answer a redstone signal on every face but the one they hang from
(the direct signal on the face they look into), and are attached blocks with the same sturdy-support
rule. **0.2.1** also repoints the package at pin `0a875da` and removes the powered-light deviation
with the pin's `state_light`.

**Package (0.2.1):** one flat SDK `.hmod` (`horizon-mod.toml`, `api = "0.2"`, code and content in the
same archive) — [`laserbridges.hmod`](laserbridges.hmod), 77,460 B, 84 entries, SHA-256
`192d58ff59c5…3656` (see [Slice 2](#slice-2--the-receivers-021-2026-11-21) for the measured ladder).
The retired `.hnmod` tree is kept as evidence at
[`_shared/evidence/laserbridges/retired-hnmod-0.1.0/`](../_shared/evidence/laserbridges/retired-hnmod-0.1.0/).
How the glue was rewritten: [Migration to the SDK format](#migration-to-the-sdk-format-020-2026-11-18).

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70; the registry closed every `.hnmod` submission
with one reason) replaced the native container with a **flat `.hmod`**: `horizon-mod.toml`
(`api = "0.2"`) + `Cargo.toml` + `src/lib.rs` (one `impl mod_api::Hooks`) + `assets/` + `data/`, built
by the launcher against the pinned game build. Pins: SDK/nightly
**`0b14f8acfd97fa2c750e42795249dcc981178ca4`**, mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, for 26.3. This supersedes closed PR **#805**.
Reference recipes: `ports/imfast/notes.md` (the 12-step original) and `ports/naturesaura/notes.md`
(blocks, parts, a content half at the root).

**Order done in this job:** the retired paths were moved to evidence *before* the first build (the
packer refuses a root holding a retired package path), `runtime.rs` was copied with `cp` and proved
with `cmp` + `sha256sum`, `cargo test` ran on the new crate with **no glue** (15/15 runtime tests
green), and only then was `src/lib.rs` written — with the pure helpers stubbed as `todo!()` so the
glue's first run was **red** (8 failed, `not yet implemented`), then green (23 passed).

### File map (old → new)

| Old `.hnmod` layout | New SDK package |
|---|---|
| `horizon-mixin.toml` (`[mod]`, `[compatibility]`, 2 `[[module]]`, 5 `[[injection]]`) | `horizon-mod.toml` (id/name/version/description/authors/api/side/capabilities + `[native]` game/horizon/mixins) |
| `Cargo.toml` (`[lib] path = "mixins/laserbridges/runtime.rs"`, `[workspace] members = ["tools/audit"]`) | `Cargo.toml` (`[lib]` default `src/lib.rs`, one dependency: `horizon-mod-api` from `.horizon/sdk/`, `publish = false`) |
| `mixins/laserbridges/runtime.rs` (crate lib *and* injected module) | `src/runtime.rs` — **byte-identical** (`ec94f3be…cd94a`), `pub mod runtime;` at the crate root |
| `mixins/laserbridges/support.rs` (the `laserbridges_state` module: queue, parse, generate) | `src/lib.rs` — the private helpers (`Source`, `Job`, `Jobs`, `parse`, `generate`, `classify`) and the `Mutex` queue |
| 5 per-hook helper `.rs` files (`placement.rs`, `neighbor.rs`, `changed.rs`, `tick_post.rs`, `embedded_content.rs`) | methods of `impl Hooks` / a `BlockPart`; no per-hook files |
| `content/` (`assets/`, `data/`, its own `horizon-mod.toml`, the built `laserbridges.hmod`) | package root: `assets/`, `data/`; the content manifest merges into the root one; no separate content archive |
| `mixins/laserbridges/data/laserbridges-content.hmod` + `embedded_content.rs` (the AE2 write-out) | **gone**: one package carries code and content, so there is nothing to embed or append to the dev list |
| `compatibility.json`, `rust-toolchain.toml` | **gone**: `[native]` in `horizon-mod.toml` is the pin, the launcher's toolchain builds |
| `tools/horizon.py`, `tools/package.py`, `tools/ladder-embed.sh`, `tools/hmod-load.sh`, `tools/notice.py`, `tools/audit/` | **gone** (moved to evidence); the ladder is `cargo test` + `cargo horizon-mod build/check/fits` |
| `laserbridges-0.1.0.hnmod` (the 0.1.0 artifact) and `laserbridges.hnmod` (an auxiliary byte-identical copy, written 8 min later by the packaging step) | `laserbridges.hmod` (62,787 B, 63 entries) |

**Kept and updated:** `tools/gen_content.py` and `tools/patch_content.py` (now write/read the package
root instead of `content/`), `tools/e2e.sh` (now launches a launcher-built game; `GAME=`,
`CONTROL_GAME=`, `SELFTEST=1`), `FINDINGS.md`, `VALIDATION.md`, this file, `LICENSE`,
`LICENSE-NOTICE.md` (new), `README.md` (rewritten for 0.2.0).
**Retired into evidence:** `mixins/`, both `.hnmod` files, `compatibility.json`,
`rust-toolchain.toml`, `Cargo.lock`, the old `AGENTS.md` (it told readers to open `compatibility.json`
and run `tools/horizon.py`), `content/` (assets/data were *moved* to the root; the old manifest, the
built 42,865 B content `.hmod` and `content/target/` stayed behind as evidence), and the tools above.

### Hook → SDK table

| Old `[[injection]]` / helper | New place | Mapping notes |
|---|---|---|
| `state-for-placement` (`placement.rs`: clicked face → `face`/`facing`) | `BlockPart::state_for_placement` on `SourcePart`, attached to the four source blocks by `Registrar::block_part` in `Hooks::register` | the part is per block, so the game asks it only for our sources; `event.context.face`/`.yaw` are the old `PlacementContext`, and `level.state_with_property` builds the state |
| the attached sources' sturdy-support requirement (was inside `placement.rs`) | `Hooks::use_item_on_block`, `UsePhase::ItemBeforeBlock`, `Flow::Cancel` + `InteractionResult::Fail` | **moved, not dropped**: the placement moments get only `&dyn LevelHost`, which has no sturdy-face read; the pre-block use moment has the host and `LevelRef::block_face_sturdy` (`SupportType::Full`, upstream's 3-arg `isFaceSturdy`) |
| `neighbor-changed` (`neighbor.rs`) | `BlockPart::neighbor_changed` on `SourcePart` | a block-scoped part hook: it fires only for our sources, so the old `parse(..).is_none()` guard is gone; it queues the position for the next tick |
| `block-changed` (`changed.rs`) | `Hooks::block_changed` (global, `level: &dyn LevelHost`) | `event.old`/`.new` are `BlockState`s; `level.state_block(..)` names them and `change()` decides. The *old state* rides in the job, because this moment has no metadata service to read its `face`/`facing`/`color` |
| `level-tick-post` (`tick_post.rs`) | `Hooks::level_tick_post` (`HostSide::Server`, `event.level: LevelRef`) | reads `best_neighbor_signal`, writes `powered` and the beam through `LevelRef::{set_block,destroy_block}` with `NEIGHBORS \| CLIENTS` (`UPDATE_ALL`); metadata from `host.block_states()` |
| `embedded-content` (`content_root` HEAD) | **none** | `assets/` + `data/` are loaded because they are in the same package |
| `[[module]] runtime` / `[[module]] state` | `pub mod runtime;` / private items in `src/lib.rs` | the queue is `static JOBS: Mutex<Option<Jobs>>`, as 0.1.0's `static STATE` was: a block part has no route to the mod value the macro boxes |

No hook, method or capability of the retired bundle is left homeless except `embedded-content`,
which the format deletes. Nothing new was invented: the `use_item_on_block` home is the SDK's own
pre-block, cancellable moment for placement.

### Package and ladder (measured 2026-11-18)

Every step ran in a `ports/tools/build-slot.sh` lease with `BUILD_LOCAL=1` and `CARGO_INCREMENTAL=0`,
directly in the port (`build-slot.sh` still speaks the retired `tools/horizon.py` ladder, so it is
used as the slot guard), no `--target-dir`.

| Check | Command | Result |
|---|---|---|
| Runtime tests, before any glue existed | `cargo test` | **15 passed**, 0 failed (`evidence/laserbridges/sdk-test-runtime.log`) |
| Glue first red | `cargo test` with the helpers as `todo!()` | **15 passed, 8 failed** — every failure `not yet implemented` (`sdk-glue-red.log`) |
| Tests after the glue | `cargo test` | **23 passed**, 0 failed (`sdk-test.log`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** (`sdk-clippy.log`) |
| Formatting | `cargo fmt --check` (not `--all`) | clean, exit 0 (`sdk-fmt.log`) |
| Package | `rm -f Cargo.lock && cargo horizon-mod build` | `target/horizon-mod/laserbridges.hmod` (`sdk-build.log`) |
| Registry check | `cargo horizon-mod check laserbridges.hmod` | exit 0: `laserbridges 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)`, **62,787 B**, SHA-256 `f8820b1a…8ed8`, unsigned (`sdk-check.log`) |
| Fits the pinned build | `cargo horizon-mod fits laserbridges.hmod --build nightly` | `0b14f8a  made for`, exit 0 (`sdk-fits.log`) |
| Same package twice | a second `rm -f Cargo.lock && cargo horizon-mod build` | 62,787 B, `f8820b1a…8ed8` both times (`sdk-build-second.log`, `sdk-sha256.txt`) |
| `runtime.rs` untouched | `unzip -p laserbridges.hmod src/runtime.rs \| sha256sum` | `ec94f3be…cd94a` = the retired file (`sdk-runtime-sha256.txt`) |
| Carried content unchanged | size + sha256 of all 55 carried files vs the retired copy | identical lists (`sdk-content-carry.txt`) |
| Contents | `unzip -l laserbridges.hmod` | **63 entries**: 35 `assets/`, 20 `data/`, `src/lib.rs`, `src/runtime.rs`, `horizon-mod.toml`, `Cargo.toml`, `README.md`, `LICENSE`, `LICENSE-NOTICE.md`, `uses.json`; **no `Cargo.lock`** (`sdk-entries.txt`) |

`uses.json` (in the package, `sdk-uses.json`, summarised in `sdk-uses-summary.txt`): **183 entries** —
5 hooks (`block_changed`, `level_tick_post`, `neighbor_changed`, `state_for_placement`,
`use_item_on_block`) with their fields, 2 parts (`block`, `item`), `Hooks::register`,
`BlocksRegistrar::block_key`, `Registrar::block_part`, the `LevelRef` handles (`block`, `block_state`,
`set_block`, `destroy_block`, `best_neighbor_signal`, `block_face_sturdy`, `dimension`), the
`BlockMetadata` reads (`name`, `property`, `with`) and `BlockState::of`, `HostBlockStates::block_states`,
`KeysHost::block_key`, `EntitiesHost::level`, the `registries` capability, 2 macros and 16 traits.

### Traps this migration hit (new to this port)

1. **`host.block_states()` needs `HostBlockStates` imported.** It is an extension trait
   (`impl<H: Host> HostBlockStates for H`), so without the import the compiler suggests
   `block_state_string` and the error looks like a missing method.
2. **`BlockState` cannot be built in a test.** `BlockKey`'s field is crate-private and only the
   registry makes keys, so a unit test can never hold a `BlockState`. Any decision a test must see
   has to be lifted off it: `change(old, new)` answers `Change::{Nothing,Evaluate,Clear,Replace}`
   over `Source` values, and the hook attaches the old state afterwards.
3. **`event.cancellation_result` is `&mut InteractionResult`**, not a field to assign: write
   `*event.cancellation_result = …`.
4. **A block part's placement moments get `&dyn LevelHost` only** — no `Host`, no
   `LevelRef::block_face_sturdy`. `state_for_placement`'s `Some(None)` (upstream's `null`) therefore
   cannot test the support, and `can_survive` cannot either; the check lives in
   `Hooks::use_item_on_block` (`UsePhase::ItemBeforeBlock`), whose `HostSide` has the level and the
   sturdy read. The observable outcome is upstream's: nothing placed, the stack kept.
5. **`block_changed` has no metadata service** (`level: &dyn LevelHost`), so the removed source's
   `face`/`facing`/`color` cannot be parsed there. The job carries the old `BlockState` (a
   process-local key + ordinal, valid for the tick) and `level_tick_post` parses it through
   `host.block_states()`.
6. **`Host::level(dimension)` is declared on `EntitiesHost`**, not `LevelsHost`; that is the only way
   from a dimension id to a `LevelRef` in the use hook.
7. **`DimensionKey` is opaque**: only `.id(host)` names it. The queue is keyed by that string, and
   `LevelDimension::key().id()` (the block moments) gives the same ids, as 0.1.0's
   `dimension_name()` did.
8. **The packer does not pack this file.** The 63 entries carry `README.md` and the licences but not
   `notes.md`/`FINDINGS.md`/`VALIDATION.md`, so those stay repository documents; nothing important
   may live only in them.

### Gaps (the migration closes none and adds two records)

> Slice 2 (0.2.1) closed item 1 and the light half of item 3; see
> [Gaps after slice 2](#gaps-after-slice-2).

1. ~~The two receivers (`laser_attached_receiver`, `laser_attached_receiver_fence`).~~
2. Dyeing a source (upstream's `useItemOn` with a dye, and the dispenser's dye behaviour).
3. The `laserbridges:on`/`off` playback (carried sounds, never played).
4. `config/laserbridges.json` (`max_length`, `blocks_cut_through_by_lasers`): both are constants.
5. A colour change mid-line strands the far cells (upstream's exact-state test, kept deliberately).
6. The beams are unbreakable like upstream (bedrock template), so there is no drop or
   break-progress path.
7. **No 0.2.0 in-game run.** `horizon-launcher` is not installed here, so no game carrying this
   package can be built; `tools/e2e.sh` is updated for a launcher-built game and its 0.1.0 logs are
   historical (the rules they exercised are `runtime.rs`, byte-identical). A behaviour claim for
   0.2.0 waits for that run.
8. The support refusal now fires at the use moment rather than inside `getStateForPlacement`
   (trap 4); if a future SDK hands placement a host, it should move back.

### Changelog for the resubmission (supersedes #805)

> Supersedes #805, closed with the `.hnmod` retirement (Horizon's mod-API merge #16941, launcher
> ≥ 0.2.70). 0.2.0 is the flat SDK package: `api = "0.2"`, one `impl Hooks` in `src/lib.rs`, the
> content at the package root, `[native]` pinned to nightly
> `0b14f8acfd97fa2c750e42795249dcc981178ca4` and mixins
> `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, for 26.3. What is new for `api = "0.2"`: the five
> retired injections became four typed moments — `block_changed`, `level_tick_post`,
> `BlockPart::neighbor_changed`, `BlockPart::state_for_placement` — plus one `use_item_on_block`
> check that keeps the attached sources' sturdy-support rule (the placement moment cannot read a
> sturdy face); the embedded-content write-out is gone because code and content are one package; the
> per-hook helper files and the `state`/`runtime` module plan are gone. `src/runtime.rs` and its 15
> tests are unchanged, byte for byte. Package: 63 entries, **62,787 B**, SHA-256
> `f8820b1a85a47b85bc88964ae77ed13083e86b707ea696c007957d508d9b8ed8`; `check` and `fits` exit 0; a
> second build is byte-identical. No 0.2.0 in-game run: the launcher is not installed here (gap 7).

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

## Slice 2 — the receivers (0.2.1, 2026-11-21, pin `0a875da`)

**The gap it closes** — `notes.md` Gaps item 1 and `FINDINGS.md` gap 1, verbatim:

> The two receivers (`laser_attached_receiver`, `laser_attached_receiver_fence`): blocks, models,
> textures, lang, recipes and their redstone output (`is_signal_source` + `block_signal` + the
> colour copy from the beam in front).

**Why this family, and why it fits `machines`.** It is the last large family in the jar and the one
that turns the mod from "a beam appears" into "a beam does something": redstone in, a laser, redstone
out. The ledger's tier is `machines`, whose shape is "content + use/place/break/tick hooks, block
entities, menus, capabilities" — a receiver is content plus a block *part* (`block_signal`,
`is_signal_source`) and the same deferred write the sources use (`level_tick_post`), with no block
entity, menu or renderer. A block entity here would be over-building; a plain content block without
the part would be the "registers, so it must work" illusion.

The other candidates were rejected on evidence, not taste: dyeing needs `use_item_on_block` with a
dye and only sets a `color` state whose texture tint has no client equivalent at this pin (a state
change a player cannot see — the wrong thing to spend a slice on before the visible family); the
on/off sounds are a carried asset the pin cannot play (`native_play_sound` takes vanilla events); the
config file is a `data`-shaped feature behind a `machines` port, and it would change `max_length`
without any behaviour the receiver slice does not already prove.

### The rule (pure, `src/runtime.rs`)

`IReceiverBlock.updateState` / `handleGetSignal` / `handleGetDirectSignal` (upstream files read:
`blocks/receiver/{IReceiverBlock,AbstractAttachedReceiverBlock,BridgeAttachedReceiverBlock,
FenceAttachedReceiverBlock}.java`, commit `792de13`):

- `look(face, facing)` — the cell a receiver reads: `floor → up`, `ceiling → down`, `wall → facing`
  (upstream's `dir`).
- `Front::{Laser{colour}, PoweredSource{colour}, Other}` — what it finds there: a laser block, a
  *powered* source, or anything else.
- `receiver_state(powered, colour, front)` / `receiver_update(...)` — a receiver is powered by a
  laser or powered source and carries that block's colour; otherwise unpowered with the colour it
  already had (upstream assigns `color` only inside the match). `receiver_update` returns `Some`
  only when the pair moves, so a settled receiver writes nothing.
- `receiver_signal(powered, face, facing, asked, direct)` — 15 when powered, `0` on the face it
  hangs from (`facing.getOpposite()`), and for a direct signal only on the face it looks into.
- `state_property(state, name)` — the property of a serialized state, for the signal moment (see
  below).

### Hooks (SDK 0.2, `src/lib.rs`)

| Moment | Part / hook | What it does |
|---|---|---|
| `register` | `Registrar::block_part` | attaches `ReceiverPart` to the two receiver blocks (the game asks it only for them) |
| `state_for_placement` | `BlockPart` | `getStateForPlacement` — the same click→face/facing rule as the sources |
| `neighbor_changed` | `BlockPart` | queues the receiver for the next level tick (the level's own tick moments cannot write) |
| `block_signal` | `BlockPart` | `getSignal`/`getDirectSignal` through `receiver_signal`; the state's `powered`/`face`/`facing` are read from `level.state_string(..)`, because this moment gets only `&dyn LevelHost` (no metadata service) |
| `is_signal_source` | `BlockPart` | `true` always, as upstream's `handleIsSignalSource` |
| `level_tick_post` | `Hooks` | `evaluate_receiver`: read the front cell, write `powered`/`color` with `UPDATE_ALL` when `receiver_update` says so |
| `block_changed` | `Hooks` | a placed/replaced/removed receiver queues an evaluation (nothing to clear: its signal goes with the block) |
| `use_item_on_block` | `Hooks` | the sturdy-support rule now covers the receiver items too (`sturdy_item`) |

No new API gap: `block_signal` v1 and `is_signal_source` v1 are both stamped at `0a875da`, and the
fixtures in the SDK (`parts/blocks_fixtures/signal_game_thread.rs`) exercise the same moment.

### Red → green (TDD)

The runtime rule was written as `todo!()` stubs with its tests first, so the first run failed for the
right reason:

```
running 35 tests
test runtime::tests::a_receiver_powers_on_a_laser_and_copies_its_colour ... FAILED
...
---- runtime::tests::a_receiver_powers_on_a_laser_and_copies_its_colour stdout ----
thread 'runtime::tests::a_receiver_powers_on_a_laser_and_copies_its_colour' panicked at src/runtime.rs:260:5:
not yet implemented: receiver state
...
test result: FAILED. 26 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out
```

(`ports/_shared/evidence/laserbridges-tmp/sdk-test-receiver-red.log`; the parser followed the same
cycle: 35 passed / 1 failed, `not yet implemented: state property`, in
`sdk-test-stateparse-red.log`.) Then the implementation, then the glue, then `37 passed` — 28
`runtime::` and 9 `tests::`.

### Upstream defect fixed — the dead "is it looking into me?" test

`IReceiverBlock.updateState` computes the neighbour's direction from **its own** state:

```java
Direction laserFacing = state.getValue(ILaserGeneratingBlock.FACING);   // receiver's own state
AttachFace laserFace  = state.getValue(ILaserGeneratingBlock.FACE);
...
if (!(laserDir == dir.getOpposite())) { shouldPowered = true; color = laserColor; }
```

`laserDir` is therefore always `dir`, and `dir == dir.getOpposite()` is never true: the guard can
never fail, so every laser block or powered source in front powers the receiver — including one at
the *end* of a beam, which is the mod's main use. Read literally from `checkState` instead, the guard
would make an end-of-beam receiver dead. The port implements the effective rule ("a laser or a
powered source in front powers it and its colour is copied") and does **not** reproduce the dead
comparison; nothing a user can notice changes, and the receiver at the end of a beam works.
Worth an upstream report: the test reads `state` where it means `checkState`.

### Content

Carried byte-for-byte from the jar (`unzip -p`, same command shape as 0.2.0):

- `assets/laserbridges/blockstates/{laser_attached_receiver,laser_attached_receiver_fence}.json`
- `assets/laserbridges/models/block/{…,_powered} × 2 receivers` (4 files), `models/item/ × 2`
- `assets/laserbridges/textures/block/laser_attached_receiver{,_fence,_powered}.png`
- the jar's own two `lang` keys and the two `loot_table` entries (the pool-level `conditions` array
  converted to the 26.3 pool-level `condition` object, as 0.2.0 did for the sources)
- `mineable/pickaxe` now names the jar's six blocks (four sources + two receivers)

Written for 26.3 by `tools/gen_content.py` (audit trail for the `Block.box` tables):
`data/laserbridges/horizon/block/{laser_attached_receiver,laser_attached_receiver_fence}.json` —
the same `face`/`facing`/`powered`/`color` state set, 24 shape tuples each from
`BridgeAttachedReceiverBlock`/`FenceAttachedReceiverBlock` (a powered receiver adds the beam plane),
`strength(0.8f)`, `noOcclusion`, and **`state_light`** on `powered`; four recipes from
`CommonClass.init` (two shaped `IGI/IEI/III` and `III/IEG/III` with iron + glass + redstone, and the
two 1:1 conversion recipes); both items in the mod's creative tab; `assets/…/items/*.json` for the
26.3 item-model definition; the `uses.json` grew from 183 to 212 entries (two more hooks —
`block_signal`, `is_signal_source` — and the handles/methods they pull in).

### The powered-light deviation is closed

`ModRegistry` gives the attached sources and both receivers `lightLevel(state -> POWERED ? 10 : 0)`;
0.2.0 declared a constant `light: 10` because the schema was taken to be per-block. At this pin the
content schema has `state_light` (`properties` + `values` + `default`), so all three blocks now emit
10 only while powered and 0 otherwise. The in-game leg reads `ctl light`: unpowered `0`, powered
`a`. This removes deviation 2 from the 0.2.0 list.

### In-game leg (quoted from `e2e.log`, 0.2.1)

```
== H: a receiver on the beam powers and drives a lamp ==
block 19,100,24: ... "state":"laserbridges:laser_attached_receiver[color=0,face=wall,facing=west,powered=false]"
block 19,101,24: ... "state":"minecraft:redstone_lamp[lit=false]"
the unpowered receiver gives nothing: the lamp above it is unlit
block 19,100,24: ... "state":"laserbridges:laser_attached_receiver[color=0,face=wall,facing=west,powered=true]"
block 18,100,24: ... "state":"laserbridges:laser_block_powered[color=0,face=wall,facing=east]"
block 19,101,24: ... "state":"minecraft:redstone_lamp[lit=true]"
block 20,100,24: ... "state":"minecraft:redstone_lamp[lit=false]"
the receiver powered on the beam, its colour is 0, and the lamp above it lights while the one behind stays dark
unpowering the source clears the beam and the receiver's signal with it
block 18,100,24: ... "state":"laserbridges:laser_block_powered[color=5,face=wall,facing=east]"
block 19,100,24: ... "state":"laserbridges:laser_attached_receiver[color=5,face=wall,facing=west,powered=true]"
the receiver copied the beam's colour 5
block 19,100,24: air
block 19,101,24: ... "state":"minecraft:redstone_lamp[lit=false]"
breaking the receiver drops its signal
== I: a receiver facing away from the beam stays off ==
the beam is one cell behind the receiver, which looks the other way and stays unpowered
== J: a powered source in front powers a receiver, an unpowered one does not ==
an unpowered source in front does not power the receiver
the same source powered by redstone does
with the redstone gone the pair stays on: the receiver signals the source it looks into
breaking the receiver breaks the loop: the source loses its 15 and the lamp above it goes out
== K: the receivers and the attached sources emit light only while powered ==
unpowered attached source light: 0
powered attached source light: a
powered receiver light: a
unpowered receiver light: 0
== L: the receivers take their facing from the click and refuse a thin support ==
receiver place reply: {"block":{"block":"laserbridges:laser_attached_receiver",..."state":"...[color=0,face=wall,facing=west,powered=false]"}...}
the fence refused the mount and kept the stack; the stone accepted it
== M: every carried recipe crafts in a crafting table ==
  result: {"count": 1, "item": "laserbridges:laser_attached_receiver", "slot": 200}
  crafted 1 laserbridges:laser_attached_receiver, ingredients consumed
  ... (12 recipes)
PASS: all 12 carried recipes craft in a crafting table, each makes its declared item and consumes its ingredients
== N: a broken receiver and source drop themselves ==
receiver drops on the ground: 1, in the inventory: 1
source drops on the ground: 1, in the inventory: 0
168 declared states accepted (168: 8 blocks x face x facing), and a colour-15 source binds
PASS: ... EXIT:0
```

Controls inside the run: the unpowered receiver, the receiver facing away, the unpowered source in
front, the lamp on the excluded face, the cleared beam and the fence click each hold the value the
powered run inverts; `SELFTEST=2` flips the receiver's `powered` assertion and the run fails at it
(`FAIL: expected powered=false at 19,100,24, got ...powered=true`, `EXIT:1`); `SELFTEST=1` fails at
the beam assertion; `CONTROL=1` runs the receiver leg on the unmodded Nightly game and fails at the
receiver id (`got {"block":"minecraft:stone"}`), so the leg is not vacuous.

**A latch, recorded not "fixed":** a receiver gives 15 on every face but the one it hangs from, so a
receiver looking into a *source* also signals that source; with the redstone removed the pair holds
itself on (see phase J). That is upstream's own `getSignal` (the exclusion is the hanging face, not
the front), so the port keeps it and the leg proves the way out: breaking the receiver drops the
source and the lamp. Not a defect — a redstone design choice; noted here so it is not "reported" as
one.

### Coverage (parity rule, ruling 2026-11-20)

| Family (from the jar) | State | Where it is proved |
|---|---|---|
| four sources: place, state set, redstone control, beam geometry, walkability | tested | phases A–E, F (states) |
| two laser blocks (unbreakable, light 10, collider) | tested | phases B, D, G (no item), F (states) |
| **two receivers: blocks, states, shapes, models** | tested | phases F (168 states), H–L |
| **receiver redstone output (`block_signal` / `is_signal_source`)** | tested | phase H (lamp above lit / lamp behind dark), J |
| **receiver colour copy** | tested | phase H (colour 5) |
| **receiver light (powered only)** | tested | phase K (`ctl light` 0 / a) |
| **attached sources' light (powered only)** | tested | phase K (0 / a) |
| **attached support requirement (sources and receivers)** | tested | phase L (fence refused, stone accepted, stack kept) |
| **all twelve carried recipes** | tested | phase M (`tools/craft_recipes.py`, each output + count + consumed ingredients) |
| **loot tables (sources and receivers)** | tested | phase N (survival break, drops on the ground) |
| `mineable/pickaxe` tag | not testable, inert | carried as the jar lists it (four sources + two receivers); no block sets `requires_tool`, so membership changes nothing the game exposes — `check_261_dialect.py` sees every block in the tag |
| sounds `laserbridges:on`/`off` | not ported | `native_play_sound` takes vanilla sound events at this pin (gap 3) |
| dyeing a source/receiver (`useItemOn` with a dye) | not ported | gap 2: the state would change but the model tint has no client equivalent here |
| dispenser dye behaviour | not ported | gap 4 (a dispenser hook nothing in the tier needs yet) |
| `config/laserbridges.json` (`max_length`, cut-through list) | not ported | gap 5: constants in `runtime.rs`, unchanged by this slice |
| a colour change mid-line strands the far cells | tested (kept) | phase H changes colour with the beam off and says why |
| receiver→source latch | tested (kept) | phase J |
| piston-moved source | not ported | gap 7 (no piston hook; upstream has none either) |
| worldgen / biomes / structures / commands / multiblocks / machines | n/a | the mod has none; class `content`, no block entity |

### Ladder and artifact (0.2.1, pin `0a875da`)

Every step through `ports/tools/build-slot.sh` (`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`), no bare
`cargo`, no `--target-dir`.

| Step | Command | Result |
|---|---|---|
| repin | `cargo horizon-mod update` | `Laser Bridges & Doors, from 0b14f8a to 0a875da: none of the 183 items it uses changed.` / `Pinned to Horizon 0a875da…`; `.horizon/sdk` refetched (`SDK 0a875da (build 0a875da…)`) |
| baseline at the pin | `cargo test --offline` | 23 passed (`sdk-test-baseline.log`) |
| runtime first (red) | the same, with the receiver helpers as `todo!()` | **26 passed, 9 failed**, every failure `not yet implemented: receiver …` (`sdk-test-receiver-red.log`) |
| tests | `cargo test --offline` | **37 passed, 0 failed** (28 `runtime::`, 9 `tests::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** — `sdk-clippy.log` |
| format | `cargo fmt --check` | `FMT_EXIT:0` — `sdk-fmt.log` |
| build 1 | `rm -f Cargo.lock && cargo horizon-mod build` | `SDK 0a875da…`, package built (`sdk-build.log`) |
| build 2 | the same again, lock removed | `CMP:identical` (`sdk-build-second.log`) |
| registry check | `cargo horizon-mod check laserbridges.hmod` | `laserbridges 0.2.1 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **77,460 B**, SHA-256 `192d58ff…3656`, unsigned (`sdk-check.log`) |
| fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 (`sdk-fits.log`) |
| fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 (`sdk-fits-other.log`) |
| gate | `python3 ports/tools/verify-sdk-package.py laserbridges` | **1/1** |
| game | `launcher-game.sh build laserbridges` | `Compiling laserbridges (1 of 1)` |
| in game | one leased `tools/e2e.sh` + `CONTROL=1` + `SELFTEST=1` + `SELFTEST=2` | `PASS`+`EXIT:0`, `FAIL`+`EXIT:1` ×3 (`laserbridges-e2e/`) |
| log gate | `check-e2e-logs.py laserbridges-e2e --strict` | `logs=4 ok=4`, exit 0 |

Artifact: **77,460 B**, SHA-256 `192d58ff59c5fa038cefa8dcdc0dcfccb0ad71189ce376fd4e29af707b983656`,
**84 entries**, no packed `Cargo.lock` (`sdk-entries.txt`, `sdk-sha256.txt`; the delivered file and
the game build were made from the same manifest and sources).

`check_261_dialect.py` reports 12 offenders, all of them the two *unverified* shapes from HANDOVER
§9d item 13 (`bonus_rolls`, `rolls: 1.0`) on the six block loot tables — the shipped form is the one
the in-game loot leg (phase N) proves works, so it is left alone rather than churned; the three
game-confirmed-fatal shapes are absent (recipe ingredients are bare strings, the loot condition is a
pool-level object, every `requires_tool:false` block is in `mineable/pickaxe`).

### Gaps after slice 2

1. Dyeing a source or a receiver (`useItemOn` with a dye, `Stats.ITEM_USED`, and the dispenser's dye
   behaviour): the `color` state is settable by `setblock`, but no interaction hook sets it and the
   carried models' `tintindex: 0` has no client tint handler here.
2. `laserbridges:on`/`off` playback: carried, never played (the pin's `native_play_sound` takes
   vanilla sound events).
3. `config/laserbridges.json`: `max_length` (15) and `blocks_cut_through_by_lasers` are constants.
4. A colour change under a live beam strands the far cells (upstream's exact-state test, kept), and a
   receiver facing a source latches (upstream's `getSignal`, kept).
5. Piston-moved sources are not hooked.
6. **Slice 3 candidate:** dyeing (the only remaining *content* interaction) with the tint recorded as
   a client-side gap, or the config file — see the Next-slice line in the report.

### Changelog for 0.2.1

> 0.2.1 adds the two receivers (`laser_attached_receiver`, `laser_attached_receiver_fence`) that make
> a beam a redstone signal source: they power on a laser block or powered source in front of them,
> copy its colour, emit light only while powered, answer 15 to their neighbours (0 on the face they
> hang from; the direct signal only on the face they look into) and are attached blocks with the
> sturdy-support rule. All twelve carried recipes are crafted in a real crafting table by the in-game
> leg. Repointed to nightly `0a875da` (mixins `e59da90…`, unchanged); the attached sources' and
> receivers' light is now per-state (`state_light`), closing the constant-light deviation. 84
> entries, 77,460 B, SHA-256 `192d58ff…3656`; `check` and both `fits` runs exit 0.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices, rust-port

## Slice 1 boundary

In scope:

- the four sources `laser_source_block`, `laser_source_fullblock`,
  `laser_fence_source_block`, `laser_fence_source_fullblock` with the upstream state set
  (`face` × `facing` × `powered` × `color`) and their upstream outlines/powered outlines;
- the two beams `laser_block_powered` and `laser_fence_powered` (unbreakable, light 10, thin
  collidable plates) that the sources write and clear;
- `getStateForPlacement` (clicked face → `face`/`facing`, plus the attached variants' sturdy-support
  requirement), `generateBridge`'s line (length, cut-through blocks, obstruction stop, shrinking
  tail clear), the powered state, and clearing the beam when the source is broken or replaced;
- carried content: models, blockstates, textures, sounds, lang, loot, the pickaxe tag, the six
  container recipes and a creative tab.

Out of scope (see Gaps): the two receivers, dyeing, the on/off sounds at runtime, the dispenser dye
behaviour, the config file.

## Behaviour (from `FINDINGS.md`)

`generateBridge` walks `i = 1 .. max_length-1` from the source along its face/facing with
`length = (max_length / 15) * power` (truncated), cuts through air/grass/fern/dead-bush, stops at
anything else, places the laser block while `i < length` and clears its own kind beyond it. A full
signal fills 14 cells; a signal of 1 fills none. The wall-mounted bridge source lays a horizontal
plate whose top is exactly half a block above the course (walkable); a floor/ceiling source lays a
thin vertical wall (the "door"/"fence").

## Hooks (0.1.0, retired `.hnmod`) — HISTORICAL, see the SDK table above

| Hook (API) | Helper | What it does |
|---|---|---|
| `state_for_placement` (1) | `placement.rs` | `getStateForPlacement`: the clicked face picks floor/ceiling/wall and the facing; an attached source answers `None` without a sturdy support |
| `neighbor_changed` (6) | `neighbor.rs` | a source's redstone input may have moved → queue it |
| `block_changed` (7) | `changed.rs` | a placed/replaced source → evaluate; a removed or replaced one → clear the beam its old state left |
| `level_tick_post` (23) | `tick_post.rs` | read `native_neighbor_signal` (96), set `powered`, write the line through `native_set_block` (24) |

Why the tick: `native_neighbor_changed`/`native_block_changed` are `&self` on `GameLevel` and cannot
write; see `FINDINGS.md` §"Why the port defers to `level_tick_post`".

## Runtime and glue split (0.1.0) — HISTORICAL for the glue; `src/runtime.rs` is unchanged

- `mixins/laserbridges/runtime.rs` — the pure crate `lib` and the injected
  `crate::game::laserbridges`: `Face`, `Facing`, `Beam`, `Clicked`, `Cell`, `Write`,
  `length()`, `Beam::offset()`, `plan()`, `placement()`, `horizontal()`. 15 unit tests.
- `mixins/laserbridges/support.rs` — the injected `crate::game::laserbridges_state`: the source
  table, the state parser, the per-tick job queue, the world reads and the writes.

## Deliberate deviations (all recorded)

1. **One tick of latency** between a redstone change and the beam (the pin's neighbour hooks cannot
   write); the second pass writes nothing, so the beam settles after one tick.
2. ~~Attached sources glow at light 10 while unpowered~~ — **closed in 0.2.1**: the pin's content
   schema has `state_light`, so the attached sources and the receivers emit 10 only while powered
   (phase K of the in-game leg reads `ctl light` 0 / a).
3. **Colour is state-only**: no dye hook, so every beam is colour 0 and the carried models' tint is
   not applied (the client tint handler has no equivalent here).
4. **Sounds carried but not played** (the pin plays vanilla sound events only).
5. **Cut blocks drop** through `native_machine_destroy_block` (upstream's `destroyBlock(pos, true)`);
   a cut *air* cell is a no-op, as upstream's call is.
6. **`max_length` and the cut-through list are constants** (15 and the eight ids), not a config
   file.
7. A source moved by a piston is not hooked (no piston hook is used); its beam would stay.

## Gaps

> The list below is the **0.2.0 state**, kept as history. Slice 2 closed item 1 and item 3's light
> half; the live gap list is [Gaps after slice 2](#gaps-after-slice-2).

1. ~~The two receivers (`laser_attached_receiver`, `laser_attached_receiver_fence`): blocks, models,
   textures, lang, recipes, and their redstone output (`is_signal_source` + `block_signal` + the
   colour copy from the laser in front).~~ — **shipped in 0.2.1**, see
   [Slice 2](#slice-2--the-receivers-021-2026-11-21).
2. Dyeing a source (`useItemOn` with a dye, and the dispenser's dye behaviour).
3. `laserbridges:on`/`off` playback.
4. `config/laserbridges.json` (`max_length`, `blocks_cut_through_by_lasers`).
5. A colour change mid-line strands the far cells (upstream's exact-state test, kept deliberately).
6. Beam blocks are unbreakable like upstream (bedrock template), so there is no drop and no
   break-progress path to test.

## Embedded content — one `.hnmod` deliverable (2026-11-10) — HISTORICAL, retired format

The content half now travels **inside** the native bundle (the AE2 pattern, proved
on `charginggadgets`): the built `content/laserbridges.hmod` is carried at
`mixins/laserbridges/data/laserbridges-content.hmod`; `embedded_content.rs` writes it to
`.horizon-cache/` at start and appends it to the dev list from one `content_root`
HEAD injection (`cancellable = true`; bindings `root`, `enabled`, `dev`). One
installed `.hnmod` installs both halves, so the split content+code blocker is
gone. `tools/package.py` packs `data/` and refuses a copy that differs from
`content/laserbridges.hmod`.

Verification (2026-11-10, pin `1d63e0a`, Minecraft 26.3):
- **ladder**: `cargo fmt --check`, `clippy --all-targets -- -D warnings` (0
  warnings), `cargo test` **15 passed / 0 failed**, `horizon.py stage / check / test /
  build` all `cargo_exit: 0` (embedded tests included).
- **artifacts**: `content/laserbridges.hmod` **42,865 B**, SHA-256 `e5a1f61f0b19d3e8497637bffb9d1b735bff042342cb17a43e7dbf981d3869fb`,
  **56** entries; `laserbridges-0.1.0.hnmod` **46,093 B**, SHA-256 `0ebbd437df732eba3979c985782259ece89a5c8dbc38f2fe179b007d0c0b2867`, **10**
  entries (rebuilt locally from these sources with `tools/package.py`; a second
  run is byte-identical and `horizon-mod check --kind native-mod` passes);
  `mixins/laserbridges/data/laserbridges-content.hmod` is byte-identical to the built `.hmod`.
- **e2e with no `--dev-mod`** (`EMBEDDED=1`): `PASS` + `EXIT:0`; the run printed
  `embedded content: 42865 B, identical to content/laserbridges.hmod`.
- **control** (`e2e-control.log`, `.horizon-cache` made unusable): the sample id
  is unknown, `PASS` + `EXIT:0` - the content really comes from the bundle.
- **fail leg** (`SELFTEST=1`): `FAIL` + `EXIT:1`.
- `check-e2e-logs.py laserbridges --strict` green (all logs end `EXIT:<n>`);
  `upstream_ready.py laserbridges` -> `READY`, exit 0.

The compile ladder ran on the second build machine; the in-game legs ran locally.

Skills: superpowers/using-superpowers, brainstorming, writing-plans, test-driven-development, systematic-debugging, verification-before-completion; BeautifulRust/rust-best-practices

## Verification (0.1.0, retired `.hnmod`) — HISTORICAL

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`** (game.rs
`64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f`), mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6). Every build/test/e2e step ran through
`ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`; no bare `cargo`, no `--target-dir`.
Evidence: `ports/_shared/evidence/laserbridges/{runtime-test-red,runtime-test-green,unit,check,test,build,e2e,selftest,hmod-load}.log`.

### Ladder

| Step | Command (via the slot runner) | Result |
|---|---|---|
| runtime TDD (red) | `cargo test --offline` with `todo!()` stubs | **4 passed, 11 failed** — every failure `not yet implemented: beam plan / source placement / horizontal facing` (`runtime-test-red.log`) |
| runtime TDD (green) | `cargo test --offline` | **15 passed**, 0 failed (`runtime-test-green.log`) |
| fmt / clippy / tests | `cargo fmt --all --check && cargo clippy --all-targets --locked --offline -- -D warnings && cargo test --locked --offline` | exit 0, **0 warnings**, 15 passed (`unit.log`) |
| stage + check | `python3 tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** (`check.log`) |
| embedded tests | `python3 tools/horizon.py test` | `test result: ok. 15 passed; 0 failed; 1240 filtered` (`test.log`) |
| fast build | `python3 tools/horizon.py build` | `cargo_exit: 0` (`build.log`) |
| e2e | `build-slot.sh e2e --job lb-1d63e0a --port ports/laserbridges -- tools/e2e.sh` | **PASS**, `EXIT:0` (`e2e.log`) |
| fail-capability | `SELFTEST=1 tools/e2e.sh` | **EXIT:1** at the flipped full-power assertion (`selftest.log`) |
| content package | `cargo-horizon-mod horizon-mod build` + `check` | 42,865 B, 56 entries, no code (`hmod` below) |
| load-check | `tools/hmod-load.sh` (shared unmodded game, `--dev-mod`) | **PASS**, `EXIT:0`, with the no-`--dev-mod` negative control (`hmod-load.log`) |
| native package | `python3 tools/package.py` | 11,042 B, 8 entries |

### In-game scenario (quoted from `e2e.log`, 0.1.0 build) — HISTORICAL

```
place reply: {"block":{"block":"laserbridges:laser_source_block","pos":[2,101,-2],
  "state":"laserbridges:laser_source_block[color=0,face=wall,facing=west,powered=false]"},"changed":true,"ok":true}
block 4,100,0: ... "state":"laserbridges:laser_source_fullblock[color=0,face=wall,facing=east,powered=true]"
block 5,100,0: ... "state":"laserbridges:laser_block_powered[color=0,face=wall,facing=east]"
block 18,100,0: ... "state":"laserbridges:laser_block_powered[color=0,face=wall,facing=east]"
block 19,100,0: air
feet y at beam: 100.5
the beam is walkable: feet at 100.5 on the plate whose top is 100.5
block 5,100,0: air
block 10,100,0: air
block 18,100,0: air
feet y at unpowered: 100.0
unpowered the beam is gone: feet at 100.0 on the floor at 100.0
block 18,100,0: ... "state":"laserbridges:laser_block_powered[color=0,face=wall,facing=east]"
re-powering builds the beam again
the plate beside the source: {"block":"minecraft:light_weighted_pressure_plate","state":"... [power=7]"}
block 10,100,6: ... "state":"laserbridges:laser_block_powered[color=0,face=wall,facing=east]"
block 11,100,6: air
a signal of 7 built 6 cells (5..10) and cleared the rest of the line
block 0,101,20: ... "state":"laserbridges:laser_fence_powered[color=0,face=floor,facing=north]"
block 0,114,20: ... "state":"laserbridges:laser_fence_powered[color=0,face=floor,facing=north]"
block 0,115,20: air
the door stands 14 cells tall (101..114)
block 0,101,20: air
block 0,114,20: air
unpowered the door opens (every cell is air again)
block 4,100,-7: ... "state":"laserbridges:laser_block_powered[face=wall,facing=north]"  (a north-facing source)
block 5,100,-6: air
a north-facing source leaves the +x line empty
block 5,100,10: air / block 10,100,10: air
an unpowered source builds nothing
block 8,100,14: "laserbridges:laser_block_powered" / block 9,100,14: "minecraft:stone" / block 10,100,14: air
the stone at x=9 stopped the beam; the cells past it stay empty
block 5,100,14: air / block 8,100,14: air
breaking the source cleared its beam
120 declared states accepted, and a colour-15 source binds
give of a beam block: {"error":"unknown item laserbridges:laser_block_powered"}
PASS: … EXIT:0
```

The controls are inside the run: the wrong-facing source, the unpowered source, the stone
obstruction and the broken source each keep a line empty that the powered run fills, and the
player's feet drop from 100.5 to 100.0 when the beam goes. `SELFTEST=1` flips the full-power
assertion and the run fails at it (`EXIT:1`), so the scenario can fail.

### Artifacts

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/laserbridges.hmod` | 42,865 B | `e5a1f61f0b19d3e8497637bffb9d1b735bff042342cb17a43e7dbf981d3869fb` | 56 |
| `target/laserbridges-0.1.0.hnmod` | 11,042 B | `ddb228a5a7ab6354e4ebd25bbf481e762f03d5fa7cd08675fb7c443d65a51b90` | 8 |

Load-check: the shared unmodded 1d63e0a5 game with `--dev-mod` registers all six blocks, binds
`face`/`facing`/`color`, refuses `color=16` and a `powered` property on a beam block, and gives the
source item; **without** `--dev-mod` the same ids are unknown (`Unknown block type`, `unknown
item`), so the check is not vacuous.

Harness notes: the content package was built with the pin's `cargo-horizon-mod` binary that the
`ironfurnaces` job left at `/tmp/opencode/ironfurnaces-work/cargo-horizon-mod` (same pin, so the
same content schema); `horizon-mod check` reports `laserbridges 6.3 (mod, API 0.2, both) … no code`.
A `setblock` sent through `ctl command` lands one server tick later, so the scenario waits before
reading a block back; a redstone wire cannot power these sources (see `FINDINGS.md`), so the
signal-strength case uses a weighted pressure plate.

### Upstream source read

- Repository, branch and commit as above (`gradle.properties` `version=6.3`, matching the jar).
- Files read: `Constants.java`, `ModRegistry.java`, `LaserBridgesConfig.java`, `CommonClass.java`,
  `neoforge/src/main/java/com/mars/laserbridges/LaserBridges.java`,
  `blocks/ILaserGeneratingBlock.java`, `blocks/source/{AbstractSourceBlock,AbstractAttachedSourceBlock,
  ISourceBlock,BridgeSourceBlock,FenceSourceBlock,BridgeAttachedSourceBlock,FenceAttachedSourceBlock}.java`,
  `blocks/laser/{AbstractLaserBlock,LaserBridgeBlock,LaserFenceBlock}.java`,
  `blocks/receiver/{IReceiverBlock,AbstractAttachedReceiverBlock}.java` (for the gap list), and the
  jar's `blockstates/`, `models/`, `lang/`, `sounds.json`, `loot_table/`, `mineable/pickaxe` tag.
- Behaviours extracted: the beam walk, length mapping, cut-through list, obstruction stop, tail
  clear, the powered/lifecycle rules, the shapes (1/16-unit tables), the recipe grids, the attached
  sources' support requirement, the receiver logic (deferred).
- Where the rewrite differs: the tick deferral, the constant light, the colour-only state, the
  sound/particle/tint omissions — see Deviations and Gaps.

## Mode B rebuild (2026-11-09) — artifact made provably current (HISTORICAL, retired format)

Pin unchanged; ladder + repackage + both e2e legs re-run in one lease
(`build-slot.sh e2e --job MB-laserbridges`, `CARGO_INCREMENTAL=0`).

| Step | Result |
|---|---|
| fmt / clippy `-D warnings` | exit 0, **0 warnings** |
| `cargo test --locked --offline` | **15 passed**, 0 failed |
| `horizon.py stage/check/test` | `cargo_exit: 0`, **15 passed** embedded |
| `.hmod` rebuild | **42,865 B**, `e5a1f61f…69fb`, **56 entries** |
| `.hnmod` repack | **11,042 B**, `ddb228a5…1b90`, **8 entries** |
| default / selftest | **`PASS`+`EXIT:0`** / **`FAIL`+`EXIT:1`** |

`check-e2e-logs.py laserbridges --strict` → `logs=2 ok=2`, exit 0.
`upstream_ready.py laserbridges` reports only the campaign-level split-port
blocker (one file per registry entry); no artifact, pin or evidence blocker.
The e2e script now also emits `EXIT:<status>` from its cleanup trap.

### Mode B + AE2 embedding (2026-11-10) — one listable artifact, re-verified

Pin unchanged (`1d63e0a5…` / `5a64af93…`, schema 6, `game.rs`
`64813cf4…f80f`). The session rebuilt the port from its current sources and
applied the **AE2 embedding pattern** so one `.hnmod` installs the content and
the code (the registry keeps one file per project): `mixins/laserbridges/data/laserbridges-content.hmod`
is the built `content/laserbridges.hmod` byte for byte, `mixins/laserbridges/embedded_content.rs`
compiles it in with `native_data!`, and the `embedded-content` `[[injection]]`
(`target = "horizon_client::game::mods::content_root"`, `kind = "HEAD"`,
`cancellable = true`, bindings `root`/`enabled`/`dev`) writes it to the game's
`.horizon-cache` and appends it to the dev-mod list. `tools/package.py` refuses
an embedded copy that differs from the built hmod.

All compile/e2e steps ran through `ports/tools/build-slot.sh` with
`CARGO_INCREMENTAL=0`; the ladder is the shared `tools/ladder-embed.sh`
(hmod build → embed → hnmod → fmt → clippy → test → stage/check/test/build),
evidence `ports/_shared/evidence/laserbridges/ladder-*`.

| Step | Result |
|---|---|
| `cargo fmt --all --check` / clippy `-D warnings` | exit 0, **0 warnings** |
| `cargo test --locked --offline` | **15 passed**, 0 failed |
| `horizon.py stage/check/test/build` | `cargo_exit: 0` in every step |
| `content/laserbridges.hmod` | **42,865 B**, `e5a1f61f…69fb`, **56 entries**, "no code: content only" |
| `laserbridges.hnmod` (single artifact, carries the content) | **46,093 B**, `0ebbd437…2867`, **10 entries** |
| e2e (with `EMBEDDED=1`, the shipped shape) | default **PASS + `EXIT:0`** / selftest **FAIL + `EXIT:1`** |
| `check-e2e-logs.py laserbridges --strict` | green (`logs=… ok=…`, exit 0) |
| `upstream_ready.py --copyleft laserbridges` | **READY**, exit 0 |

Fail-capability: the selftest/negative legs flip a real assertion and exit
non-zero; the embedded shape is the one the artifact ships (no `--dev-mod`), and
the e2e checks the bundle wrote its content out (or the loader reports
`content from 1 mod(s)` when the game consumes the file).

## Review repairs (PR #610, 2026-11-11) — HISTORICAL, retired format

Review: `ports/_shared/horizon-mods/reviews/pr-610-laserbridges.md` (CHANGES_REQUESTED, 2 items + addendum).

| Reviewer's item | This job |
|---|---|
| 1. artifact not staged in the registry | **orchestrator** (no write access) |
| 2. relative link and prose ending mid-sentence ("shrinking", "no drop and no") | link was already absolute; the **description is rewritten** with the four sources, the two beams, placement, the bridge generator, and the real gap list (receivers, dyeing, sounds, config, the deliberate colour-change stranding, no drop/break sound) |
| addendum: neither archive includes LICENSE/NOTICE | **fixed** by the packager (`mixins/laserbridges/LICENSE`) |
| the PR/release prose still advertises the old `c4e2c5c7…` digest | **orchestrator** (the file is the new `03342186471c…`) |

Artifact: `laserbridges-0.1.0.hnmod` **47,693 B** `03342186471cb9f70eba549fdc25ad15cd6dc362583edab3b6b44a69059a6753`
(content `laserbridges.hmod`).
