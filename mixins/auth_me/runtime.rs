use std::sync::{Mutex, MutexGuard};

use super::Game;
use horizon_client::hud::HudBatch;
use horizon_client::mod_api::NativeScreenView;

pub const ID: &str = "auth_me";
pub const NAME: &str = "Auth Me (Offline)";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str =
    "Change your in-game player name directly without a Microsoft session.";

const MAX_NAME_LEN: usize = 16;

struct State {
    custom_name: Option<String>,
    initial_captured: bool,
    initial_player_name: Option<String>,
    dirty: bool,
    modal_open: bool,
    input: String,
    error: Option<&'static str>,
}

impl State {
    const fn new() -> Self {
        Self {
            custom_name: None,
            initial_captured: false,
            initial_player_name: None,
            dirty: false,
            modal_open: false,
            input: String::new(),
            error: None,
        }
    }
}

static STATE: Mutex<State> = Mutex::new(State::new());

fn state() -> MutexGuard<'static, State> {
    STATE.lock().unwrap_or_else(|e| e.into_inner())
}

pub fn is_supported_screen(screen_id: &str) -> bool {
    matches!(screen_id, "title" | "multiplayer" | "disconnected")
}

pub fn is_offline_override() -> bool {
    state().custom_name.is_some()
}

pub(super) fn sync_frame(game: &mut Game<'_>) {
    let mut st = state();
    if !st.initial_captured {
        st.initial_player_name = game.options.player_name.clone();
        st.initial_captured = true;
    }
    if !is_supported_screen_from_game(game) && st.modal_open {
        st.modal_open = false;
        st.error = None;
    }
    if st.dirty {
        st.dirty = false;
        if let Some(name) = st.custom_name.clone() {
            game.options.player_name = Some(name);
        } else {
            game.options.player_name = st.initial_player_name.clone();
        }
    }
}

fn is_supported_screen_from_game(game: &Game<'_>) -> bool {
    let Some(title) = game.title.as_ref() else {
        return false;
    };
    is_supported_screen(title.mode.id())
}

fn active_display_name(game: &Game<'_>, st: &State) -> String {
    if let Some(name) = st.custom_name.as_ref() {
        return name.clone();
    }
    if let Some(auth) = horizon_client::auth::current() {
        return auth.name.clone();
    }
    game.options.player_name()
}

fn save_offline_name(name: &str) {
    let path = horizon_client::logging::player_dir().join(horizon_client::offline_name::FILE);
    if let Err(e) = std::fs::write(&path, format!("name:{name}\n")) {
        log::warn!("auth_me: could not save {}: {e}", path.display());
    }
}

fn apply_input(st: &mut State) {
    let trimmed = st.input.trim();
    if trimmed.is_empty() {
        st.error = Some("Name cannot be empty");
        return;
    }
    if !horizon_client::offline_name::is_valid(trimmed) {
        st.error = Some("Use 1-16 chars: A-Z, 0-9, _");
        return;
    }
    let name = trimmed.to_owned();
    save_offline_name(&name);
    st.custom_name = Some(name);
    st.dirty = true;
    st.modal_open = false;
    st.error = None;
}

fn reset_override(st: &mut State) {
    st.custom_name = None;
    st.dirty = true;
    st.modal_open = false;
    st.error = None;
}

#[derive(Clone, Copy)]
struct Layout {
    btn: [f32; 4],
    box_rect: [f32; 4],
    input_rect: [f32; 4],
    apply_btn: [f32; 4],
    reset_btn: [f32; 4],
    cancel_btn: [f32; 4],
}

fn layout(game: &Game<'_>, screen: &NativeScreenView<'_>, label: &str) -> Layout {
    let btn_w = (game.native_text_width(label) + 16.0).max(110.0);
    let btn = [8.0, 8.0, btn_w, 20.0];

    let [sx, sy, sw, sh] = screen.rect;
    let bw = 240.0_f32.min((sw - 20.0).max(180.0));
    let bh = 114.0;
    let bx = sx + ((sw - bw) * 0.5).floor();
    let by = sy + ((sh - bh) * 0.5).floor();
    let box_rect = [bx, by, bw, bh];

    let pad = 12.0;
    let inner_w = bw - pad * 2.0;
    let input_rect = [bx + pad, by + 38.0, inner_w, 20.0];

    let gap = 6.0;
    let btn_y = by + bh - 28.0;
    let third_w = ((inner_w - gap * 2.0) / 3.0).floor();
    let apply_btn = [bx + pad, btn_y, third_w, 18.0];
    let reset_btn = [bx + pad + third_w + gap, btn_y, third_w, 18.0];
    let cancel_btn = [bx + pad + (third_w + gap) * 2.0, btn_y, third_w, 18.0];

    Layout {
        btn,
        box_rect,
        input_rect,
        apply_btn,
        reset_btn,
        cancel_btn,
    }
}

