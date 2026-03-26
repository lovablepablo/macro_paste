//! Configuration module – loads/saves settings from a JSON file next to the executable.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Application configuration stored as JSON beside the .exe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Hotkey identifier string, e.g. "Ctrl+Shift+V"
    pub hotkey: String,
    /// Delay in milliseconds between each simulated keystroke
    pub delay_ms: u64,
    /// Whether the app should start with Windows
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
    /// Returns the path to config.json next to the running executable
    fn config_path() -> PathBuf {
        let mut path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("."));
        path.set_file_name("config.json");
        path
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
