//! Clipboard module – reads text from the system clipboard (cross-platform via arboard).

use arboard::Clipboard;

/// Attempt to read text from the clipboard.
/// Returns `Some(text)` if the clipboard contains text, `None` otherwise.
pub fn get_clipboard_text() -> Option<String> {
    let mut clipboard = Clipboard::new().ok()?;
    match clipboard.get_text() {
        Ok(text) if !text.is_empty() => Some(text),
        _ => None,
    }
}
