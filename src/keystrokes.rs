//! Keystroke simulation module – sends text as individual key events via SendInput.
//!
//! Uses KEYEVENTF_UNICODE to send each character directly as a Unicode scan code,
//! which avoids any keyboard layout issues and correctly handles special characters
//! like umlauts (äöü), symbols (@€), etc.

use std::thread;
use std::time::Duration;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

/// Send a string as individual keystrokes with the given delay between each character.
/// Each character is sent as a key-down + key-up pair using Unicode scan codes.
pub fn send_string_as_keystrokes(text: &str, delay_ms: u64) {
    let delay = Duration::from_millis(delay_ms);

    for ch in text.chars() {
        // Handle newlines: send Enter key (VK_RETURN = 0x0D)
        if ch == '\n' {
            send_virtual_key(0x0D);
        } else if ch == '\r' {
            // Skip carriage return (Windows line endings are \r\n, we handle \n)
            continue;
        } else if ch == '\t' {
            // Tab key (VK_TAB = 0x09)
            send_virtual_key(0x09);
        } else {
            // Send as Unicode character – works for all printable chars including special ones
            send_unicode_char(ch);
        }

        thread::sleep(delay);
    }
}

/// Send a single Unicode character as key-down + key-up via SendInput
fn send_unicode_char(ch: char) {
    // UTF-16 encode the character (may produce 1 or 2 surrogates)
    let mut utf16_buf = [0u16; 2];
    let utf16_units = ch.encode_utf16(&mut utf16_buf);

    for &scan_code in utf16_units.iter() {
        // Key down event
        let key_down = create_unicode_input(scan_code, KEYEVENTF_UNICODE);
        // Key up event
        let key_up = create_unicode_input(scan_code, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);

        // Send both events together for atomicity
        let inputs = [key_down, key_up];
        unsafe {
            SendInput(&inputs, size_of::<INPUT>() as i32);
        }
    }
}

/// Send a virtual key code as key-down + key-up (for Enter, Tab, etc.)
fn send_virtual_key(vk: u16) {
    let key_down = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: KEYBD_EVENT_FLAGS(0),
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let key_up = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: 0,
                dwFlags: KEYEVENTF_KEYUP,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let inputs = [key_down, key_up];
    unsafe {
        SendInput(&inputs, size_of::<INPUT>() as i32);
    }
}

/// Helper: create an INPUT struct for a Unicode scan code with the given flags
fn create_unicode_input(scan_code: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0), // Must be 0 for Unicode input
                wScan: scan_code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
