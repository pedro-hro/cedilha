//! Configuration persistence and live hot-reloading for Cedilha.
//!
//! Preferences are stored in `$XDG_CONFIG_HOME/cedilha/config.toml`
//! (or `~/.config/cedilha/config.toml`).
//!
//! Cedilha watches this file in the background so edits made via external editors
//! (Neovim, Helix, etc.) hot-reload into the running application immediately.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prefs {
    /// Active color theme.
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
    /// Custom font family. Defaults to JetBrains Mono.
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// Max content column width in pixels (default: 1040).
    #[serde(default = "default_editor_max_width")]
    pub editor_max_width: u32,
}

fn default_font_family() -> String {
    "JetBrains Mono".to_string()
}

fn default_editor_max_width() -> u32 {
    1040
}

impl Default for Prefs {
    fn default() -> Self {
        let theme = if crate::omarchy::omarchy_dir().is_some() {
            "omarchy".to_string()
        } else {
            "catppuccin-mocha".to_string()
        };
        Prefs {
            theme,
            font_size: 15,
            tab_size: 2,
            typewriter: true,
            line_numbers: true,
            font_family: default_font_family(),
            editor_max_width: default_editor_max_width(),
        }
    }
}

/// Resolve the directory used to store configuration.
pub fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("XDG_CONFIG_HOME") {
        if !dir.is_empty() {
            return PathBuf::from(dir).join("cedilha");
        }
    }
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".config").join("cedilha"))
        .unwrap_or_else(|_| PathBuf::from("cedilha-config"))
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn legacy_json_path() -> PathBuf {
    config_dir().join("prefs.json")
}

/// Generate a well-commented TOML file string.
pub fn generate_toml(prefs: &Prefs) -> String {
    format!(
r#"# Cedilha Configuration (~/.config/cedilha/config.toml)
# Opinionated, distraction-free Markdown & LaTeX editor

# Active color theme.
# Options: "omarchy" (follows Omarchy Linux system theme),
#          "catppuccin-mocha", "catppuccin-latte", "tokyo-night", "gruvbox-dark"
theme = "{}"

# Typography
# font_family: e.g. "JetBrains Mono", "Iosevka", "Fira Code", "monospace"
font_family = "{}"
font_size = {}
tab_size = {}

# Layout
# Max content column width in pixels (default: 1040)
editor_max_width = {}

# Behavior
# Keep the active line vertically centered (typewriter scrolling)
typewriter = {}

# Display line numbers in the gutter
line_numbers = {}
"#,
        prefs.theme,
        prefs.font_family,
        prefs.font_size,
        prefs.tab_size,
        prefs.editor_max_width,
        prefs.typewriter,
        prefs.line_numbers
    )
}

/// Load configuration from disk, with fallback and migration from prefs.json.
pub fn load_once() -> Result<Prefs, std::io::Error> {
    let toml_file = config_path();
    if toml_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&toml_file) {
            if let Ok(prefs) = toml::from_str::<Prefs>(&content) {
                return Ok(prefs);
            }
        }
    }

    // Migrate from legacy prefs.json if present
    let json_file = legacy_json_path();
    if json_file.exists() {
        if let Ok(bytes) = std::fs::read(&json_file) {
            if let Ok(mut legacy) = serde_json::from_slice::<Prefs>(&bytes) {
                if legacy.font_family.is_empty() {
                    legacy.font_family = default_font_family();
                }
                if legacy.editor_max_width == 0 {
                    legacy.editor_max_width = default_editor_max_width();
                }
                let _ = save(&legacy);
                return Ok(legacy);
            }
        }
    }

    let default_prefs = Prefs::default();
    let _ = save(&default_prefs);
    Ok(default_prefs)
}

/// Persist configuration to config.toml with comments.
pub fn save(prefs: &Prefs) -> Result<(), std::io::Error> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)?;
    let toml_content = generate_toml(prefs);
    std::fs::write(config_path(), toml_content)
}

/// Start a lightweight watcher that monitors config.toml and emits "config-changed".
pub fn start_config_watcher(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let path = config_path();
        let mut last_content = std::fs::read_to_string(&path).ok();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(1000));
            if let Ok(content) = std::fs::read_to_string(&path) {
                if last_content.as_deref() != Some(&content) {
                    if let Ok(new_prefs) = toml::from_str::<Prefs>(&content) {
                        let _ = app_handle.emit("config-changed", &new_prefs);
                    }
                    last_content = Some(content);
                }
            }
        }
    });
}
