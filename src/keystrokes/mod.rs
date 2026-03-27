//! Keystroke simulation module – delegates to platform-specific implementations.
//!
//! Sends clipboard text as individual key events, simulating real keyboard input.
//! This is the core functionality that makes remote desktop password pasting work.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

use std::thread;
use std::time::Duration;

/// Wait until all modifier keys are released before starting to type.
pub fn wait_for_modifiers_released() {
    #[cfg(target_os = "windows")]
    windows::wait_for_modifiers_released();

    #[cfg(target_os = "macos")]
    macos::wait_for_modifiers_released();
}

/// Send a string as individual keystrokes with the given delay between each character.
/// Delegates to the platform-specific implementation for the actual key simulation.
pub fn send_string_as_keystrokes(text: &str, delay_ms: u64) {
    let delay = Duration::from_millis(delay_ms);

    for ch in text.chars() {
        match ch {
            '\r' => continue, // Skip carriage return (handled via \n)
            '\n' => send_special_key(SpecialKey::Enter),
            '\t' => send_special_key(SpecialKey::Tab),
            _ => send_unicode_char(ch),
        }
        thread::sleep(delay);
    }
}

/// Special keys that need virtual key codes instead of Unicode input
enum SpecialKey {
    Enter,
    Tab,
}

/// Send a single Unicode character as a keystroke (platform-specific)
fn send_unicode_char(ch: char) {
    #[cfg(target_os = "windows")]
    windows::send_unicode_char(ch);

    #[cfg(target_os = "macos")]
    macos::send_unicode_char(ch);
}

/// Send a special key press (platform-specific)
fn send_special_key(key: SpecialKey) {
    #[cfg(target_os = "windows")]
    {
        let vk = match key {
            SpecialKey::Enter => 0x0D, // VK_RETURN
            SpecialKey::Tab => 0x09,   // VK_TAB
        };
        windows::send_virtual_key(vk);
    }

    #[cfg(target_os = "macos")]
    {
        let key_code = match key {
            SpecialKey::Enter => 0x24, // kVK_Return
            SpecialKey::Tab => 0x30,   // kVK_Tab
        };
        macos::send_key_code(key_code);
    }
}
