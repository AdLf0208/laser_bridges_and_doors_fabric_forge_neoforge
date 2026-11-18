#!/usr/bin/env bash
# In-game check for Laser Bridges & Doors (the laser sources and the beams they write).
#
# This script needs a game the LAUNCHER built with this package installed (mod API 0.2):
#
#   horizon-launcher native-build . --instance <instance> --trust-native-code
#   GAME=<instance>/horizon-client CTL=<matching horizon-ctl> tools/e2e.sh
#
# The package is one flat `.hmod`: its `assets/`+`data/` are the carried content and its
# `src/` is the behaviour, so there is no `--dev-mod` and no separate content archive to point
# at. CONTROL_GAME=<an unmodded launcher game> adds the negative leg: the same ids are unknown
# there, which is what proves the checks are not vacuous.
#
# The 0.1.0 runs in ../_shared/evidence/laserbridges/e2e*.log are HISTORICAL: they drove a
# `tools/horizon.py` build with the retired `.hnmod` compiled in, a route that no longer exists.
# The assertions below are unchanged, because the rules they test are unchanged
# (`src/runtime.rs` is byte-identical); only the launch line moved to the launcher's build.
#
# The End arena: a stone floor at y=99 (top surface y=100) and clear air above. A wall-facing
# source at y=100 writes a beam along the same y, so its plate (8 pixels up, 1/16 thick) has its
# top at y=100.5 and a player can stand on it.
#
# Scenario: a placed source takes its face and facing from the click; a powered source fills its
# line and the beam holds the player at 100.5 while the unpowered floor is 100.0; removing the
# redstone clears every cell and re-powering rebuilds them; a signal of 7 builds 6 cells; a floor
# fence source builds a 14-cell vertical door that opens when unpowered; controls: the wrong
# facing, an unpowered source, a stone obstruction and a broken source each leave a line empty
# that the powered run fills; every declared state binds, `color=16` and a beam-block `powered`
# are refused; the four source items exist and the beam blocks have none.
#
# SELFTEST=1 flips the full-power beam assertion (it must FAIL), proving the scenario can fail.
set -euo pipefail
export LC_ALL=C

GAME=${GAME:?set GAME to the launcher-built horizon-client carrying laserbridges}
CONTROL_GAME=${CONTROL_GAME:-}
CTL=${CTL:-horizon-ctl}
RES=${RES:-}
SOCK=${SOCK:-/tmp/opencode/laserbridges-e2e.sock}
SELFTEST=${SELFTEST:-0}
KEEP=${KEEP:-0}
WORLD=LaserBridgesE2E

command -v "$CTL" >/dev/null 2>&1 || [ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
LAUNCH_EXTRA=()
if [ -n "$RES" ]; then
    [ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
    LAUNCH_EXTRA=(--resources "$RES")
fi

SCRATCH=$(mktemp -d /tmp/opencode/laserbridges-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; ctl logs --lines 80 >&2 || true; exit 1; }
setblock() { ctl command "execute in minecraft:the_end run setblock $1" >/dev/null; }
block_json() { ctl block "$1" "$2" "$3"; }

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then echo "kept scratch: $SCRATCH" >&2; else rm -rf "$SCRATCH" 2>/dev/null || true; fi
    echo "EXIT:$status"
}
trap cleanup EXIT

# The block at a position, printed, and checked against a block id and any
# number of `key=value` state fragments.
expect_block() { # x y z block-id [fragment...]
    local x=$1 y=$2 z=$3 id=$4; shift 4
    local json; json=$(block_json "$x" "$y" "$z")
    case "$json" in
        *"$id"*) ;;
        *) fail "expected $id at $x,$y,$z, got $json" ;;
    esac
    local fragment
    for fragment in "$@"; do
        case "$json" in
            *"$fragment"*) ;;
            *) fail "expected $fragment at $x,$y,$z, got $json" ;;
        esac
    done
    echo "block $x,$y,$z: $json"
}

expect_air() { # x y z
    local json; json=$(block_json "$1" "$2" "$3")
    case "$json" in
        *minecraft:air*) echo "block $1,$2,$3: air" ;;
        *) fail "expected air at $1,$2,$3, got $json" ;;
    esac
}

expect_setblock() { # state
    local out; out=$(ctl command "execute in minecraft:the_end run setblock $1")
    case "$out" in
        *"Changed the block"*) ;;
        *) fail "setblock $1 was refused: $out" ;;
    esac
}

