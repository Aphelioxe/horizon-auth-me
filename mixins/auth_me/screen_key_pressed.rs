fn auth_me_key_pressed(
    game: &crate::game::Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    modifiers: u32,
    canceled: &mut bool,
) {
    crate::game::native_auth_me::key_pressed(game, screen, key, modifiers, canceled);
}
