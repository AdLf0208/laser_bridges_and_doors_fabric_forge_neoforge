# Validation

Validated on Linux with Rust 1.99.1 against **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20) and horizon-mixins
`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, through `ports/tools/build-slot.sh`
(`BUILD_LOCAL=1`, `CARGO_INCREMENTAL=0`, no bare `cargo`, no `--target-dir`).

The sections below are for 0.2.1 (slice 2, the receivers). The 0.1.0/0.2.0 rows are in
[notes.md](notes.md) and their logs stay under `ports/_shared/evidence/laserbridges/`.

## 0.2.1 — ladder

| Check | Command | Result |
|---|---|---|
| repin | `cargo horizon-mod update` | pinned to `0a875da`, `.horizon/sdk` refetched |
| baseline at the pin | `cargo test --offline` | 23 passed (`sdk-test-baseline.log`) |
| runtime first (red) | `cargo test --offline` with the receiver helpers as `todo!()` | **26 passed, 9 failed**, every failure `not yet implemented: receiver …` (`sdk-test-receiver-red.log`) |
| parser red | the same with `state_property` stubbed | **35 passed, 1 failed**, `not yet implemented: state property` (`sdk-test-stateparse-red.log`) |
| tests | `cargo test --offline` | **37 passed**, 0 failed (28 `runtime::` + 9 `tests::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** — `sdk-clippy.log` |
| format | `cargo fmt --check` (not `--all`) | `FMT_EXIT:0` — `sdk-fmt.log` |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` ×2 | `CMP:identical` (`sdk-build.log`, `sdk-build-second.log`) |
| registry check | `cargo horizon-mod check laserbridges.hmod` | exit 0, `laserbridges 0.2.1 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 77,460 B, SHA-256 `192d58ff…3656` (`sdk-check.log`) |
| fits the pin | `cargo horizon-mod fits … --build 0a875da` | `0a875da  made for`, exit 0 (`sdk-fits.log`) |
| fits another build | `cargo horizon-mod fits … --build 0b14f8a` | `0b14f8a  fits`, exit 0 (`sdk-fits-other.log`) |
| gate | `python3 ports/tools/verify-sdk-package.py laserbridges` | **1/1** |
| game build | `ports/tools/launcher-game.sh build laserbridges` | `Compiling laserbridges (1 of 1)` |
| in game | one leased `tools/e2e.sh` | **PASS**, `EXIT:0` (`laserbridges-e2e/e2e.log`) |
| control | `CONTROL=1 tools/e2e.sh` (the unmodded Nightly game) | **FAIL** at the receiver id, `EXIT:1` (`e2e-control.log`) |
| fail-capability | `SELFTEST=1` / `SELFTEST=2` | **FAIL**, `EXIT:1` at the beam / receiver `powered` assertion (`e2e-selftest.log`, `e2e-selftest-receiver.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py laserbridges-e2e --strict` | `logs=4 ok=4`, exit 0 |

Artifact: `laserbridges.hmod` **77,460 B**, SHA-256
`192d58ff59c5fa038cefa8dcdc0dcfccb0ad71189ce376fd4e29af707b983656`, **84 entries**, no packed
`Cargo.lock` (`sdk-entries.txt`, `sdk-sha256.txt`).

## In-game coverage (0.2.1)

Proved by `tools/e2e.sh` phases A–N: the four sources, the two beams and their walkability (A–E),
every declared state over the eight blocks (F, 168), the receivers' power-on, colour copy, signal
faces, light and the latch (H–K), placement and the sturdy-support refusal (L), all twelve carried
recipes in a crafting table with their ingredients consumed (M, `tools/craft_recipes.py`), and the
sources'/receivers' own drops (N). The family→status table is in
[notes.md](notes.md#coverage-parity-rule-ruling-2026-11-20).

## Not run, and why

- **Dyeing, the on/off sounds and the config file**: recorded gaps (notes.md, "Gaps after slice 2"),
  so nothing claims them.
- **A rendered screenshot of the models/tint**: the models' `tintindex: 0` has no client tint handler
  at this pin; the run uses `--offscreen` for the menus and does not claim a visual.
- **`check_261_dialect.py`**: it reports the two *unverified* loot shapes (`bonus_rolls`,
  `rolls: 1.0`) on the six tables; the in-game loot leg (phase N) proves the shipped form works, so
  the form is kept rather than churned (HANDOVER §9d item 13).
