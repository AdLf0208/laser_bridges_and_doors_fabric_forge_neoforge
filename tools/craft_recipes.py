#!/usr/bin/env python3
"""Craft every recipe the package carries, in a real crafting table.

Called by `tools/e2e.sh` with the open game's control socket. The harness opens a
crafting table as a screen whose 3x3 grid is slots 100..108 (row-major) and whose
result is 200; the player's own slots keep their inventory numbers (hotbar 0..8).

What it proves, for each of the twelve carried recipes:

1. the game reads the recipe: the declared pattern makes exactly one of the declared
   output, in the crafting table (`minecraft:crafting_shaped`, 3x3) or the same table's
   single slot (`minecraft:crafting_shapeless`);
2. the ingredients are consumed: every one of them is gone from the inventory and the
   grid is empty after the result is taken;
3. the crafted item lands in the inventory under its own id.

Upstream generates these recipes in Java (`DeimosRecipeGenerator` in `CommonClass.init`)
and the jar carries no recipe JSON, so the port's `data/laserbridges/recipe/*.json` are
the mod's own grid data written for 26.3; this is the proof they are live data and not
decoration.

Exit 1 with the observed result on the first mismatch. `NEGATIVE=1` skips the take of
the last recipe and inverts its consumption assertion, so the run MUST exit 1.
"""
from __future__ import annotations

import json
import os
import subprocess
import sys

SELFTEST = os.environ.get("NEGATIVE") == "1"

# The harness's crafting table screen: the 3x3 grid (row-major) and the result.
GRID = list(range(100, 109))
RESULT = 200
HOTBAR = 0

IRON = "minecraft:iron_ingot"
GLASS = "minecraft:glass"
CRYSTAL = "minecraft:end_crystal"
REDSTONE = "minecraft:redstone"

SOURCE = "laserbridges:laser_source_block"
FENCE_SOURCE = "laserbridges:laser_fence_source_block"
FULL = "laserbridges:laser_source_fullblock"
FENCE_FULL = "laserbridges:laser_fence_source_fullblock"
RECEIVER = "laserbridges:laser_attached_receiver"
FENCE_RECEIVER = "laserbridges:laser_attached_receiver_fence"

# Every carried recipe: `grid` maps a slot offset (0..8) to the item one of it goes in,
# `ingredients` is the exact inventory the craft consumes, `makes` is the result.
RECIPES = [
    {
        "name": "laser_source_block",
        "grid": {0: IRON, 1: GLASS, 2: IRON, 3: IRON, 4: CRYSTAL, 5: IRON,
                 6: IRON, 7: IRON, 8: IRON},
        "ingredients": [(IRON, 7), (GLASS, 1), (CRYSTAL, 1)],
        "makes": SOURCE,
    },
    {
        "name": "laser_fence_source_block",
        "grid": {0: IRON, 1: IRON, 2: IRON, 3: IRON, 4: CRYSTAL, 5: GLASS,
                 6: IRON, 7: IRON, 8: IRON},
        "ingredients": [(IRON, 7), (GLASS, 1), (CRYSTAL, 1)],
        "makes": FENCE_SOURCE,
    },
    {
        "name": "laser_source_fullblock",
        "grid": {1: SOURCE, 3: IRON, 4: IRON, 5: IRON, 6: IRON, 7: IRON, 8: IRON},
        "ingredients": [(SOURCE, 1), (IRON, 6)],
        "makes": FULL,
    },
    {
        "name": "laser_fence_source_fullblock",
        "grid": {0: IRON, 1: IRON, 3: IRON, 4: IRON, 5: FENCE_SOURCE, 6: IRON, 7: IRON},
        "ingredients": [(FENCE_SOURCE, 1), (IRON, 6)],
        "makes": FENCE_FULL,
    },
    {
        "name": "convert_bridge_to_fence",
        "grid": {4: SOURCE},
        "ingredients": [(SOURCE, 1)],
        "makes": FENCE_SOURCE,
    },
    {
        "name": "convert_fence_to_bridge",
        "grid": {4: FENCE_SOURCE},
        "ingredients": [(FENCE_SOURCE, 1)],
        "makes": SOURCE,
    },
    {
        "name": "convert_fullblock_bridge_to_fence",
        "grid": {4: FULL},
        "ingredients": [(FULL, 1)],
        "makes": FENCE_FULL,
    },
    {
        "name": "convert_fullblock_fence_to_bridge",
        "grid": {4: FENCE_FULL},
        "ingredients": [(FENCE_FULL, 1)],
        "makes": FULL,
    },
    {
        "name": "laser_attached_receiver",
        "grid": {0: IRON, 1: GLASS, 2: IRON, 3: IRON, 4: REDSTONE, 5: IRON,
                 6: IRON, 7: IRON, 8: IRON},
        "ingredients": [(IRON, 7), (GLASS, 1), (REDSTONE, 1)],
        "makes": RECEIVER,
    },
    {
        "name": "laser_attached_receiver_fence",
        "grid": {0: IRON, 1: IRON, 2: IRON, 3: IRON, 4: REDSTONE, 5: GLASS,
                 6: IRON, 7: IRON, 8: IRON},
        "ingredients": [(IRON, 7), (GLASS, 1), (REDSTONE, 1)],
        "makes": FENCE_RECEIVER,
    },
    {
        "name": "convert_receiver_to_fence",
        "grid": {4: RECEIVER},
        "ingredients": [(RECEIVER, 1)],
        "makes": FENCE_RECEIVER,
    },
    {
        "name": "convert_receiver_fence_to_receiver",
        "grid": {4: FENCE_RECEIVER},
        "ingredients": [(FENCE_RECEIVER, 1)],
        "makes": RECEIVER,
    },
]


