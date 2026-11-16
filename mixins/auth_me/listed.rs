fn auth_me_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::native_auth_me as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