expect_setblock_refused() { # state
    local out; out=$(ctl command "execute in minecraft:the_end run setblock $1")
    case "$out" in
        *"Changed the block"*) fail "setblock $1 was accepted, it should be refused: $out" ;;
        *) echo "refused as required: $out" ;;
    esac
}

feet_y() { ctl state | jq -r '.player.position[1]'; }

# The player's feet land on the beam plate's top (100.5) or on the floor
# (100.0); the drop is 0.7 blocks, so 20 ticks is plenty.
drop_to() { # x y z label
    ctl command "execute in minecraft:the_end run tp @s $1 $2 $3" >/dev/null
    ctl wait 20 >/dev/null
    local y; y=$(feet_y)
    echo "feet y at $4: $y" >&2
    echo "$y"
}

expect_resting() { # actual expected label
    awk -v a="$1" -v e="$2" 'BEGIN { d = a - e; if (d < 0) d = -d; exit !(d < 0.02) }' \
        || fail "$3: expected y $2, got $1"
}
launch() { # game world
    "$CTL" --socket "$SOCK" launch \
        --game "$1" --world "$2" --seed 20261108 --creative --headless --wait \
        ${LAUNCH_EXTRA[@]+"${LAUNCH_EXTRA[@]}"}
    ctl wait-ready 120
}

if [ -n "$CONTROL_GAME" ]; then
    [ -x "$CONTROL_GAME" ] || { echo "missing CONTROL_GAME: $CONTROL_GAME" >&2; exit 1; }
    echo "== control: a game without the package does not know the ids =="
    launch "$CONTROL_GAME" ControlLaserBridges
    control_reply=$(ctl command "setblock 0 100 0 laserbridges:laser_source_block" 2>&1 || true)
    echo "control setblock: $control_reply"
    case "$control_reply" in
        *[Uu]nknown*|*failed*|*error*) ;;
        *) fail "a game without the package accepted laserbridges:laser_source_block: $control_reply" ;;
    esac
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -f "$SOCK"
fi

echo "== launch (The End, the laserbridges package) =="
cd "$SCRATCH"
launch "$GAME" "$WORLD"
ctl command "execute in minecraft:the_end run tp @s 0.5 100 0.5" >/dev/null
ctl wait 10
ctl command "execute in minecraft:the_end run fill -6 99 -16 26 99 26 minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run fill -6 100 -16 26 110 26 minecraft:air" >/dev/null
ctl command "gamemode creative @s" >/dev/null
ctl wait 2

echo "== A: a placed source takes its face and facing from the click =="
# The target cell floats (air below it), so the only face `ctl place` can aim
# at is the wall beside it: the source must mount on the wall, not the floor.
setblock "1 101 -2 minecraft:stone"
ctl command "execute in minecraft:the_end run tp @s 3.5 101 -2.5" >/dev/null
ctl wait 2
ctl give laserbridges:laser_source_block 1 >/dev/null
ctl wait 4
place_reply=$(ctl place 2 101 -2 --item laserbridges:laser_source_block)
echo "place reply: $place_reply"
ctl wait 4
expect_block 2 101 -2 laserbridges:laser_source_block face=wall facing=west powered=false
# The floor click (a solid block below, air elsewhere) mounts on the floor.
ctl command "execute in minecraft:the_end run tp @s 3.5 100 -4.5" >/dev/null
ctl wait 2
place_floor=$(ctl place 2 100 -4 --item laserbridges:laser_source_block)
echo "floor place reply: $place_floor"
ctl wait 4
expect_block 2 100 -4 laserbridges:laser_source_block face=floor

echo "== B: a powered bridge source builds a walkable beam =="
setblock "3 100 0 minecraft:stone"
setblock "4 100 0 laserbridges:laser_source_fullblock[face=wall,facing=east,color=0,powered=false]"
ctl wait 4
expect_block 4 100 0 laserbridges:laser_source_fullblock powered=false
expect_air 5 100 0
setblock "4 101 0 minecraft:redstone_block"
ctl wait 4
expect_block 4 100 0 laserbridges:laser_source_fullblock powered=true
if [ "$SELFTEST" = 1 ]; then
    # Deliberately wrong: the beam cell is filled, so this must fail.
    expect_air 10 100 0
