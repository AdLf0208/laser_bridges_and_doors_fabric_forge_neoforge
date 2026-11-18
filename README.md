# Laser Bridges & Doors [Fabric/Forge/NeoForge] — a Horizon port

**Laser Bridges & Doors [Fabric/Forge/NeoForge]** is a Minecraft mod. Adds Laser Source Block which when powered creates laser platforms and doors. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [Mars-The-Planet/Laser-Bridges-And-Doors](https://github.com/Mars-The-Planet/Laser-Bridges-And-Doors) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Laser Bridges & Doors [Fabric/Forge/NeoForge] ([source](https://github.com/Mars-The-Planet/Laser-Bridges-And-Doors)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `laserbridges.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- A Horizon port of laserbridges.

## Not yet ported

- The two receivers (`laser_attached_receiver`, `laser_attached_receiver_fence`): blocks, models,
- Dyeing a source (upstream's `useItemOn` with a dye, and the dispenser's dye behaviour).
- The `laserbridges:on`/`off` playback (carried sounds, never played).
- `config/laserbridges.json` (`max_length`, `blocks_cut_through_by_lasers`): both are constants.
- A colour change mid-line strands the far cells (upstream's exact-state test, kept deliberately).
- The beams are unbreakable like upstream (bedrock template), so there is no drop or
- **No 0.2.0 in-game run.** `horizon-launcher` is not installed here, so no game carrying this
- The support refusal now fires at the use moment rather than inside `getStateForPlacement`

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `laserbridges.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Laser Bridges & Doors [Fabric/Forge/NeoForge] was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/Mars-The-Planet/Laser-Bridges-And-Doors

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run laserbridges` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/laserbridges/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
