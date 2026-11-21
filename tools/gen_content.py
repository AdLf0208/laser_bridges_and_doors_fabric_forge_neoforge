#!/usr/bin/env python3
"""Emit the 26.3 content declarations Laser Bridges & Doors needs.

The jar carries the blockstates, models, textures, sounds, lang and loot
tables, and they are copied from it unchanged (see `SOURCE.md`). What it cannot
carry is `data/laserbridges/horizon/`: a content block is declared from a
vanilla template plus whatever differs, so the mod's own state set
(`face`/`facing`/`powered`/`color`), its outline/collision and its hardness,
light and occlusion have to be written out.

They are derived from the upstream shapes, which are `Block.box` calls in
1/16-block units (Laser Bridges & Doors 6.3, MIT):
`Constants`, `BridgeSourceBlock`, `FenceSourceBlock`,
`BridgeAttachedSourceBlock`, `FenceAttachedSourceBlock`, `LaserBridgeBlock`,
`LaserFenceBlock`, `BridgeAttachedReceiverBlock` and
`FenceAttachedReceiverBlock`. This script is the audit trail for that
translation; running it again rewrites the same files.

    python3 tools/gen_content.py
"""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PACKAGE = ROOT
DATA = PACKAGE / "data" / "laserbridges"

# Upstream shapes, in 1/16-block units, as `Block.box(minX, minY, minZ, maxX,
# maxY, maxZ)`: (minX, minY, minZ, maxX, maxY, maxZ).
CX = (7.99999, 0, 0, 8, 14, 16)
CZ = (0, 0, 7.99999, 16, 14, 8)
FX = (7.99999, 0, 0, 8, 16, 16)
FZ = (0, 0, 7.99999, 16, 16, 8)
FLAT = (0, 7.99999, 0, 16, 8, 16)  # every wall face of the bridge laser block
FENCE_NS = (7.99999, 0, 0, 8, 16, 16)
FENCE_WE = (0, 0, 7.99999, 16, 16, 8)

FACES = ("floor", "wall", "ceiling")
FACINGS = ("north", "south", "east", "west")


def box(units):
    """A `Block.box` tuple in block units, as the content schema takes it."""
    return [round(value / 16.0, 9) for value in units]


def boxes(*units):
    return [box(shape) for shape in units]


def axis(facing):
    return "x" if facing in ("east", "west") else "z"


def attached_bridge(face, facing):
    """`BridgeAttachedSourceBlock`."""
    if face == "floor":
        if axis(facing) == "x":
            return ((5, 0, 0, 11, 2, 16),)
        return ((0, 0, 5, 16, 2, 11),)
    if face == "ceiling":
        if axis(facing) == "x":
            return ((5, 14, 0, 11, 16, 16),)
        return ((0, 14, 5, 16, 16, 11),)
    return {
        "east": ((0, 5, 0, 2, 11, 16),),
        "west": ((14, 5, 0, 16, 11, 16),),
        "north": ((0, 5, 14, 16, 11, 16),),
        "south": ((0, 5, 0, 16, 11, 2),),
    }[facing]


def bridge_beam(face, facing):
    """`Constants`: the plane a bridge beam adds to its source's own cell."""
    if face == "floor":
        return FX if axis(facing) == "x" else FZ
    if face == "ceiling":
        return CX if axis(facing) == "x" else CZ
    return FLAT


def attached_bridge_powered(face, facing):
    return attached_bridge(face, facing) + (bridge_beam(face, facing),)


def full_bridge(face, facing):
    """`BridgeSourceBlock`."""
    if face == "floor":
        second = (5, 13, 0, 11, 15, 16) if axis(facing) == "x" else (0, 13, 5, 16, 15, 11)
        return ((0, 0, 0, 16, 13, 16), second)
    if face == "ceiling":
        second = (5, 1, 0, 11, 3, 16) if axis(facing) == "x" else (0, 1, 5, 16, 3, 11)
        return ((0, 3, 0, 16, 16, 16), second)
    return {
        "north": ((0, 0, 3, 16, 16, 16), (0, 5, 1, 16, 11, 3)),
        "south": ((0, 0, 0, 16, 16, 13), (0, 5, 13, 16, 11, 15)),
        "west": ((3, 0, 0, 16, 16, 16), (1, 5, 0, 3, 11, 16)),
        "east": ((0, 0, 0, 13, 16, 16), (13, 5, 0, 15, 11, 16)),
    }[facing]


def full_bridge_powered(face, facing):
    return full_bridge(face, facing) + (bridge_beam(face, facing),)


