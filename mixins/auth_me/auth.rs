fn auth_me_auth(
    game: &crate::game::Game<'_>,
) -> ::core::ops::ControlFlow<Option<std::sync::Arc<horizon_client::auth::AuthSession>>> {
    let _ = game;
    if crate::game::native_auth_me::is_offline_override() {
        ::core::ops::ControlFlow::Break(None)
    } else {
        ::core::ops::ControlFlow::Continue(())
    }
}
