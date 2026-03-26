//! Autostart module – manages launching the app at system startup.
//! Delegates to platform-specific implementations.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

/// Enable autostart (adds app to system startup)
pub fn enable_autostart() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return windows::enable_autostart();

    #[cfg(target_os = "macos")]
    return macos::enable_autostart();
}

/// Disable autostart (removes app from system startup)
pub fn disable_autostart() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    return windows::disable_autostart();

    #[cfg(target_os = "macos")]
    return macos::disable_autostart();
}

/// Check if autostart is currently enabled
pub fn is_autostart_enabled() -> bool {
    #[cfg(target_os = "windows")]
    return windows::is_autostart_enabled();

    #[cfg(target_os = "macos")]
    return macos::is_autostart_enabled();
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