def attached_fence(face, facing):
    """`FenceAttachedSourceBlock`."""
    if face == "floor":
        if axis(facing) == "x":
            return ((0, 0, 5, 16, 2, 11),)
        return ((5, 0, 0, 11, 2, 16),)
    if face == "ceiling":
        if axis(facing) == "x":
            return ((0, 14, 5, 16, 16, 11),)
        return ((5, 14, 0, 11, 16, 16),)
    return {
        "north": ((5, 0, 14, 11, 16, 16),),
        "south": ((5, 0, 0, 11, 16, 2),),
        "west": ((14, 0, 5, 16, 16, 11),),
        "east": ((0, 0, 5, 2, 16, 11),),
    }[facing]


def fence_beam(face, facing):
    """A fence beam's plane; upstream swaps the bridge constants' axes."""
    if face == "floor":
        return FZ if axis(facing) == "x" else FX
    if face == "ceiling":
        return CZ if axis(facing) == "x" else CX
    return FENCE_NS if facing in ("north", "south") else FENCE_WE


def attached_fence_powered(face, facing):
    return attached_fence(face, facing) + (fence_beam(face, facing),)


def full_fence(face, facing):
    """`FenceSourceBlock`."""
    if face == "floor":
        second = (0, 13, 5, 16, 15, 11) if axis(facing) == "x" else (5, 13, 0, 11, 15, 16)
        return ((0, 0, 0, 16, 13, 16), second)
    if face == "ceiling":
        second = (0, 1, 5, 16, 3, 11) if axis(facing) == "x" else (5, 1, 0, 11, 3, 16)
        return ((0, 3, 0, 16, 16, 16), second)
    return {
        "north": ((0, 0, 3, 16, 16, 16), (5, 0, 1, 11, 16, 3)),
        "south": ((0, 0, 0, 16, 16, 13), (5, 0, 13, 11, 16, 15)),
        "west": ((3, 0, 0, 16, 16, 16), (1, 0, 5, 3, 16, 11)),
        "east": ((0, 0, 0, 13, 16, 16), (13, 0, 5, 15, 16, 11)),
    }[facing]


def full_fence_powered(face, facing):
    return full_fence(face, facing) + (fence_beam(face, facing),)


def bridge_laser(face, facing):
    """`LaserBridgeBlock`."""
    return (bridge_beam(face, facing),)


def fence_laser(face, facing):
    """`LaserFenceBlock`."""
    return (fence_beam(face, facing),)


def receiver_bridge(face, facing):
    """`BridgeAttachedReceiverBlock` (`ILaserGeneratingBlock.handleGetShape`)."""
    if face == "floor":
        if axis(facing) == "x":
            return ((5, 0, 0, 11, 2, 16),)
        return ((0, 0, 5, 16, 2, 11),)
    if face == "ceiling":
        if axis(facing) == "x":
            return ((5, 14, 0, 11, 16, 16),)
        return ((0, 14, 5, 16, 16, 11),)
    return {
        "east": ((0, 5, 0, 2, 11, 16),),
        "west": ((14, 5, 0, 16, 11, 16),),
        "north": ((0, 5, 14, 16, 11, 16),),
        "south": ((0, 5, 0, 16, 11, 2),),
    }[facing]


def receiver_bridge_powered(face, facing):
    """A powered bridge receiver adds the bridge beam's plane, as the sources do."""
    return receiver_bridge(face, facing) + (bridge_beam(face, facing),)


def receiver_fence(face, facing):
    """`FenceAttachedReceiverBlock`."""
    if face == "floor":
        if axis(facing) == "x":
            return ((0, 0, 5, 16, 2, 11),)
        return ((5, 0, 0, 11, 2, 16),)
    if face == "ceiling":
        if axis(facing) == "x":
            return ((0, 14, 5, 16, 16, 11),)
        return ((5, 14, 0, 11, 16, 16),)
    return {
        "north": ((5, 0, 14, 11, 16, 16),),
        "south": ((5, 0, 0, 11, 16, 2),),
        "west": ((14, 0, 5, 16, 16, 11),),
        "east": ((0, 0, 5, 2, 16, 11),),
    }[facing]


def receiver_fence_powered(face, facing):
    """The fence receiver's powered shape mixes `Constants`' axes, as upstream does."""
    if face == "floor":
        extra = FZ if axis(facing) == "x" else FX
    elif face == "ceiling":
        extra = CZ if axis(facing) == "x" else CX
    else:
        extra = FENCE_NS if facing in ("north", "south") else FENCE_WE
    return receiver_fence(face, facing) + (extra,)


