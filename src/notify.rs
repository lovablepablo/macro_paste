//! Simple cross-platform error notification via native message boxes.

/// Inform the user that the required input permission is missing and offer to
/// open the relevant system settings. macOS only (Accessibility); no-op
/// elsewhere. Runs in a background thread so the modal dialog never blocks the
/// event loop, and de-duplicates so repeated paste attempts don't stack dialogs.
pub fn prompt_missing_permission() {
    #[cfg(target_os = "macos")]
    {
        use std::sync::atomic::{AtomicBool, Ordering};
        static PROMPT_OPEN: AtomicBool = AtomicBool::new(false);

        // Skip if a prompt is already on screen.
        if PROMPT_OPEN.swap(true, Ordering::SeqCst) {
            return;
        }

        std::thread::spawn(|| {
            use std::process::Command;

            const OPEN_BTN: &str = "Open Settings";
            let script = format!(
                "display dialog \"macro_paste needs the Accessibility permission \
                 to send keystrokes.\\n\\nPlease enable macro_paste under:\\n\
                 System Settings → Privacy & Security → Accessibility.\" \
                 with title \"Permission required\" \
                 buttons {{\"Later\", \"{OPEN_BTN}\"}} default button \"{OPEN_BTN}\" \
                 with icon caution"
            );

            let clicked_open = Command::new("osascript")
                .arg("-e")
                .arg(&script)
                .output()
                .map(|out| String::from_utf8_lossy(&out.stdout).contains(OPEN_BTN))
                .unwrap_or(false);

            if clicked_open {
                let _ = Command::new("open")
                    .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
                    .spawn();
            }

            PROMPT_OPEN.store(false, Ordering::SeqCst);
        });
    }
}

/// Show an error message to the user via a native message box.
/// Also logs to stderr as a fallback.
pub fn show_error(title: &str, message: &str) {
    eprintln!("{title}: {message}");

    #[cfg(target_os = "windows")]
    {
        use windows::core::PCWSTR;
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

        let title_wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        let msg_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
        unsafe {
            let _ = MessageBoxW(
                None,
                PCWSTR::from_raw(msg_wide.as_ptr()),
                PCWSTR::from_raw(title_wide.as_ptr()),
                MB_ICONERROR | MB_OK,
            );
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("osascript")
            .arg("-e")
            .arg(format!(
                "display dialog \"{}\" with title \"{}\" buttons {{\"OK\"}} default button \"OK\" with icon stop",
                message.replace('"', "\\\""),
                title.replace('"', "\\\""),
            ))
            .spawn();
    }
}
