//! Windows keystroke simulation via SendInput API.
//!
//! Uses KEYEVENTF_UNICODE to send each character directly as a Unicode scan code,
//! which avoids any keyboard layout issues and correctly handles special characters
//! like umlauts (äöü), symbols (@€), etc.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
    KEYEVENTF_KEYUP, KEYEVENTF_UNICODE, VIRTUAL_KEY,
};

/// Wait until all modifier keys (Ctrl, Shift, Alt, Win) are released,
/// then add a small buffer so the OS can process the key-up events.
/// Times out after 3 seconds to avoid hanging indefinitely.
pub fn wait_for_modifiers_released() {
    const MODIFIER_KEYS: &[i32] = &[
        0x10, // VK_SHIFT
        0x11, // VK_CONTROL
        0x12, // VK_MENU (Alt)
        0x5B, // VK_LWIN
        0x5C, // VK_RWIN
    ];

    let timeout = std::time::Instant::now() + std::time::Duration::from_secs(3);

    loop {
        let all_released = MODIFIER_KEYS
            .iter()
            .all(|&vk| (unsafe { GetAsyncKeyState(vk) } as u16) & 0x8000 == 0);

        if all_released || std::time::Instant::now() > timeout {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Small buffer after release so the OS can finish processing the key-up events
    std::thread::sleep(std::time::Duration::from_millis(50));
}

/// Send a single Unicode character as key-down + key-up via SendInput
pub fn send_unicode_char(ch: char) {
    // UTF-16 encode the character (may produce 1 or 2 surrogates)
    let mut utf16_buf = [0u16; 2];
    let utf16_units = ch.encode_utf16(&mut utf16_buf);

    for &scan_code in utf16_units.iter() {
        let key_down = create_unicode_input(scan_code, KEYEVENTF_UNICODE);
        let key_up = create_unicode_input(scan_code, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);

        let inputs = [key_down, key_up];
        unsafe {
            SendInput(&inputs, size_of::<INPUT>() as i32);
        }
    }
}

/// Send a virtual key code as key-down + key-up (for Enter, Tab, etc.)
pub fn send_virtual_key(vk: u16) {
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
