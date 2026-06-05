//! macOS keystroke simulation via Core Graphics CGEvent API.
//!
//! Uses CGEventCreateKeyboardEvent and CGEventKeyboardSetUnicodeString
//! to simulate key presses. Requires Accessibility permission
//! (System Settings > Privacy & Security > Accessibility).

use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::OnceLock;

use core_foundation::string::CFStringRef;
use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation};
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

// --- Carbon / HIToolbox FFI for keyboard-layout-aware character mapping ---
#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISPropertyUnicodeKeyLayoutData: CFStringRef;

    fn TISCopyCurrentKeyboardLayoutInputSource() -> *mut c_void;
    fn TISGetInputSourceProperty(source: *mut c_void, key: CFStringRef) -> *mut c_void;
    fn LMGetKbdType() -> u8;

    #[allow(clippy::too_many_arguments)]
    fn UCKeyTranslate(
        keyLayoutPtr: *const u8,
        virtualKeyCode: u16,
        keyAction: u16,
        modifierKeyState: u32,
        keyboardType: u32,
        keyTranslateOptions: u32,
        deadKeyState: *mut u32,
        maxStringLength: usize,
        actualStringLength: *mut usize,
        unicodeString: *mut u16,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFDataGetBytePtr(data: *const c_void) -> *const u8;
    fn CFRelease(cf: *const c_void);
}

/// A character's keystroke representation: the physical virtual key code plus
/// the modifier flags (Shift / Option) required to produce it on the current
/// keyboard layout.
#[derive(Clone, Copy)]
struct KeyStroke {
    keycode: u16,
    flags: CGEventFlags,
}

/// Reverse keyboard-layout map: character -> physical keystroke.
///
/// Built once from the user's *current* keyboard layout via `UCKeyTranslate`.
/// This is what lets remote desktop clients (RDP, VNC, VMs) receive the correct
/// key. Those clients ignore the Unicode payload of a synthetic event and read
/// the hardware key code instead — without a real key code, keycode 0 maps to
/// "a", so every character would arrive as "aaaa…".
fn keymap() -> &'static HashMap<char, KeyStroke> {
    static MAP: OnceLock<HashMap<char, KeyStroke>> = OnceLock::new();
    MAP.get_or_init(build_keymap)
}

fn build_keymap() -> HashMap<char, KeyStroke> {
    const KEY_ACTION_DOWN: u16 = 0;
    // modifierKeyState passed to UCKeyTranslate is (Carbon event modifiers >> 8).
    // shiftKey (1 << 9) >> 8 = 2, optionKey (1 << 11) >> 8 = 8.
    const MOD_COMBOS: &[(u32, CGEventFlags)] = &[
        (0, CGEventFlags::CGEventFlagNull),
        (2, CGEventFlags::CGEventFlagShift),
        (8, CGEventFlags::CGEventFlagAlternate),
        (
            10,
            CGEventFlags::from_bits_truncate(
                CGEventFlags::CGEventFlagShift.bits() | CGEventFlags::CGEventFlagAlternate.bits(),
            ),
        ),
    ];

    let mut map: HashMap<char, KeyStroke> = HashMap::new();

    unsafe {
        let source = TISCopyCurrentKeyboardLayoutInputSource();
        if source.is_null() {
            return map;
        }

        let layout_data = TISGetInputSourceProperty(source, kTISPropertyUnicodeKeyLayoutData);
        if layout_data.is_null() {
            CFRelease(source);
            return map;
        }

        let layout_ptr = CFDataGetBytePtr(layout_data);
        let kbd_type = LMGetKbdType() as u32;

        // Iterate every physical key with each modifier combination and record
        // which character it produces. First (simplest) combo wins per char.
        for keycode in 0u16..128 {
            for &( uc_mods, cg_flags) in MOD_COMBOS {
                let mut dead_key_state: u32 = 0;
                let mut buf = [0u16; 8];
                let mut len: usize = 0;

                let status = UCKeyTranslate(
                    layout_ptr,
                    keycode,
                    KEY_ACTION_DOWN,
                    uc_mods,
                    kbd_type,
                    0, // kUCKeyTranslateNoDeadKeysBit not set; resolve dead keys
                    &mut dead_key_state,
                    buf.len(),
                    &mut len,
                    buf.as_mut_ptr(),
                );

                if status != 0 || len == 0 {
                    continue;
                }

                if let Some(ch) = String::from_utf16_lossy(&buf[..len]).chars().next() {
                    // Skip control characters; those are handled as special keys.
                    if ch.is_control() {
                        continue;
                    }
                    map.entry(ch).or_insert(KeyStroke {
                        keycode,
                        flags: cg_flags,
                    });
                }
            }
        }

        CFRelease(source);
    }

    map
}