fi
expect_block 5 100 0 laserbridges:laser_block_powered face=wall facing=east color=0
expect_block 10 100 0 laserbridges:laser_block_powered face=wall facing=east color=0
expect_block 18 100 0 laserbridges:laser_block_powered face=wall facing=east color=0
expect_air 19 100 0
beam_y=$(drop_to 10.5 100.7 0.5 beam)
expect_resting "$beam_y" 100.5 "standing on the beam"
echo "the beam is walkable: feet at $beam_y on the plate whose top is 100.5"

setblock "4 101 0 minecraft:air"
ctl wait 4
expect_block 4 100 0 laserbridges:laser_source_fullblock powered=false
expect_air 5 100 0
expect_air 10 100 0
expect_air 18 100 0
floor_y=$(drop_to 10.5 100.7 0.5 unpowered)
expect_resting "$floor_y" 100.0 "standing where the beam was"
echo "unpowered the beam is gone: feet at $floor_y on the floor at 100.0"

setblock "4 101 0 minecraft:redstone_block"
ctl wait 4
expect_block 18 100 0 laserbridges:laser_block_powered
echo "re-powering builds the beam again"
setblock "4 101 0 minecraft:air"
ctl wait 4

echo "== C: a weaker signal gives a shorter beam =="
# A redstone wire cannot sit above the source (the source's thin outline is not
# sturdy, so the wire breaks and the pin's wire also refuses to power the block
# below it: `wire_signal` returns 0 for `Direction::Down`). A weighted pressure
# plate beside it does report its own power, and its `power` state can be set.
setblock "3 100 6 minecraft:stone"
setblock "4 100 6 laserbridges:laser_source_fullblock[face=wall,facing=east,color=0,powered=false]"
setblock "4 101 6 minecraft:redstone_block"
ctl wait 4
expect_block 4 100 6 laserbridges:laser_source_fullblock powered=true
expect_block 18 100 6 laserbridges:laser_block_powered
setblock "4 101 6 minecraft:air"
setblock "4 100 7 minecraft:light_weighted_pressure_plate[power=7]"
ctl wait 6
plate=$(block_json 4 100 7)
echo "the plate beside the source: $plate"
expect_block 4 100 6 laserbridges:laser_source_fullblock powered=true
expect_block 5 100 6 laserbridges:laser_block_powered
expect_block 10 100 6 laserbridges:laser_block_powered
expect_air 11 100 6
expect_air 18 100 6
echo "a signal of 7 built 6 cells (5..10) and cleared the rest of the line"
setblock "4 100 7 minecraft:air"
ctl wait 4
expect_air 5 100 6

echo "== D: a floor fence source builds a vertical door =="
setblock "0 100 20 laserbridges:laser_fence_source_fullblock[face=floor,facing=north,color=0,powered=false]"
ctl wait 4
expect_air 0 101 20
setblock "0 100 19 minecraft:redstone_block"
ctl wait 4
expect_block 0 100 20 laserbridges:laser_fence_source_fullblock powered=true
expect_block 0 101 20 laserbridges:laser_fence_powered face=floor
expect_block 0 104 20 laserbridges:laser_fence_powered face=floor
expect_block 0 114 20 laserbridges:laser_fence_powered face=floor
expect_air 0 115 20
echo "the door stands 14 cells tall (101..114)"
setblock "0 100 19 minecraft:air"
ctl wait 4
expect_air 0 101 20
expect_air 0 114 20
echo "unpowered the door opens (every cell is air again)"

echo "== E: controls =="
# The same source facing north beams north; the +x line stays empty.
setblock "3 100 -6 minecraft:stone"
setblock "4 100 -6 laserbridges:laser_source_fullblock[face=wall,facing=north,color=0,powered=false]"
setblock "4 101 -6 minecraft:redstone_block"
ctl wait 4
expect_block 4 100 -6 laserbridges:laser_source_fullblock powered=true
expect_block 4 100 -7 laserbridges:laser_block_powered facing=north
expect_air 5 100 -6
expect_air 10 100 -6
echo "a north-facing source leaves the +x line empty"
setblock "4 101 -6 minecraft:air"
ctl wait 4
setblock "4 100 -6 minecraft:air"

# An unpowered source builds nothing.
setblock "4 100 10 laserbridges:laser_source_fullblock[face=wall,facing=east,color=0,powered=false]"
ctl wait 4
expect_block 4 100 10 laserbridges:laser_source_fullblock powered=false
expect_air 5 100 10
expect_air 10 100 10
echo "an unpowered source builds nothing"

