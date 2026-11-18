# Validation

Validated on Linux with Rust 1.97 (toolchain file pins 1.95.0) against Horizon
`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` and horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57` (plan schema 6), through
`ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`.

| Check | Command | Result |
|---|---|---|
| runtime tests (red first) | `cargo test --offline` on `todo!()` stubs | 4 passed, 11 failed with `not yet implemented` |
| runtime tests | `cargo test --locked --offline` | **15 passed**, 0 failed |
| formatting | `cargo fmt --all --check` | clean |
| lint | `cargo clippy --all-targets --locked --offline -- -D warnings` | 0 warnings |
| staged compile | `python3 tools/horizon.py check` | `cargo_exit: 0`, 0 warnings |
| embedded tests | `python3 tools/horizon.py test` | `15 passed; 0 failed; 1240 filtered` |
| fast build | `python3 tools/horizon.py build` | `cargo_exit: 0` |
| in-game scenario | `tools/e2e.sh` through `build-slot.sh e2e` | **PASS**, `EXIT:0` |
| fail-capability | `SELFTEST=1 tools/e2e.sh` | **EXIT:1** at the flipped assertion |
| content package | `horizon-mod build` + `horizon-mod check` | 42,865 B, 56 entries, no code |
| content load-check | `tools/hmod-load.sh` | **PASS**, `EXIT:0`, negative control unknown ids |
| native package | `python3 tools/package.py` | 11,042 B, 8 entries |

Logs: `ports/_shared/evidence/laserbridges/`.

Not run, and why:

- the template's own fixture tests (`tools/test_template.py`, `tools/test_stage.py`) were removed:
  they compile a one-`break_block` fixture and assert one injection and one module, so they can
  only ever validate the template's example, not this bundle's four hooks and two modules. The
  standalone crate tests, the staged check/test/build and the in-game run cover the same ground for
  this port. `tools/audit` (the AST comparison crate they used) stays as a workspace member and
  builds cleanly.
- a recipe-crafting check in-game: the pin has no headless crafting-grid path (`ctl click-slot`
  needs a renderer). The six recipes are validated structurally by `horizon-mod check` and are
  present in the package.
- dyeing, receivers, sounds and the config file: deferred in `notes.md` (Gaps), so nothing claims
  them.
