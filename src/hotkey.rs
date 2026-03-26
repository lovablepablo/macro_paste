//! Hotkey module – registers and manages the global hotkey.

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::GlobalHotKeyManager;

/// Parses a hotkey string like "Ctrl+Shift+V" into a `HotKey` struct.
/// Returns `None` if the string cannot be parsed.
pub fn parse_hotkey(hotkey_str: &str) -> Option<HotKey> {
    let parts: Vec<&str> = hotkey_str.split('+').map(|s| s.trim()).collect();
    let mut modifiers = Modifiers::empty();
    let mut key_code: Option<Code> = None;

    for part in &parts {
        match part.to_uppercase().as_str() {
            "CTRL" | "CONTROL" => modifiers |= Modifiers::CONTROL,
            "SHIFT" => modifiers |= Modifiers::SHIFT,
            "ALT" => modifiers |= Modifiers::ALT,
            "WIN" | "SUPER" | "META" => modifiers |= Modifiers::META,
            key => {
                // Parse the key part (last element, typically a single letter or F-key)
                key_code = parse_key_code(key);
            }
        }
    }

    let code = key_code?;
    Some(HotKey::new(Some(modifiers), code))
}

/// Parse a key name string into a `Code` enum variant
fn parse_key_code(key: &str) -> Option<Code> {
    match key.to_uppercase().as_str() {
        // Letters
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        // Digits
        "0" => Some(Code::Digit0),
        "1" => Some(Code::Digit1),
        "2" => Some(Code::Digit2),
        "3" => Some(Code::Digit3),
        "4" => Some(Code::Digit4),
        "5" => Some(Code::Digit5),
        "6" => Some(Code::Digit6),
        "7" => Some(Code::Digit7),
        "8" => Some(Code::Digit8),
        "9" => Some(Code::Digit9),
        // Function keys
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        _ => None,
    }
}

/// Register a global hotkey and return the manager + hotkey for later use.
/// The manager must be kept alive for the hotkey to remain active.
pub fn register_hotkey(hotkey_str: &str) -> Result<(GlobalHotKeyManager, HotKey), String> {
    let manager = GlobalHotKeyManager::new().map_err(|e| format!("Failed to create hotkey manager: {e}"))?;
    let hotkey = parse_hotkey(hotkey_str).ok_or_else(|| format!("Invalid hotkey: {hotkey_str}"))?;
    manager.register(hotkey).map_err(|e| format!("Failed to register hotkey: {e}"))?;
    Ok((manager, hotkey))
}

/// Unregister an existing hotkey, register a new one, and return the updated hotkey
pub fn update_hotkey(
    manager: &GlobalHotKeyManager,
    old_hotkey: HotKey,
    new_hotkey_str: &str,
) -> Result<HotKey, String> {
    // Unregister old hotkey (ignore errors if it was already unregistered)
    let _ = manager.unregister(old_hotkey);

    let new_hotkey =
        parse_hotkey(new_hotkey_str).ok_or_else(|| format!("Invalid hotkey: {new_hotkey_str}"))?;
    manager
        .register(new_hotkey)
        .map_err(|e| format!("Failed to register hotkey: {e}"))?;
    Ok(new_hotkey)
}
