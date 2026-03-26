//! Clipboard module – reads text from the Windows clipboard.

use clipboard_win::{formats, get_clipboard};

/// Attempt to read text from the clipboard.
/// Returns `Some(text)` if the clipboard contains text, `None` otherwise.
pub fn get_clipboard_text() -> Option<String> {
    // Try to read Unicode text from clipboard
    let result: Result<String, _> = get_clipboard(formats::Unicode);
    match result {
        Ok(text) if !text.is_empty() => Some(text),
        _ => None,
    }
}
