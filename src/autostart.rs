//! Autostart module – manages the Windows Registry entry for launching at login.

use winreg::enums::*;
use winreg::RegKey;

/// Registry path for current user autostart entries
const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
/// Name of our app in the registry
const APP_NAME: &str = "MacroPaste";

/// Enable autostart by adding a registry entry pointing to the current executable
pub fn enable_autostart() -> Result<(), String> {
    let exe_path = std::env::current_exe()
        .map_err(|e| format!("Failed to get exe path: {e}"))?;

    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let (key, _) = hkcu
        .create_subkey(RUN_KEY)
        .map_err(|e| format!("Failed to open registry key: {e}"))?;

    key.set_value(APP_NAME, &exe_path.to_string_lossy().to_string())
        .map_err(|e| format!("Failed to set registry value: {e}"))?;

    Ok(())
}

/// Disable autostart by removing the registry entry
pub fn disable_autostart() -> Result<(), String> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let key = hkcu
        .open_subkey_with_flags(RUN_KEY, KEY_WRITE)
        .map_err(|e| format!("Failed to open registry key: {e}"))?;

    // Ignore error if value doesn't exist (already disabled)
    let _ = key.delete_value(APP_NAME);
    Ok(())
}

/// Check if autostart is currently enabled
pub fn is_autostart_enabled() -> bool {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    if let Ok(key) = hkcu.open_subkey_with_flags(RUN_KEY, KEY_READ) {
        let value: Result<String, _> = key.get_value(APP_NAME);
        value.is_ok()
    } else {
        false
    }
}

/// Toggle autostart on/off, returns the new state
pub fn toggle_autostart(enable: bool) -> Result<bool, String> {
    if enable {
        enable_autostart()?;
    } else {
        disable_autostart()?;
    }
    Ok(enable)
}