SOURCES = {
    "laser_source_block": (attached_bridge, attached_bridge_powered),
    "laser_source_fullblock": (full_bridge, full_bridge_powered),
    "laser_fence_source_block": (attached_fence, attached_fence_powered),
    "laser_fence_source_fullblock": (full_fence, full_fence_powered),
}

# `ModRegistry`: the two wall-mounted sources and the two receivers emit light only
# while powered; everything else is a constant 10. The content schema takes that as
# `state_light`, so the former constant-light deviation is gone.
POWERED_LIGHT = {"properties": ["powered"], "values": {"true": 10, "false": 0}}

RECEIVERS = {
    "laser_attached_receiver": (receiver_bridge, receiver_bridge_powered),
    "laser_attached_receiver_fence": (receiver_fence, receiver_fence_powered),
}

# `AbstractAttachedSourceBlock`'s two ids: the only sources whose light is per-state.
ATTACHED = {"laser_source_block", "laser_fence_source_block"}

LASERS = {
    "laser_block_powered": bridge_laser,
    "laser_fence_powered": fence_laser,
}

# `AbstractSourceBlock.registerDefaultState`: `AttachFace` defaults to WALL,
# `HORIZONTAL_FACING` to NORTH, `POWERED` to false, `COLOR` to 0. The declared
# values are listed in that order so the first combination is the default.
STATE_PROPERTIES = {
    "face": {"type": "enum", "values": list(FACES)},
    "facing": {"type": "enum", "values": list(FACINGS)},
    "powered": {"type": "boolean"},
    "color": {"type": "enum", "values": [str(colour) for colour in range(16)]},
}


def source_file(name, shapes, powered_shapes, attached):
    values = {}
    for face in FACES:
        for facing in FACINGS:
            values[f"{face},{facing},false"] = boxes(*shapes(face, facing))
            values[f"{face},{facing},true"] = boxes(*powered_shapes(face, facing))
    block = {
        "template": "minecraft:stone",
        "properties": {
            "hardness": 0.8,
            "explosion_resistance": 0.8,
            "light": 10,
            "requires_tool": False,
            "occludes": False,
        },
        "state_properties": STATE_PROPERTIES,
        "shapes": {
            "properties": ["face", "facing", "powered"],
            "values": values,
        },
        "item": True,
    }
    if attached:
        # `ModRegistry`: the attached sources' `lightLevel` is `POWERED ? 10 : 0`.
        block["state_light"] = POWERED_LIGHT
    return block


def receiver_file(name, shapes, powered_shapes):
    values = {}
    for face in FACES:
        for facing in FACINGS:
            values[f"{face},{facing},false"] = boxes(*shapes(face, facing))
            values[f"{face},{facing},true"] = boxes(*powered_shapes(face, facing))
    return {
        "template": "minecraft:stone",
        "properties": {
            "hardness": 0.8,
            "explosion_resistance": 0.8,
            "requires_tool": False,
            "occludes": False,
        },
        "state_light": POWERED_LIGHT,
        "state_properties": STATE_PROPERTIES,
        "shapes": {
            "properties": ["face", "facing", "powered"],
            "values": values,
        },
        "item": True,
    }


def laser_file(shapes):
    values = {}
    for face in FACES:
        for facing in FACINGS:
            values[f"{face},{facing}"] = boxes(*shapes(face, facing))
    return {
        # `strength(-1.0F, 3600000.0F)`: unbreakable, like bedrock.
        "template": "minecraft:bedrock",
        "properties": {"light": 10, "occludes": False},
        "state_properties": {
            "face": {"type": "enum", "values": list(FACES)},
            "facing": {"type": "enum", "values": list(FACINGS)},
            "color": {"type": "enum", "values": [str(colour) for colour in range(16)]},
        },
        "shapes": {"properties": ["face", "facing"], "values": values},
        "item": False,
    }


