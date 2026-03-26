//! Windows single instance check via named mutex.

use super::InstanceLock;
use windows::core::w;
use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;
use windows::Win32::System::Threading::CreateMutexW;

/// Create a named mutex – exits if another instance already holds it
pub fn ensure_single_instance() -> InstanceLock {
    let handle = unsafe { CreateMutexW(None, false, w!("Global\\MacroPaste_SingleInstance")) };

    match handle {
        Ok(h) => {
            if unsafe { windows::Win32::Foundation::GetLastError() } == ERROR_ALREADY_EXISTS {
                std::process::exit(0);
            }
            InstanceLock { _handle: h }
        }
        Err(_) => std::process::exit(0),
    }
}