fn hit(rect: [f32; 4], mouse: [f32; 2]) -> bool {
    let [x, y, w, h] = rect;
    mouse[0] >= x && mouse[0] <= x + w && mouse[1] >= y && mouse[1] <= y + h
}

fn draw_button(
    game: &Game<'_>,
    batch: &mut HudBatch,
    rect: [f32; 4],
    label: &str,
    hovered: bool,
    accent_border: u32,
    text_rgb: u32,
) {
    let [x, y, w, h] = rect;
    let border = if hovered { 0xFFFFFFFF } else { accent_border };
    let bg = if hovered { 0xE03A4458 } else { 0xCC1E222B };
    game.native_fill(batch, [x, y, w, h], border);
    game.native_fill(batch, [x + 1.0, y + 1.0, w - 2.0, h - 2.0], bg);
    let tw = game.native_text_width(label);
    let tx = x + ((w - tw) * 0.5).floor();
    let ty = y + ((h - 8.0) * 0.5).floor();
    game.native_text(batch, label, [tx, ty], text_rgb, true);
}

pub(super) fn render_post(game: &Game<'_>, batch: &mut HudBatch, screen: &NativeScreenView<'_>) {
    if !is_supported_screen(screen.id) {
        return;
    }
    let st = state();
    let display_name = active_display_name(game, &st);
    let is_offline = st.custom_name.is_some();
    let label = if is_offline {
        format!("Name: {display_name} [Offline]")
    } else {
        format!("Name: {display_name}")
    };
    let lay = layout(game, screen, &label);

    let btn_border = if is_offline { 0xFF55FF55 } else { 0xFFAAAAAA };
    let btn_rgb = if is_offline { 0xAAFFAA } else { 0xFFFFFF };
    draw_button(
        game,
        batch,
        lay.btn,
        &label,
        hit(lay.btn, screen.mouse),
        btn_border,
        btn_rgb,
    );

    if !st.modal_open {
        return;
    }

    let [sx, sy, sw, sh] = screen.rect;
    game.native_fill(batch, [sx, sy, sw, sh], 0x99000000);

    let [bx, by, bw, bh] = lay.box_rect;
    game.native_fill(batch, [bx, by, bw, bh], 0xFFFFFFFF);
    game.native_fill(batch, [bx + 1.0, by + 1.0, bw - 2.0, bh - 2.0], 0xF0181A20);

    game.native_text(
        batch,
        "Auth Me - Offline Username",
        [bx + 12.0, by + 10.0],
        0xFFFFFF,
        true,
    );
    let mode_text = if is_offline {
        format!("Active: {display_name} (Offline)")
    } else {
        format!("Active: {display_name} (Default/MS)")
    };
    game.native_text(batch, &mode_text, [bx + 12.0, by + 23.0], 0xAAAAAA, true);

    let [ix, iy, iw, ih] = lay.input_rect;
    game.native_fill(batch, [ix, iy, iw, ih], 0xFFA0A0A0);
    game.native_fill(batch, [ix + 1.0, iy + 1.0, iw - 2.0, ih - 2.0], 0xFF000000);

    let caret = if (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        / 500)
        % 2
        == 0
    {
        "_"
    } else {
        ""
    };
    let shown_input = format!("{}{}", st.input, caret);
    game.native_text(batch, &shown_input, [ix + 5.0, iy + 6.0], 0xFFFFFF, false);

    if let Some(err) = st.error {
        game.native_text(batch, err, [bx + 12.0, by + 63.0], 0xFF5555, true);
    } else {
        game.native_text(
            batch,
            "Enter to apply, Esc to cancel",
            [bx + 12.0, by + 63.0],
            0x888888,
            true,
        );
    }

    draw_button(
        game,
        batch,
        lay.apply_btn,
        "Apply",
        hit(lay.apply_btn, screen.mouse),
        0xFF55AA55,
        0xAAFFAA,
    );
    draw_button(
        game,
        batch,
        lay.reset_btn,
        "Reset",
        hit(lay.reset_btn, screen.mouse),
        0xFFAA8844,
        0xFFDD88,
    );
    draw_button(
        game,
        batch,
        lay.cancel_btn,
        "Cancel",
        hit(lay.cancel_btn, screen.mouse),
        0xFF888888,
        0xDDDDDD,
    );
}

