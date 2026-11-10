// A neighbour changed: a source's redstone input may have moved, so queue it
// for the next level tick.
fn on_neighbor_changed(
    level: &crate::game::GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    state: BlockStateId,
) {
    crate::game::laserbridges_state::on_neighbor_changed(level, dimension, pos, state);
}
