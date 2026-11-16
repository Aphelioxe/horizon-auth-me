fn auth_me_mouse_released(screen: &NativeScreenView<'_>, canceled: &mut bool) {
    crate::game::native_auth_me::mouse_released(screen, canceled);
}
