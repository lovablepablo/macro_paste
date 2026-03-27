//! macOS keystroke simulation via Core Graphics CGEvent API.
//!
//! Uses CGEventCreateKeyboardEvent and CGEventKeyboardSetUnicodeString
//! to simulate key presses. Requires Accessibility permission
//! (System Settings > Privacy & Security > Accessibility).

use core_graphics::event::{CGEvent, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

/// Wait until all modifier keys (Cmd, Shift, Ctrl, Option) are released,
/// then add a small buffer so the OS can finish processing the key-up events.
/// Times out after 3 seconds to avoid hanging indefinitely.
pub fn wait_for_modifiers_released() {
    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        // kCGEventSourceStateHIDSystemState = 1
        fn CGEventSourceKeyState(stateID: i32, key: u16) -> bool;
    }

    const HID_SYSTEM_STATE: i32 = 1;
    const MODIFIER_KEYCODES: &[u16] = &[
        0x37, // kVK_Command (left)
        0x36, // kVK_RightCommand
        0x38, // kVK_Shift (left)
        0x3C, // kVK_RightShift
        0x3B, // kVK_Control (left)
        0x3E, // kVK_RightControl
        0x3A, // kVK_Option (left)
        0x3D, // kVK_RightOption
    ];

    let timeout = std::time::Instant::now() + std::time::Duration::from_secs(3);

    loop {
        let all_released = MODIFIER_KEYCODES
            .iter()
            .all(|&kc| !unsafe { CGEventSourceKeyState(HID_SYSTEM_STATE, kc) });

        if all_released || std::time::Instant::now() > timeout {
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Small buffer after release so the OS can finish processing the key-up events
    std::thread::sleep(std::time::Duration::from_millis(50));
}

/// Send a single Unicode character as a keystroke via CGEvent.
/// Silently fails if permissions are missing or event creation fails.
pub fn send_unicode_char(ch: char) {
    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Failed to create CGEventSource – check Accessibility permissions");
            return;
        }
    };

    // Create a dummy key event (keycode 0), then override with Unicode string
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), 0, true) {
        // Set the Unicode character on the event
        let mut utf16_buf = [0u16; 2];
        let utf16_units = ch.encode_utf16(&mut utf16_buf);
        event.set_string_from_utf16_unchecked(utf16_units);

        // Post key down
        event.post(CGEventTapLocation::HID);

        // Post key up
        if let Ok(up_event) = CGEvent::new_keyboard_event(source, 0, false) {
            up_event.post(CGEventTapLocation::HID);
        }
    }
}

/// Send a macOS virtual key code as key-down + key-up (for Enter, Tab, etc.).
/// Silently fails if permissions are missing or event creation fails.
pub fn send_key_code(keycode: u16) {
    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Failed to create CGEventSource – check Accessibility permissions");
            return;
        }
    };

    // Key down
    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), keycode, true) {
        event.post(CGEventTapLocation::HID);
    }

    // Key up
    if let Ok(event) = CGEvent::new_keyboard_event(source, keycode, false) {
        event.post(CGEventTapLocation::HID);
    }
}
