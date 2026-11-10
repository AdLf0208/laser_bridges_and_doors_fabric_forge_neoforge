// After the level tick: apply the sources and clears queued this tick.
fn on_level_tick_post(game: &mut crate::game::Game<'_>, dimension: &str) {
    crate::game::laserbridges_state::on_level_tick_post(game, dimension);
}
