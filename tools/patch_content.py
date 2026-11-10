#!/usr/bin/env python3
"""Patch the carried Laser Bridges & Doors files for 26.3 and for this slice.

The assets, blockstates, models, sounds and loot tables in `content/` come
from the exact ATM10 jar, byte for byte (see `_shared/carry-work/laserbridges/
SOURCE.md`). Four things have to change:

* the loot tables use the 1.20/1.21 array form of a pool condition; 26.3 reads
  one `condition` per pool, so the array becomes an object;
* 26.3 moved item model *definitions* to `assets/<ns>/items/`, so each shipped
  block item needs one pointing at the block model the jar's `models/item`
  file named;
* the lang file carries keys for the deferred receivers and a config screen
  this slice has no mod for, so only the shipped blocks stay;
* the jar's `minecraft:mineable/pickaxe` tag also lists the two deferred
  receivers; the port ships only the blocks it registers.

    python3 tools/patch_content.py
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTENT = ROOT / "content"
ASSETS = CONTENT / "assets" / "laserbridges"
DATA = CONTENT / "data"

# The blocks this slice ships: four sources, two lasers. `laserbridges:laser_*`
# beam blocks have no item (`item: false`), so they get no model definition.
SOURCES = (
    "laser_source_block",
    "laser_fence_source_block",
    "laser_source_fullblock",
    "laser_fence_source_fullblock",
)
LASERS = ("laser_block_powered", "laser_fence_powered")
SHIPPED = SOURCES + LASERS


def load(path):
    return json.loads(path.read_text())


def dump(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def patch_loot_tables():
    for name in SHIPPED:
        path = DATA / "laserbridges" / "loot_table" / "blocks" / f"{name}.json"
        if not path.exists():
            # A beam block is unbreakable, so the jar has no table for it.
            continue
        table = load(path)
        for pool in table["pools"]:
            if "conditions" in pool:
                conditions = pool.pop("conditions")
                if len(conditions) != 1:
                    raise SystemExit(f"{path}: expected one condition, got {conditions}")
                pool["condition"] = {
                    "type": conditions[0].get("condition", conditions[0].get("type"))
                }
        dump(path, table)


def item_definitions():
    """26.3 `assets/<ns>/items/<name>.json` for each shipped block item."""
    existing = {name: load(ASSETS / "models" / "item" / f"{name}.json") for name in SOURCES}
    for name, model in existing.items():
        parent = model.get("parent")
        if not parent or not parent.startswith("laserbridges:"):
            raise SystemExit(f"unexpected item model for {name}: {model}")
        dump(
            ASSETS / "items" / f"{name}.json",
            {"model": {"type": "minecraft:model", "model": parent}},
        )


def lang():
    path = ASSETS / "lang" / "en_us.json"
    source = load(path)
    keep = {
        f"block.laserbridges.{name}": value
        for key, value in source.items()
        if key.startswith("block.laserbridges.")
        for name in (key.rsplit(".", 1)[-1],)
        if name in SHIPPED
    }
    if len(keep) != len(SHIPPED):
        raise SystemExit(f"lang keys for the slice: {sorted(keep)}")
    dump(path, keep)


def tags():
    path = DATA / "minecraft" / "tags" / "block" / "mineable" / "pickaxe.json"
    dump(path, {"replace": False, "values": [f"laserbridges:{name}" for name in SHIPPED]})


def main():
    patch_loot_tables()
    item_definitions()
    lang()
    tags()
    print("patched loot tables, item definitions, lang and the pickaxe tag")


if __name__ == "__main__":
    main()
