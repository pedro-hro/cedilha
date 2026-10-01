//! Preference persistence for Cedilha.
//!
//! Preferences are stored as JSON in the platform config directory
//! (`$XDG_CONFIG_HOME/cedilha/prefs.json`, or a sensible fallback).

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User-configurable application preferences.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prefs {
    /// Active color theme. One of `catppuccin-mocha`, `catppuccin-latte`,
    /// `tokyo-night`, `gruvbox-dark`.
    pub theme: String,
    /// Base editor font size in pixels.
    pub font_size: u32,
    /// Tab width in columns.
    pub tab_size: u32,
    /// Typewriter (centered active line) scrolling.
    pub typewriter: bool,
    /// Line numbers in the editor gutter.
    #[serde(default)]
    pub line_numbers: bool,
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            theme: "catppuccin-mocha".to_string(),
            font_size: 15,
            tab_size: 2,
            typewriter: true,
            line_numbers: false,
        }
    }
}

/// Resolve the directory used to store preferences.
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("cedilha");
        }
    }
    // Fallback to $HOME/.config/cedilha
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".config").join("cedilha"))
        .unwrap_or_else(|_| PathBuf::from("cedilha-config"))
}

/// Load preferences from disk, falling back to defaults on any error.
pub fn load_once() -> Result<Prefs, std::io::Error> {
    let path = config_dir().join("prefs.json");
    let bytes = std::fs::read(&path)?;
    let prefs: Prefs = serde_json::from_slice(&bytes)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string()))?;
    Ok(prefs)
}

/// Persist preferences to disk, creating the config directory if needed.
pub fn save(prefs: &Prefs) -> Result<(), std::io::Error> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("prefs.json");
    let json = serde_json::to_string_pretty(prefs).unwrap_or_default();
    std::fs::write(path, json)
}
