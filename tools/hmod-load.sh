#!/usr/bin/env bash
# Proves the built content package (content/laserbridges.hmod) loads on a real
# game: the shared, unmodded 1d63e0a5 game with `--dev-mod <the .hmod>` must
# register the four sources and the two lasers with their declared states. A
# second launch WITHOUT `--dev-mod` is the negative control: the same ids must be
# unknown there, so the check cannot pass vacuously.
set -euo pipefail
export LC_ALL=C

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PIN_BIN/horizon-client}
HMOD=${HMOD:-$CLONE/ports/laserbridges/content/laserbridges.hmod}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/laserbridges-hmod.sock}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -f "$HMOD" ] || { echo "missing package: $HMOD" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/laserbridges-hmod-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

cleanup() {
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "${KEEP:-0}" = 1 ]; then echo "kept scratch: $SCRATCH" >&2; else rm -rf "$SCRATCH" 2>/dev/null || true; fi
}
trap cleanup EXIT

launch() { # launch WORLD [--dev-mod PATH]
    local world=$1; shift
    rm -f "$SOCK"
    cd "$SCRATCH"
    "$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$world" --seed 20261105 --creative --headless --wait \
        --resources "$RES" "$@" >/dev/null
    ctl wait-ready 120
    ctl wait 20
}

SOURCES="laser_source_block laser_fence_source_block laser_source_fullblock laser_fence_source_fullblock"
LASERS="laser_block_powered laser_fence_powered"

echo "== launch WITH the package (--dev-mod $HMOD) =="
launch LaserBridgesHmod --dev-mod "$HMOD"
for block in $SOURCES; do
    out=$(ctl command "setblock 0 64 0 laserbridges:$block[face=wall,facing=east,color=0,powered=false]")
    echo "setblock $block: $out"
    grep -q "Changed the block" <<<"$out" || fail "the package did not register $block"
    ctl wait 2
    state=$(ctl block 0 64 0 | jq -r '.state // empty')
    echo "block 0,64,0 = $state"
    grep -q "laserbridges:$block" <<<"$state" || fail "$block did not bind"
    grep -q "face=wall" <<<"$state" || fail "$block lost face=wall"
    grep -q "facing=east" <<<"$state" || fail "$block lost facing=east"
done
for block in $LASERS; do
    out=$(ctl command "setblock 0 64 0 laserbridges:$block[face=floor,facing=north,color=3]")
    echo "setblock $block: $out"
    grep -q "Changed the block" <<<"$out" || fail "the package did not register $block"
    ctl wait 2
    state=$(ctl block 0 64 0 | jq -r '.state // empty')
    echo "block 0,64,0 = $state"
    grep -q "color=3" <<<"$state" || fail "$block lost color=3"
done
out=$(ctl command "setblock 0 64 0 laserbridges:laser_source_block[face=wall,facing=north,color=16,powered=false]")
echo "colour 16 (must be refused): $out"
grep -q "Changed the block" <<<"$out" && fail "the package accepted color=16"
out=$(ctl command "setblock 0 64 0 laserbridges:laser_block_powered[face=floor,facing=north,color=0,powered=true]")
echo "powered on a beam block (must be refused): $out"
grep -q "Changed the block" <<<"$out" && fail "a beam block accepted a powered state"
ctl give laserbridges:laser_source_block 1 >/dev/null
ctl wait 4
ctl inventory | jq -e '[.hotbar[], .main[]] | map(select(.item == "laserbridges:laser_source_block")) | length >= 1' >/dev/null \
    || fail "the package did not register the source item: $(ctl inventory | jq -c '{hotbar, main}')"
echo "the source item is registered"
ctl quit >/dev/null 2>&1 || true
sleep 2

echo "== negative control: the SAME ids without --dev-mod =="
launch LaserBridgesHmodControl
out=$(ctl command "setblock 0 64 0 laserbridges:laser_source_block[face=wall,facing=north,color=0,powered=false]")
echo "setblock reply: $out"
if grep -q "Changed the block" <<<"$out"; then
    fail "the unmodded game accepted laserbridges:laser_source_block (the check is vacuous)"
fi
state=$(ctl block 0 64 0 | jq -r '.state // empty')
echo "control block 0,64,0 = $state"
grep -q "laserbridges:" <<<"$state" && fail "the control block is a laserbridges block"
give=$(ctl give laserbridges:laser_source_block 1 2>&1 || true)
echo "give reply: $give"
ctl wait 4
ctl inventory | jq -e '[.hotbar[], .main[]] | map(select(.item == "laserbridges:laser_source_block")) | length == 0' >/dev/null \
    || fail "the unmodded game gave the source item"
echo "the control game knows neither the block nor the item"

ctl quit >/dev/null 2>&1 || true
trap - EXIT
sleep 1
rm -rf "$SCRATCH" 2>/dev/null || true
echo "PASS: the package registers the four sources and two lasers with their declared states, refuses color=16 and a powered beam block, and gives the source item; the same ids are unknown without it"
