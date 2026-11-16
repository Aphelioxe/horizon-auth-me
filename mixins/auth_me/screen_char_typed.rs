fn auth_me_char_typed(screen: &NativeScreenView<'_>, c: char, canceled: &mut bool) {
    crate::game::native_auth_me::char_typed(screen, c, canceled);
}
