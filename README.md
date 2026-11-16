# Horizon Auth Me (`auth_me`)

Offline account / username switcher native mixin mod for Horizon Client (`26.3`). Adds an **Account** button (`G` shortcut or top-right button on the title screen) allowing you to change your active offline username at runtime without restarting the client.

## What Was Fixed

Horizon's `title_owns_text` only enables `screen_chars_typed` while `Screen::Playing` is active, so deleting characters on the title screen previously left the username field unable to accept new typed characters.

This version fixes text entry directly inside the `screen_key_pressed` hook:
- Handles `a–z` / `A–Z` (Shift), top-row digits `0–9` (when Shift is not held), keypad digits `0–9`, and `_` on both US (`Shift + minus`) and Turkish-Q (`Shift + slash`) keyboards.
- Ignores `Ctrl` / `Alt` / `Super` modified keystrokes so shortcuts do not insert stray characters, and supports `Ctrl+Backspace` to clear the field.
- Supports `Enter` / `Keypad Enter` to apply, `Backspace` to delete, and `Escape` to cancel.

## Installation

1. Download `auth_me-1.0.0.hnmod` from [Releases](https://github.com/Aphelioxe/horizon-auth-me/releases).
2. Place it in your Horizon native mods directory (`%APPDATA%\horizon\horizon-native-mods\`).
3. Launch Horizon Client and trust native mods when prompted.

## Packaging `.hnmod`

`.hnmod` is a zip archive containing `horizon-native.json` and the `mixins/auth_me/` directory at the archive root.

## License

[MIT](LICENSE)
