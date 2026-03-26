//! Configuration module – loads/saves settings from a platform-specific config file.
//!
//! - macOS: `~/Library/Application Support/macro_paste/config.json`
//! - Windows: next to the executable (portable)

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration stored as JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Hotkey identifier string, e.g. "Ctrl+Shift+V"
    pub hotkey: String,
    /// Delay in milliseconds between each simulated keystroke
    pub delay_ms: u64,
    /// Whether the app should start with the OS
    pub autostart: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "Ctrl+Shift+V".to_string(),
            delay_ms: 30,
            autostart: false,
        }
    }
}

impl Config {
    /// Returns the platform-specific path to config.json.
    /// macOS uses ~/Library/Application Support/macro_paste/ (standard convention).
    /// Windows stores the file next to the executable for portability.
    fn config_path() -> PathBuf {
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            let dir = PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("macro_paste");
            let _ = fs::create_dir_all(&dir);
            dir.join("config.json")
        }
        #[cfg(not(target_os = "macos"))]
        {
            let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
            path.set_file_name("config.json");
            path
        }
    }

    /// Load config from disk, or return defaults if the file doesn't exist / is invalid
    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            let content = fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            let config = Self::default();
            // Create default config file on first run
            let _ = config.save();
            config
        }
    }

    /// Persist current config to disk
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        fs::write(&path, json).map_err(|e| e.to_string())
    }
}