/// Returns true if this process is trusted for Accessibility (the permission
/// required to synthesize keyboard input). macOS silently drops posted key
/// events when this is false, so we check it to give the user feedback.
pub fn is_accessibility_trusted() -> bool {
    #[link(name = "ApplicationServices", kind = "framework")]
    unsafe extern "C" {
        fn AXIsProcessTrusted() -> bool;
    }
    unsafe { AXIsProcessTrusted() }
}

/// Send a single Unicode character as a keystroke via CGEvent.
///
/// Sets the Unicode string on the event — the primary path: native macOS apps
/// and remote desktop clients in *Unicode* keyboard mode read this, so every
/// character (uppercase, umlauts, symbols) is delivered correctly regardless of
/// keyboard layout. Additionally sets a best-effort physical key code +
/// modifier flags from the current layout, so remote desktop clients in
/// *scancode* mode still produce the right key instead of mapping the
/// placeholder key code 0 to "a".
///
/// Note: in scancode mode RDP strips synthesized modifier state, so
/// Shift-dependent characters cannot be produced reliably there. Switch the RDP
/// client to Unicode keyboard mode (Connections → Keyboard Mode → Unicode) for
/// fully correct input — see README. Silently fails if permissions are missing
/// or event creation fails.
pub fn send_unicode_char(ch: char) {
    let source = match CGEventSource::new(CGEventSourceStateID::HIDSystemState) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("Failed to create CGEventSource – check Accessibility permissions");
            return;
        }
    };

    // Best-effort physical key for this character on the current layout; falls
    // back to key code 0 (Unicode-only) for characters with no direct key.
    let stroke = keymap().get(&ch).copied();
    let keycode = stroke.map(|s| s.keycode).unwrap_or(0);
    let flags = stroke
        .map(|s| s.flags)
        .unwrap_or(CGEventFlags::CGEventFlagNull);

    let mut utf16_buf = [0u16; 2];
    let utf16_units = ch.encode_utf16(&mut utf16_buf);

    if let Ok(event) = CGEvent::new_keyboard_event(source.clone(), keycode, true) {
        event.set_flags(flags);
        event.set_string_from_utf16_unchecked(utf16_units);
        event.post(CGEventTapLocation::HID);

        if let Ok(up_event) = CGEvent::new_keyboard_event(source, keycode, false) {
            up_event.set_flags(flags);
            up_event.set_string_from_utf16_unchecked(utf16_units);
            up_event.post(CGEventTapLocation::HID);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keymap_builds_and_is_populated() {
        let map = build_keymap();
        // A US/DE layout must at least produce lowercase ASCII letters.
        println!("keymap entries: {}", map.len());
        for ch in ['a', 'A', '1', '!', '@'] {
            match map.get(&ch) {
                Some(ks) => println!("{ch:?} -> keycode {} flags {:?}", ks.keycode, ks.flags),
                None => println!("{ch:?} -> (none)"),
            }
        }
        assert!(map.contains_key(&'a'), "keymap should contain 'a'");
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
