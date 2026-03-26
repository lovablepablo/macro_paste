//! macOS keystroke simulation via Core Graphics CGEvent API.
//!
//! Uses CGEventCreateKeyboardEvent and CGEventKeyboardSetUnicodeString
//! to simulate key presses. Requires Accessibility permission
//! (System Settings > Privacy & Security > Accessibility).

use core_graphics::event::{CGEvent, CGEventTapLocation};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

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
