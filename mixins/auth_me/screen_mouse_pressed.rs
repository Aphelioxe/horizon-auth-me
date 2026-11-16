fn auth_me_mouse_pressed(
    game: &crate::game::Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    crate::game::native_auth_me::mouse_pressed(game, screen, button, canceled);
}
