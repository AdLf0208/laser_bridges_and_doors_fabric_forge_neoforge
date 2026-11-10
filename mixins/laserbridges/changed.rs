// A block was placed, replaced or removed: evaluate a new source, or clear the
// beam a removed one left behind.
fn on_block_changed(
    level: &crate::game::GameLevel<'_>,
    dimension: LevelDimension,
    pos: BlockPos,
    old: BlockStateId,
    new: BlockStateId,
) {
    crate::game::laserbridges_state::on_block_changed(level, dimension, pos, old, new);
}
