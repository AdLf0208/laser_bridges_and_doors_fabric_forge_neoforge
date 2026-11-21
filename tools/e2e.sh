#!/usr/bin/env bash
# In-game check for Laser Bridges & Doors (the laser sources and the beams they write).
#
# This script needs a game the LAUNCHER built with this package installed (mod API 0.2):
#
#   horizon-launcher native-build . --instance <instance> --trust-native-code
#   GAME=<instance>/horizon-client CTL=<matching horizon-ctl> tools/e2e.sh
#
# CONTROL=1 runs the negative leg instead: the same receiver scenario on CONTROL_GAME (the
# same Nightly without the package), which must FAIL at the receiver id, proving the leg is
# not vacuous. SELFTEST=1 / SELFTEST=2 are the in-run counterfactuals (see the tail).
#
# The package is one flat `.hmod`: its `assets/`+`data/` are the carried content and its
# `src/` is the behaviour, so there is no `--dev-mod` and no separate content archive to point
# at.
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
# Slice 2 adds the two receivers: a receiver on the last beam cell powers on, copies the beam's
# colour and lights a lamp on a face it gives to, while a lamp behind it (the face upstream's
# getSignal excludes) stays dark; a receiver facing away, an unpowered source in front and a
# cleared beam each leave it off; the receivers and the attached sources emit light only while
# powered; every one of the twelve carried recipes crafts in a crafting table; the receivers drop
# themselves when broken.
#
# SELFTEST=1 flips the full-power beam assertion and SELFTEST=2 the receiver's `powered` assertion
# (both must FAIL), proving the scenario can fail.
set -euo pipefail
export LC_ALL=C

GAME=${GAME:?set GAME to the launcher-built horizon-client carrying laserbridges}
CONTROL_GAME=${CONTROL_GAME:-}
CTL=${CTL:-horizon-ctl}
PORT_DIR=$(cd "$(dirname "$0")" && pwd)
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
        --game "$1" --world "$2" --seed 20261108 --creative --offscreen --wait \
        ${LAUNCH_EXTRA[@]+"${LAUNCH_EXTRA[@]}"}
    ctl wait-ready 120
}

if [ "${CONTROL:-0}" = 1 ]; then
    [ -n "$CONTROL_GAME" ] || { echo "CONTROL=1 needs CONTROL_GAME" >&2; exit 1; }
    [ -x "$CONTROL_GAME" ] || { echo "missing CONTROL_GAME: $CONTROL_GAME" >&2; exit 1; }
    echo "== control: a game without the package must fail the receiver scenario =="
    launch "$CONTROL_GAME" ControlLaserBridges
    control_reply=$(ctl command "execute in minecraft:the_end run setblock 19 100 24 laserbridges:laser_attached_receiver" 2>&1 || true)
    echo "control setblock: $control_reply"
    case "$control_reply" in
        *[Uu]nknown*|*failed*|*error*|*[Rr]efused*) ;;
        *) fail "a game without the package accepted laserbridges:laser_attached_receiver: $control_reply" ;;
    esac
    # The same assertion the shipped leg makes, on a game whose registry has no receiver:
    # it MUST fail, which is what proves the receiver leg depends on the package.
    expect_block 19 100 24 laserbridges:laser_attached_receiver powered=false
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
            for block in laser_attached_receiver laser_attached_receiver_fence; do
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
echo "$states declared states accepted (168: 8 blocks x face x facing), and a colour-15 source binds"
expect_setblock_refused "20 100 -14 laserbridges:laser_source_block[face=wall,facing=north,color=16,powered=false]"
expect_setblock_refused "20 100 -14 laserbridges:laser_block_powered[face=wall,facing=north,color=0,powered=true]"
expect_setblock_refused "20 100 -14 minecraft:stone[face=floor]"