pub(super) fn mouse_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    canceled: &mut bool,
) {
    if !is_supported_screen(screen.id) || button != 0 {
        return;
    }
    let mut st = state();
    let display_name = active_display_name(game, &st);
    let label = if st.custom_name.is_some() {
        format!("Name: {display_name} [Offline]")
    } else {
        format!("Name: {display_name}")
    };
    let lay = layout(game, screen, &label);

    if !st.modal_open {
        if hit(lay.btn, screen.mouse) {
            st.input = display_name;
            st.error = None;
            st.modal_open = true;
            *canceled = true;
        }
        return;
    }

    *canceled = true;
    if hit(lay.apply_btn, screen.mouse) {
        apply_input(&mut st);
    } else if hit(lay.reset_btn, screen.mouse) {
        reset_override(&mut st);
    } else if hit(lay.cancel_btn, screen.mouse) || !hit(lay.box_rect, screen.mouse) {
        st.modal_open = false;
        st.error = None;
    }
}

pub fn mouse_released(screen: &NativeScreenView<'_>, canceled: &mut bool) {
    if !is_supported_screen(screen.id) {
        return;
    }
    if state().modal_open {
        *canceled = true;
    }
}

fn key_to_username_char(key: &str, modifiers: u32) -> Option<char> {
    if (modifiers & (2 | 4 | 8)) != 0 {
        return None;
    }
    let shift = (modifiers & 1) != 0;
    if let Some(letter) = key.strip_prefix("key.keyboard.") {
        if letter.len() == 1 {
            let ch = letter.chars().next()?;
            if ch.is_ascii_lowercase() {
                return Some(if shift { ch.to_ascii_uppercase() } else { ch });
            }
            if ch.is_ascii_digit() && !shift {
                return Some(ch);
            }
        }
        if let Some(digit) = letter.strip_prefix("keypad.") {
            if digit.len() == 1 {
                let ch = digit.chars().next()?;
                if ch.is_ascii_digit() {
                    return Some(ch);
                }
            }
        }
        if matches!(letter, "minus" | "slash") && shift {
            return Some('_');
        }
    }
    None
}

pub(super) fn key_pressed(
    _game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    modifiers: u32,
    canceled: &mut bool,
) {
    if !is_supported_screen(screen.id) {
        return;
    }
    let mut st = state();
    if !st.modal_open {
        return;
    }
    *canceled = true;
    match key {
        "key.keyboard.escape" => {
            st.modal_open = false;
            st.error = None;
        }
        "key.keyboard.enter" | "key.keyboard.keypad.enter" => {
            apply_input(&mut st);
        }
        "key.keyboard.backspace" => {
            if (modifiers & 2) != 0 {
                st.input.clear();
            } else {
                st.input.pop();
            }
            st.error = None;
        }
        _ => {
            if let Some(ch) = key_to_username_char(key, modifiers)
                && st.input.len() < MAX_NAME_LEN
            {
                st.input.push(ch);
                st.error = None;
            }
        }
    }
}

pub fn char_typed(screen: &NativeScreenView<'_>, _c: char, canceled: &mut bool) {
    if !is_supported_screen(screen.id) {
        return;
    }
    if state().modal_open {
        *canceled = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_screens_match_expected_ids() {
        assert!(is_supported_screen("title"));
        assert!(is_supported_screen("multiplayer"));
        assert!(is_supported_screen("disconnected"));
        assert!(!is_supported_screen("inventory"));
    }

    #[test]
    fn hit_checks_bounds_inclusively() {
        let r = [10.0, 20.0, 100.0, 20.0];
        assert!(hit(r, [10.0, 20.0]));
        assert!(hit(r, [110.0, 40.0]));
        assert!(!hit(r, [9.9, 25.0]));
        assert!(!hit(r, [50.0, 41.0]));
    }

    #[test]
    fn key_mapping_handles_letters_digits_underscore_and_modifiers() {
        assert_eq!(key_to_username_char("key.keyboard.a", 0), Some('a'));
        assert_eq!(key_to_username_char("key.keyboard.a", 1), Some('A'));
        assert_eq!(key_to_username_char("key.keyboard.7", 0), Some('7'));
        assert_eq!(key_to_username_char("key.keyboard.7", 1), None);
        assert_eq!(key_to_username_char("key.keyboard.keypad.4", 0), Some('4'));
        assert_eq!(key_to_username_char("key.keyboard.minus", 1), Some('_'));
        assert_eq!(key_to_username_char("key.keyboard.slash", 1), Some('_'));
        assert_eq!(key_to_username_char("key.keyboard.a", 2), None);
    }
}