def main() -> int:
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]

    def run(*args: str) -> dict:
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks: int = 2) -> None:
        run("wait", str(ticks))

    def slots() -> list[dict]:
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"no menu is open: {screen}")
        return screen["slots"]

    def counts() -> dict[str, int]:
        return run("inventory")["counts"]

    def find(item: str) -> int:
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"missing ingredient {item} (inventory: {counts()})")

    def place(item: str, targets: list[int]) -> None:
        """Pick the stack up, put one in each target, put the rest back."""
        slot = find(item)
        run("click-slot", str(slot))
        wait(1)
        for target in targets:
            run("click-slot", str(target), "right")
            wait(1)
        run("click-slot", str(slot))
        wait(2)

    def setup(recipe: dict) -> None:
        """A fresh table, an empty inventory and exactly this recipe's ingredients."""
        try:
            run("close")
        except SystemExit:
            pass
        wait(3)
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {y} {float(z) + 0.5}")
        wait(3)
        run("command", "clear @s")
        for item, count in recipe["ingredients"]:
            run("give", item, str(count))
        wait(3)
        run("interact", x, y, z)
        wait(5)
        open_slots = {s["slot"] for s in slots()}
        missing = [slot for slot in (GRID[0], RESULT) if slot not in open_slots]
        if missing:
            raise SystemExit(f"FAIL: the crafting table did not open (missing slots {missing})")

    def fill(recipe: dict) -> None:
        by_item: dict[str, list[int]] = {}
        for offset, item in recipe["grid"].items():
            by_item.setdefault(item, []).append(GRID[offset])
        for item, targets in by_item.items():
            place(item, sorted(targets))

    def result() -> dict | None:
        found = [s for s in slots() if s["slot"] == RESULT]
        if not found or not found[0].get("item"):
            return None
        return found[0]

    def take() -> None:
        run("click-slot", str(RESULT))
        wait(2)
        run("click-slot", str(HOTBAR))
        wait(2)

    def consumed(recipe: dict) -> None:
        left = counts()
        for item, _ in recipe["ingredients"]:
            if left.get(item, 0) != 0:
                raise SystemExit(f"FAIL: {recipe['name']}: {left[item]} of {item} left after the craft")
        grid = [s for s in slots() if s["slot"] in GRID and s.get("item")]
        if grid:
            raise SystemExit(f"FAIL: {recipe['name']}: the grid still holds {grid}")

    for index, recipe in enumerate(RECIPES):
        last = index == len(RECIPES) - 1
        print(f"== {recipe['name']} ==")
        setup(recipe)
        fill(recipe)
        got = result()
        print(f"  result: {json.dumps(got)}")
        if not got or got.get("item") != recipe["makes"] or got.get("count") != 1:
            grid = [s for s in slots() if s["slot"] in GRID]
            raise SystemExit(
                f"FAIL: {recipe['name']}: expected 1 {recipe['makes']}, got {got}; grid {json.dumps(grid)}"
            )
        if last and SELFTEST:
            # Deliberately wrong: take nothing, then run the consumption assertion.
            # A run that reports success here proves the check cannot fail.
            consumed(recipe)
            raise SystemExit(
                f"FAIL: selftest: {recipe['name']} was never taken yet the consumption check passed"
            )
        take()
        consumed(recipe)
        if counts().get(recipe["makes"], 0) < 1:
            raise SystemExit(f"FAIL: {recipe['name']}: the crafted item is not in the inventory")
        print(f"  crafted 1 {recipe['makes']}, ingredients consumed")

    run("close")
    print(f"PASS: all {len(RECIPES)} carried recipes craft in a crafting table, each makes its declared item and consumes its ingredients")
    return 0


if __name__ == "__main__":
    sys.exit(main())
