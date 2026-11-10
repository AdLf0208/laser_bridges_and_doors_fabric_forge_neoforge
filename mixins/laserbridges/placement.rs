// `getStateForPlacement`: a source mounts on the clicked face and emits away
// from it.
fn on_state_for_placement(
    level: &crate::game::GameLevel<'_>,
    state: BlockStateId,
    context: &PlacementContext,
    answer: &mut Option<Option<BlockStateId>>,
) {
    crate::game::laserbridges_state::on_state_for_placement(level, state, context, answer);
}
