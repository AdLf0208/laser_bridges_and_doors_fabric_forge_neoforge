# Laser Bridges & Doors for Horizon

Laser Bridges & Doors 6.3 (`Mars-The-Planet/Laser-Bridges-And-Doors`, **MIT**) rewritten for Horizon
as one flat SDK package (**mod API 0.2**, `.hmod`). A redstone-powered laser source fills the line in
front of it with a collidable laser block: a wall-mounted *bridge* source lays a walkable plate, a
floor or ceiling source lays a thin vertical wall (the "door"/"fence"). Power off, and the beam is
gone.

**Local-only** until the maintainer approves publishing (ARCHITECTURE.md §8). MIT allows
derivatives; the approval is still required.

- Implemented: the four sources, the two beams, redstone control, the beam geometry and its
  collision, and the attached sources' sturdy-support requirement. Receivers, dyeing, sounds and the
  config file are the recorded gaps — see [notes.md](notes.md).
- Behaviour analysis: [FINDINGS.md](FINDINGS.md). Jar provenance and the carry list:
  [`../_shared/carry-work/laserbridges/SOURCE.md`](../_shared/carry-work/laserbridges/SOURCE.md);
  licence provenance: [LICENSE-NOTICE.md](LICENSE-NOTICE.md).

## Package layout

```text
horizon-mod.toml        The only manifest and identity (api 0.2, side both, [native] pins)
Cargo.toml              One crate; its only dependency is horizon-mod-api from .horizon/sdk/
src/lib.rs              The one `impl Hooks`: the queue, the signal read, the writes, the use check
src/runtime.rs          The pure beam model and its 15 unit tests (byte-identical to 0.1.0)
assets/laserbridges/    Carried upstream models, blockstates, textures, sounds, lang
data/laserbridges/      The 26.3 block declarations, recipes, loot and creative tab
tools/
  gen_content.py        Emits the block declarations and recipes (the upstream shape tables)
  patch_content.py      Patches the carried loot/lang/items/tags for 26.3
  e2e.sh                The in-game scenario (fail-capable, SELFTEST=1 flips it)
```

## Build and verify

```sh
# the package's own tests and lints (the SDK is fetched into .horizon/sdk/ on first use)
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check                     # NOT --all: that formats the fetched SDK

# the package
rm -f Cargo.lock                      # a packed lock changes the artifact
cargo horizon-mod build               # -> target/horizon-mod/laserbridges.hmod
cp target/horizon-mod/laserbridges.hmod ./laserbridges.hmod
cargo horizon-mod check laserbridges.hmod
cargo horizon-mod fits laserbridges.hmod --build nightly

# the content sources (only when changing them)
python3 tools/gen_content.py && python3 tools/patch_content.py

# the game: the LAUNCHER builds the package into a game (no launcher in this environment yet)
horizon-launcher native-build . --instance <instance> --trust-native-code
GAME=<instance>/horizon-client tools/e2e.sh
```

Every build, test and lint in this campaign runs through `ports/tools/build-slot.sh`
(`run --job laserbridges-sdk --port ports/laserbridges --step <step> -- bash -c 'cd ports/laserbridges && …'`),
never a bare `cargo` outside a lease. The verification results and their logs are in
[notes.md](notes.md) and `ports/_shared/evidence/laserbridges/`.

## Licence

The upstream mod and this port are MIT. [`LICENSE`](LICENSE) is the MIT text; the carried assets,
models, blockstates, textures, sounds and loot tables are upstream's own files, and
[LICENSE-NOTICE.md](LICENSE-NOTICE.md) records where the licence and the copyright were read.
