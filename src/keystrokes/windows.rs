//! Windows keystroke simulation via SendInput API.
//!
//! Uses VkKeyScanW + scan codes for RDP compatibility: Microsoft Remote Desktop
//! only forwards physical scan codes, not KEYEVENTF_UNICODE events. Every INPUT
//! event carries both the virtual key code AND the scan code so that RDP's
//! low-level keyboard hook sees complete events and forwards them correctly.

use windows::Win32::UI::Input::KeyboardAndMouse::{
    GetAsyncKeyState, MapVirtualKeyW, SendInput, VkKeyScanW, INPUT, INPUT_0, INPUT_KEYBOARD,
    KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, KEYEVENTF_SCANCODE, KEYEVENTF_UNICODE,
    MAPVK_VK_TO_VSC, VIRTUAL_KEY, VK_CONTROL, VK_MENU, VK_SHIFT,
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

/// Send a single Unicode character as key-down + key-up via SendInput.
///
/// Uses VkKeyScanW to map the character to a virtual key + scan code pair so
/// the input is forwarded correctly through Microsoft Remote Desktop. Characters
/// that have no mapping on the current keyboard layout (e.g. Asian scripts) fall
/// back to KEYEVENTF_UNICODE which works locally but not through RDP.
pub fn send_unicode_char(ch: char) {
    // Characters outside the BMP cannot be mapped via VkKeyScanW
    if ch as u32 <= 0xFFFF {
        let vk_scan: i16 = unsafe { VkKeyScanW(ch as u16) };

        // -1 means no mapping exists on the current keyboard layout
        if vk_scan != -1 {
            let vk = (vk_scan as u16) & 0xFF;
            let shift_state = ((vk_scan as u16) >> 8) & 0xFF;
            let scan_code = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;

            if scan_code != 0 {
                // Press required modifiers (each with VK + scan code for RDP)
                if shift_state & 1 != 0 { send_vk_raw(VK_SHIFT.0, false); }
                if shift_state & 2 != 0 { send_vk_raw(VK_CONTROL.0, false); }
                if shift_state & 4 != 0 { send_vk_raw(VK_MENU.0, false); }

                // Small delay so RDP processes modifier events before the character
                if shift_state != 0 {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }

                // Send the physical key with both VK and scan code
                send_key_raw(vk, scan_code, false);
                send_key_raw(vk, scan_code, true);

                // Small delay before releasing modifiers
                if shift_state != 0 {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }

                // Release modifiers in reverse order
                if shift_state & 4 != 0 { send_vk_raw(VK_MENU.0, true); }
                if shift_state & 2 != 0 { send_vk_raw(VK_CONTROL.0, true); }
                if shift_state & 1 != 0 { send_vk_raw(VK_SHIFT.0, true); }
                return;
            }
        }
    }

    // Fallback: Unicode event (works locally, but not through RDP)
    let mut utf16_buf = [0u16; 2];
    let utf16_units = ch.encode_utf16(&mut utf16_buf);
    for &code_unit in utf16_units.iter() {
        let key_down = create_unicode_input(code_unit, KEYEVENTF_UNICODE);
        let key_up = create_unicode_input(code_unit, KEYEVENTF_UNICODE | KEYEVENTF_KEYUP);
        unsafe { SendInput(&[key_down, key_up], size_of::<INPUT>() as i32); }
    }
}

/// Send a virtual key code as key-down + key-up (for Enter, Tab, etc.)
/// Automatically includes the scan code for RDP compatibility.
pub fn send_virtual_key(vk: u16) {
    send_vk_raw(vk, false);
    send_vk_raw(vk, true);
}

/// Send a single key event with both VK and scan code set.
/// The scan code is looked up automatically via MapVirtualKeyW.
/// Used for modifier keys and special keys (Enter, Tab, etc.).
fn send_vk_raw(vk: u16, key_up: bool) {
    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) } as u16;
    let flags = if key_up { KEYEVENTF_KEYUP } else { KEYBD_EVENT_FLAGS(0) };
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32); }
}

/// Send a single key event with explicit VK + scan code + KEYEVENTF_SCANCODE flag.
/// Used for character keys where both values are already known.
fn send_key_raw(vk: u16, scan_code: u16, key_up: bool) {
    let flags = if key_up {
        KEYEVENTF_SCANCODE | KEYEVENTF_KEYUP
    } else {
        KEYEVENTF_SCANCODE
    };
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk),
                wScan: scan_code,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };
    unsafe { SendInput(&[input], size_of::<INPUT>() as i32); }
}

/// Helper: create an INPUT struct for a Unicode code unit with the given flags
fn create_unicode_input(code_unit: u16, flags: KEYBD_EVENT_FLAGS) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(0),
                wScan: code_unit,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}
