fn auth_me_frame(game: &mut crate::game::Game<'_>) {
    crate::game::native_auth_me::sync_frame(game);
}