RECIPES = {
    # `CommonClass.init`'s `DeimosRecipeGenerator` calls. The conversion
    # recipes are the 1:1 shapeless swaps the generator writes.
    "laser_source_block": {
        "type": "minecraft:crafting_shaped",
        "pattern": ["IGI", "IEI", "III"],
        "key": {
            "I": "minecraft:iron_ingot",
            "G": "minecraft:glass",
            "E": "minecraft:end_crystal",
        },
        "result": {"id": "laserbridges:laser_source_block"},
    },
    "laser_fence_source_block": {
        "type": "minecraft:crafting_shaped",
        "pattern": ["III", "IEG", "III"],
        "key": {
            "I": "minecraft:iron_ingot",
            "G": "minecraft:glass",
            "E": "minecraft:end_crystal",
        },
        "result": {"id": "laserbridges:laser_fence_source_block"},
    },
    "laser_source_fullblock": {
        "type": "minecraft:crafting_shaped",
        "pattern": [" B ", "III", "III"],
        "key": {
            "B": "laserbridges:laser_source_block",
            "I": "minecraft:iron_ingot",
        },
        "result": {"id": "laserbridges:laser_source_fullblock"},
    },
    "laser_fence_source_fullblock": {
        "type": "minecraft:crafting_shaped",
        "pattern": ["II ", "IIF", "II "],
        "key": {
            "F": "laserbridges:laser_fence_source_block",
            "I": "minecraft:iron_ingot",
        },
        "result": {"id": "laserbridges:laser_fence_source_fullblock"},
    },
    "convert_bridge_to_fence": {
        "type": "minecraft:crafting_shapeless",
        "ingredients": ["laserbridges:laser_source_block"],
        "result": {"id": "laserbridges:laser_fence_source_block"},
    },
    "convert_fence_to_bridge": {
        "type": "minecraft:crafting_shapeless",
        "ingredients": ["laserbridges:laser_fence_source_block"],
        "result": {"id": "laserbridges:laser_source_block"},
    },
    "convert_fullblock_bridge_to_fence": {
        "type": "minecraft:crafting_shapeless",
        "ingredients": ["laserbridges:laser_source_fullblock"],
        "result": {"id": "laserbridges:laser_fence_source_fullblock"},
    },
    "convert_fullblock_fence_to_bridge": {
        "type": "minecraft:crafting_shapeless",
        "ingredients": ["laserbridges:laser_fence_source_fullblock"],
        "result": {"id": "laserbridges:laser_source_fullblock"},
    },
    "laser_attached_receiver": {
        "type": "minecraft:crafting_shaped",
        "pattern": ["IGI", "IEI", "III"],
        "key": {
            "I": "minecraft:iron_ingot",
            "G": "minecraft:glass",
            "E": "minecraft:redstone",
        },
        "result": {"id": "laserbridges:laser_attached_receiver"},
    },
    "laser_attached_receiver_fence": {
        "type": "minecraft:crafting_shaped",
        "pattern": ["III", "IEG", "III"],
        "key": {
            "I": "minecraft:iron_ingot",
            "G": "minecraft:glass",
            "E": "minecraft:redstone",
        },
        "result": {"id": "laserbridges:laser_attached_receiver_fence"},
    },
    "convert_receiver_to_fence": {
        "type": "minecraft:crafting_shapeless",
        "ingredients": ["laserbridges:laser_attached_receiver"],
        "result": {"id": "laserbridges:laser_attached_receiver_fence"},
    },
    "convert_receiver_fence_to_receiver": {
        "type": "minecraft:crafting_shapeless",
        "ingredients": ["laserbridges:laser_attached_receiver_fence"],
        "result": {"id": "laserbridges:laser_attached_receiver"},
    },
}

TAB = {
    "title": "Laser Bridges & Doors",
    "icon": "laserbridges:laser_source_block",
    "items": [
        "laserbridges:laser_source_block",
        "laserbridges:laser_fence_source_block",
        "laserbridges:laser_source_fullblock",
        "laserbridges:laser_fence_source_fullblock",
        "laserbridges:laser_attached_receiver",
        "laserbridges:laser_attached_receiver_fence",
    ],
}


def write(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2) + "\n")


def main():
    for name, (shapes, powered) in SOURCES.items():
        write(
            DATA / "horizon" / "block" / f"{name}.json",
            source_file(name, shapes, powered, name in ATTACHED),
        )
    for name, (shapes, powered) in RECEIVERS.items():
        write(DATA / "horizon" / "block" / f"{name}.json", receiver_file(name, shapes, powered))
    for name, shapes in LASERS.items():
        write(DATA / "horizon" / "block" / f"{name}.json", laser_file(shapes))
    for name, recipe in RECIPES.items():
        write(DATA / "recipe" / f"{name}.json", recipe)
    write(DATA / "horizon" / "creative_tab" / "laserbridges.json", TAB)
    print(
        f"wrote {len(SOURCES) + len(RECEIVERS) + len(LASERS)} block declarations, "
        f"{len(RECIPES)} recipes, 1 tab"
    )


if __name__ == "__main__":
    main()
