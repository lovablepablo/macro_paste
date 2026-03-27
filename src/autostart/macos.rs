//! macOS autostart – manages a Launch Agent plist for launching at login.

use std::fs;
use std::path::PathBuf;

/// Bundle identifier used for the Launch Agent plist filename
const BUNDLE_ID: &str = "io.github.lovablepablo.macropaste";

/// Get the path to the Launch Agent plist file
fn plist_path() -> Result<PathBuf, String> {
    let home = std::env::var("HOME")
        .map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join(format!("{BUNDLE_ID}.plist")))
}

/// Enable autostart by creating a Launch Agent plist
pub fn enable_autostart() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {e}"))?;

    let path = plist_path()?;
    let plist_dir = path.parent()
        .ok_or_else(|| "Invalid plist path".to_string())?;

    // Ensure LaunchAgents directory exists
    fs::create_dir_all(plist_dir)
        .map_err(|e| format!("Failed to create LaunchAgents dir: {e}"))?;

    // Write the plist XML
    let plist_content = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{BUNDLE_ID}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{exe}</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
</dict>
</plist>"#,
        exe = exe_path.to_string_lossy()
    );

    fs::write(&path, plist_content)
        .map_err(|e| format!("Failed to write plist: {e}"))?;

    Ok(())
}

/// Disable autostart by removing the Launch Agent plist
pub fn disable_autostart() -> Result<(), String> {
    let path = plist_path()?;
    if path.exists() {
        fs::remove_file(&path)
            .map_err(|e| format!("Failed to remove plist: {e}"))?;
    }
    Ok(())
}

/// Check if autostart is currently enabled (plist file exists)
pub fn is_autostart_enabled() -> bool {
    plist_path().map(|p| p.exists()).unwrap_or(false)
}
