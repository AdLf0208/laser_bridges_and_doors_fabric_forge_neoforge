# Notice and provenance for this port

**Upstream:** *Laser Bridges & Doors* 6.3 by **Mars** — repository
`Mars-The-Planet/Laser-Bridges-And-Doors`, branch `1.21.1`, commit
`792de13e5acb5ba1f650a1b1de46bf7b43c6a620`; the pack's own jar is
`laserbridges-1.21.1-neoforge-6.3.jar` (SHA-1 `6f3ec7b8e77450ed0a0865e3ca646ab41464adf3`).

**Licence: MIT.** Read from the artefact, not a catalogue: the jar's
`META-INF/neoforge.mods.toml` declares `license = "MIT"`, `authors = "Mars"`, `version = "6.3"`, and
the repository's `LICENSE` is the MIT text with Mars's copyright line. The full text travels in this
package as [`LICENSE`](LICENSE), which adds this port's copyright line beside the original owner's.

**What this package contains**

- `src/lib.rs`, `src/runtime.rs` — a clean-room Rust rewrite of the ported behaviour (the beam's
  line, its length, what stops it, and the state a placement takes). No Java source was copied,
  translated or decompiled; the upstream Java was read for behaviour only, as the campaign's rules
  require.
- `assets/laserbridges/**` and `data/laserbridges/**` — the *upstream mod's own* models, blockstates,
  textures, sounds, language file and loot tables, carried unchanged from the jar except for the
  26.3 conversions and the slice's removals, which are mechanical and reproducible with
  `tools/patch_content.py`. `data/laserbridges/horizon/**` (the block declarations) and the recipes
  are written for 26.3 by `tools/gen_content.py`, derived from upstream's own shape and recipe
  constants.

**Local-only:** the port is not published by this job. Publishing needs the licence to allow
derivatives (MIT does) *and* the campaign maintainer's approval (`ARCHITECTURE.md` §8).
