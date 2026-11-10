# laserbridges (Horizon port)

A Horizon port of [laserbridges](Mars-The-Planet/Laser-Bridges-And-Doors) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- In scope:
- the four sources `laser_source_block`, `laser_source_fullblock`,
- `laser_fence_source_block`, `laser_fence_source_fullblock` with the upstream state set
- (`face` × `facing` × `powered` × `color`) and their upstream outlines/powered outlines;
- the two beams `laser_block_powered` and `laser_fence_powered` (unbreakable, light 10, thin
- collidable plates) that the sources write and clear;
- `getStateForPlacement` (clicked face → `face`/`facing`, plus the attached variants' sturdy-support
- requirement), `generateBridge`'s line (length, cut-through blocks, obstruction stop, shrinking

## Not yet ported

- The two receivers (`laser_attached_receiver`, `laser_attached_receiver_fence`): blocks, models,
- textures, lang, recipes, and their redstone output (`is_signal_source` + `block_signal` + the
- colour copy from the laser in front).
- Dyeing a source (`useItemOn` with a dye, and the dispenser's dye behaviour).
- `laserbridges:on`/`off` playback.
- `config/laserbridges.json` (`max_length`, `blocks_cut_through_by_lasers`).
- A colour change mid-line strands the far cells (upstream's exact-state test, kept deliberately).
- Beam blocks are unbreakable like upstream (bedrock template), so there is no drop and no

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: Mars-The-Planet/Laser-Bridges-And-Doors. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