echo "== H: a receiver on the beam powers and drives a lamp =="
# The beam from the source at 4 runs east along z=24; the receiver at 19 looks west
# into its last cell (18). A lamp above the receiver reads the signal on a face that
# is not the one it hangs from; a lamp behind it reads the excluded face and stays off.
setblock "4 100 24 laserbridges:laser_source_block[face=wall,facing=east,color=0,powered=false]"
setblock "19 100 24 laserbridges:laser_attached_receiver[face=wall,facing=west,color=0,powered=false]"
setblock "19 101 24 minecraft:redstone_lamp"
setblock "20 100 24 minecraft:redstone_lamp"
ctl wait 6
expect_block 4 100 24 laserbridges:laser_source_block powered=false
expect_block 19 100 24 laserbridges:laser_attached_receiver powered=false
expect_block 19 101 24 minecraft:redstone_lamp lit=false
echo "the unpowered receiver gives nothing: the lamp above it is unlit"
setblock "4 101 24 minecraft:redstone_block"
ctl wait 8
expect_block 4 100 24 laserbridges:laser_source_block powered=true
if [ "$SELFTEST" = 2 ]; then
    # Deliberately wrong: the receiver is powered, so this must fail and prove the
    # receiver assertion is not vacuous.
    expect_block 19 100 24 laserbridges:laser_attached_receiver powered=false
fi
expect_block 19 100 24 laserbridges:laser_attached_receiver powered=true
expect_block 18 100 24 laserbridges:laser_block_powered color=0
expect_block 19 101 24 minecraft:redstone_lamp lit=true
expect_block 20 100 24 minecraft:redstone_lamp lit=false
echo "the receiver powered on the beam, its colour is 0, and the lamp above it lights while the one behind stays dark"
setblock "4 101 24 minecraft:air"
ctl wait 8
expect_block 19 100 24 laserbridges:laser_attached_receiver powered=false
expect_block 19 101 24 minecraft:redstone_lamp lit=false
expect_air 18 100 24
echo "unpowering the source clears the beam and the receiver's signal with it"
# The colour copy: change the colour with the beam off (a colour change under a live
# beam strands the far cells, upstream's own exact-state rule, kept deliberately), then
# power on, so the new beam and the receiver both carry colour 5.
setblock "4 100 24 laserbridges:laser_source_block[face=wall,facing=east,color=5,powered=false]"
setblock "4 101 24 minecraft:redstone_block"
ctl wait 8
expect_block 18 100 24 laserbridges:laser_block_powered color=5
expect_block 19 100 24 laserbridges:laser_attached_receiver color=5
expect_block 19 100 24 laserbridges:laser_attached_receiver powered=true
echo "the receiver copied the beam's colour 5"
ctl command "execute in minecraft:the_end run tp @s 19.5 100 25.5" >/dev/null
ctl wait 3
ctl break 19 100 24
ctl wait 6
expect_air 19 100 24
expect_block 19 101 24 minecraft:redstone_lamp lit=false
echo "breaking the receiver drops its signal"

echo "== I: a receiver facing away from the beam stays off =="
setblock "4 100 22 laserbridges:laser_source_block[face=wall,facing=east,color=0,powered=false]"
setblock "4 101 22 minecraft:redstone_block"
setblock "19 100 22 laserbridges:laser_attached_receiver[face=wall,facing=east,color=0,powered=false]"
setblock "19 101 22 minecraft:redstone_lamp"
ctl wait 8
expect_block 18 100 22 laserbridges:laser_block_powered
expect_block 19 100 22 laserbridges:laser_attached_receiver powered=false
expect_block 19 101 22 minecraft:redstone_lamp lit=false
echo "the beam is one cell behind the receiver, which looks the other way and stays unpowered"
setblock "4 101 22 minecraft:air"

