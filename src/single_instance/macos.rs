//! macOS single instance check via file lock (flock).

use super::InstanceLock;
use std::fs::{self, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

/// Get the lock file path in a user-specific location
fn lock_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    let dir = PathBuf::from(home).join(".macropaste");
    // Ensure directory exists
    let _ = fs::create_dir_all(&dir);
    dir.join("instance.lock")
}

/// Try to acquire an exclusive file lock – exits if another instance holds it
pub fn ensure_single_instance() -> InstanceLock {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path())
        .unwrap_or_else(|_| std::process::exit(0));

    // Try non-blocking exclusive lock
    let fd = file.as_raw_fd();
    let result = unsafe { libc::flock(fd, libc::LOCK_EX | libc::LOCK_NB) };

    if result != 0 {
        // Another instance holds the lock
        std::process::exit(0);
    }

    InstanceLock { _file: file }
}
