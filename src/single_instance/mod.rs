//! Single instance module – prevents multiple copies of the app from running.
//! Uses platform-specific locking mechanisms.

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

/// Ensure only one instance of the app is running.
/// Exits silently if another instance is already active.
/// Returns a guard value that must be kept alive for the lock to persist.
pub fn ensure_single_instance() -> InstanceLock {
    #[cfg(target_os = "windows")]
    return windows::ensure_single_instance();

    #[cfg(target_os = "macos")]
    return macos::ensure_single_instance();
}

/// Opaque lock handle – the lock is released when this is dropped
pub struct InstanceLock {
    #[cfg(target_os = "windows")]
    _handle: ::windows::Win32::Foundation::HANDLE,
    #[cfg(target_os = "macos")]
    _file: std::fs::File,
}