echo "== J: a powered source in front powers a receiver, an unpowered one does not =="
setblock "4 100 26 laserbridges:laser_source_block[face=wall,facing=east,color=0,powered=false]"
setblock "5 100 26 laserbridges:laser_attached_receiver[face=wall,facing=west,color=0,powered=false]"
setblock "5 101 26 minecraft:redstone_lamp"
ctl wait 6
expect_block 5 100 26 laserbridges:laser_attached_receiver powered=false
expect_block 5 101 26 minecraft:redstone_lamp lit=false
echo "an unpowered source in front does not power the receiver"
setblock "4 101 26 minecraft:redstone_block"
ctl wait 8
expect_block 4 100 26 laserbridges:laser_source_block powered=true
expect_block 5 100 26 laserbridges:laser_attached_receiver powered=true
expect_block 5 101 26 minecraft:redstone_lamp lit=true
echo "the same source powered by redstone does"
setblock "4 101 26 minecraft:air"
ctl wait 8
# A receiver gives 15 on every face but the one it hangs from, so the face it looks
# into also signals: with a source in front the pair holds itself on (upstream's own
# `getSignal`, a latch, kept). The way out is to remove the receiver.
expect_block 4 100 26 laserbridges:laser_source_block powered=true
expect_block 5 100 26 laserbridges:laser_attached_receiver powered=true
expect_block 5 101 26 minecraft:redstone_lamp lit=true
echo "with the redstone gone the pair stays on: the receiver signals the source it looks into"
ctl command "execute in minecraft:the_end run tp @s 6.5 100 26.5" >/dev/null
ctl wait 3
ctl break 5 100 26
ctl wait 8
expect_air 5 100 26
expect_block 4 100 26 laserbridges:laser_source_block powered=false
expect_block 5 101 26 minecraft:redstone_lamp lit=false
echo "breaking the receiver breaks the loop: the source loses its 15 and the lamp above it goes out"

echo "== K: the receivers and the attached sources emit light only while powered =="
# Away from every beam: the cell's stored block light is its own emission.
setblock "24 100 -10 laserbridges:laser_source_block[face=wall,facing=east,color=0,powered=false]"
ctl wait 6
dark_source=$(ctl light 24 100 -10 24 100 -10 | jq -r '.block')
echo "unpowered attached source light: $dark_source"
[ "$dark_source" = "0" ] || fail "an unpowered attached source emits light '$dark_source', expected 0"
setblock "24 101 -10 minecraft:redstone_block"
ctl wait 8
lit_source=$(ctl light 24 100 -10 24 100 -10 | jq -r '.block')
echo "powered attached source light: $lit_source"
[ "$lit_source" = "a" ] || fail "a powered attached source emits '$lit_source', expected a (10)"
setblock "24 101 -10 minecraft:air"
# A receiver facing a bare laser block: the beam cell's own light is 10, so a receiver
# that emitted nothing would read 9 from it; 10 is the receiver's own emission.
setblock "25 100 -12 laserbridges:laser_block_powered[face=wall,facing=east,color=0]"
setblock "24 100 -12 laserbridges:laser_attached_receiver[face=wall,facing=east,color=0,powered=false]"
ctl wait 8
expect_block 24 100 -12 laserbridges:laser_attached_receiver powered=true
lit_receiver=$(ctl light 24 100 -12 24 100 -12 | jq -r '.block')
echo "powered receiver light: $lit_receiver"
[ "$lit_receiver" = "a" ] || fail "a powered receiver emits '$lit_receiver', expected a (10)"
setblock "25 100 -12 minecraft:air"
ctl wait 8
expect_block 24 100 -12 laserbridges:laser_attached_receiver powered=false
dark_receiver=$(ctl light 24 100 -12 24 100 -12 | jq -r '.block')
echo "unpowered receiver light: $dark_receiver"
[ "$dark_receiver" = "0" ] || fail "an unpowered receiver emits '$dark_receiver', expected 0"
echo "both blocks are dark unpowered and emit 10 powered, as upstream's lightLevel"