# Stone stops the line where it stands.
setblock "4 100 14 laserbridges:laser_source_fullblock[face=wall,facing=east,color=0,powered=false]"
setblock "9 100 14 minecraft:stone"
setblock "4 101 14 minecraft:redstone_block"
ctl wait 4
expect_block 5 100 14 laserbridges:laser_block_powered
expect_block 8 100 14 laserbridges:laser_block_powered
expect_block 9 100 14 minecraft:stone
expect_air 10 100 14
expect_air 18 100 14
echo "the stone at x=9 stopped the beam; the cells past it stay empty"

# Breaking the source clears the beam it had.
setblock "4 100 14 minecraft:air"
ctl wait 4
expect_air 5 100 14
expect_air 8 100 14
echo "breaking the source cleared its beam"

echo "== F: every declared state is accepted =="
states=0
for face in floor wall ceiling; do
    for facing in north south east west; do
        for powered in false true; do
            for block in laser_source_block laser_source_fullblock laser_fence_source_block laser_fence_source_fullblock; do
                expect_setblock "20 100 -14 laserbridges:$block[face=$face,facing=$facing,color=0,powered=$powered]"
                states=$((states + 1))
            done
        done
        for block in laser_block_powered laser_fence_powered; do
            expect_setblock "20 100 -14 laserbridges:$block[face=$face,facing=$facing,color=0]"
            states=$((states + 1))
        done
    done
done
reply=$(ctl command "execute in minecraft:the_end run setblock 20 100 -14 laserbridges:laser_source_block[face=wall,facing=north,color=15,powered=false]")
echo "colour-15 setblock: $reply"
echo "immediately after: $(block_json 20 100 -14)"
ctl wait 4
echo "four ticks later: $(block_json 20 100 -14)"
echo "above the source: $(block_json 20 101 -14)"
expect_block 20 100 -14 laserbridges:laser_source_block color=15
echo "$states declared states accepted, and a colour-15 source binds"
expect_setblock_refused "20 100 -14 laserbridges:laser_source_block[face=wall,facing=north,color=16,powered=false]"
expect_setblock_refused "20 100 -14 laserbridges:laser_block_powered[face=wall,facing=north,color=0,powered=true]"
expect_setblock_refused "20 100 -14 minecraft:stone[face=floor]"

echo "== G: the source items exist and the beam blocks have none =="
setblock "20 100 -14 minecraft:air"
ctl give laserbridges:laser_source_block 1 >/dev/null
ctl give laserbridges:laser_fence_source_fullblock 1 >/dev/null
ctl wait 6
ctl inventory | jq -e '[.hotbar[], .main[]] | map(select(.item == "laserbridges:laser_source_block" or .item == "laserbridges:laser_fence_source_fullblock")) | length >= 2' >/dev/null \
    || fail "the source items were not given: $(ctl inventory | jq -c '{hotbar, main}')"
beam_give=$(ctl give laserbridges:laser_block_powered 1 2>&1 || true)
echo "give of a beam block: $beam_give"
ctl wait 4
ctl inventory | jq -e '[.hotbar[], .main[]] | map(select(.item == "laserbridges:laser_block_powered")) | length == 0' >/dev/null \
    || fail "the beam block has an item"
echo "the beam blocks have no item"

# Keep the content warnings for the offline check.
if [ -f "$SCRATCH/.horizon-cache/mod-content/horizon-mod-content-warnings.txt" ]; then
    cp "$SCRATCH/.horizon-cache/mod-content/horizon-mod-content-warnings.txt" \
        /tmp/opencode/laserbridges-content-warnings.txt
fi
if [ -f "$SCRATCH/.horizon-cache/mod-content/model-states.json" ]; then
    cp "$SCRATCH/.horizon-cache/mod-content/model-states.json" \
        /tmp/opencode/laserbridges-model-states.json
fi

ctl quit >/dev/null 2>&1 || true
trap - EXIT
sleep 1
rm -rf "$SCRATCH" 2>/dev/null || true
echo "PASS: a placed source takes its face and facing from the click, a full signal fills 14 beam cells and a player stands on them at 100.5 while the unpowered floor is 100.0, removing the redstone clears every cell and re-powering rebuilds them, a signal of 7 builds 6 cells, a floor fence source builds a 14-cell vertical door that opens when unpowered, a north-facing source leaves the +x line empty, an unpowered source builds nothing, a stone block stops the line, breaking the source clears its beam, 120 declared states bind with color=16 and beam-block powered refused, and the four source items exist while the beam blocks have none"
echo "EXIT:0"
