fn auth_me_screen_render(
    game: &crate::game::Game<'_>,
    batch: &mut HudBatch,
    screen: &NativeScreenView<'_>,
) {
    crate::game::native_auth_me::render_post(game, batch, screen);
}