echo "== L: the receivers take their facing from the click and refuse a thin support =="
setblock "1 101 -2 minecraft:stone"
setblock "2 101 -2 minecraft:air"
ctl command "execute in minecraft:the_end run tp @s 3.5 101 -2.5" >/dev/null
ctl wait 2
ctl give laserbridges:laser_attached_receiver 3 >/dev/null
ctl wait 4
receiver_place=$(ctl place 2 101 -2 --item laserbridges:laser_attached_receiver)
echo "receiver place reply: $receiver_place"
ctl wait 4
expect_block 2 101 -2 laserbridges:laser_attached_receiver face=wall facing=west
# A fence post's side face is not a full support, so the wall-mounted receiver is refused
# and the stack is kept; a stone block's side face is, and the same click places it.
# `ctl interact` names the face, which is what makes the thin side reachable (`place`
# targets a cell and resolves to the floor instead).
setblock "9 100 -9 minecraft:air"
setblock "8 101 -9 minecraft:oak_fence"
setblock "9 101 -11 minecraft:air"
setblock "8 101 -11 minecraft:stone"
ctl command "execute in minecraft:the_end run tp @s 10.5 101 -9.5" >/dev/null
ctl wait 2
ctl select laserbridges:laser_attached_receiver >/dev/null
ctl wait 2
refused=$(ctl interact 8 101 -9 east 2>&1 || true)
echo "right-click on a fence side with the receiver: $refused"
ctl wait 4
expect_air 9 101 -9
ctl command "execute in minecraft:the_end run tp @s 10.5 101 -10.5" >/dev/null
ctl wait 2
allowed=$(ctl interact 8 101 -11 east 2>&1 || true)
echo "the same click on a stone side: $allowed"
ctl wait 4
expect_block 9 101 -11 laserbridges:laser_attached_receiver face=wall
ctl inventory | jq -e '[.hotbar[], .main[]] | map(select(.item == "laserbridges:laser_attached_receiver")) | map(.count) | add >= 1' >/dev/null \
    || fail "the refused receiver placement consumed the stack: $(ctl inventory | jq -c '{hotbar, main}')"
echo "the fence refused the mount and kept the stack; the stone accepted it"

echo "== M: every carried recipe crafts in a crafting table =="
setblock "14 100 20 minecraft:air"
python3 "$PORT_DIR/craft_recipes.py" "$SOCK" "$CTL" 14 100 20 || fail "the recipe check failed"
echo "recipes PASS"

echo "== N: a broken receiver and source drop themselves =="
ctl command "gamemode survival @s" >/dev/null
ctl command "execute in minecraft:the_end run tp @s 22.5 100 18.5" >/dev/null
ctl wait 4
setblock "22 100 20 laserbridges:laser_attached_receiver[face=floor,facing=north,color=0,powered=false]"
setblock "24 100 20 laserbridges:laser_source_block[face=floor,facing=north,color=0,powered=false]"
ctl wait 6
ctl break 22 100 20
ctl wait 10
receiver_drops=$(ctl entities 8 item | jq '[.entities[] | select(.kind == "item" and .item.item == "laserbridges:laser_attached_receiver") | .item.count] | add // 0')
receiver_held=$(ctl inventory | jq '[.hotbar[], .main[], .offhand?] | map(select(.item == "laserbridges:laser_attached_receiver")) | map(.count) | add // 0')
echo "receiver drops on the ground: $receiver_drops, in the inventory: $receiver_held"
[ $((receiver_drops + receiver_held)) -ge 1 ] || fail "the broken receiver dropped neither a ground item nor an inventory item"
ctl break 24 100 20
ctl wait 10
source_drops=$(ctl entities 8 item | jq '[.entities[] | select(.kind == "item" and .item.item == "laserbridges:laser_source_block") | .item.count] | add // 0')
source_held=$(ctl inventory | jq '[.hotbar[], .main[], .offhand?] | map(select(.item == "laserbridges:laser_source_block")) | map(.count) | add // 0')
echo "source drops on the ground: $source_drops, in the inventory: $source_held"
[ $((source_drops + source_held)) -ge 1 ] || fail "the broken source dropped neither a ground item nor an inventory item"
ctl command "gamemode creative @s" >/dev/null
ctl wait 4
echo "the carried loot tables drop the receivers and the sources"

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
echo "PASS: a placed source takes its face and facing from the click, a full signal fills 14 beam cells and a player stands on them at 100.5 while the unpowered floor is 100.0, removing the redstone clears every cell and re-powering rebuilds them, a signal of 7 builds 6 cells, a floor fence source builds a 14-cell vertical door that opens when unpowered, a north-facing source leaves the +x line empty, an unpowered source builds nothing, a stone block stops the line, breaking the source clears its beam, 168 declared states over the eight blocks bind with color=16 and beam-block powered refused, the two receivers power on a beam or a powered source and copy its colour while a receiver facing away stays off, a powered receiver drives the lamp above it and not the one behind, breaking one drops its signal, the receivers and attached sources emit light only while powered, all 12 carried recipes craft in a crafting table consuming their ingredients, broken receivers and sources drop themselves, and the four source items exist while the beam blocks have none"
echo "EXIT:0"
