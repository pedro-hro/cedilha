//! Configuration persistence and live hot-reloading for Cedilha.
//!
//! Preferences are stored in `$XDG_CONFIG_HOME/cedilha/config.toml`
//! (or `~/.config/cedilha/config.toml`).
//!
//! Cedilha watches this file in the background so edits made via external editors
//! (Neovim, Helix, etc.) hot-reload into the running application immediately.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prefs {
    /// Active color theme.
    #[serde(default = "default_theme")]
    pub theme: String,
    /// Base editor font size in pixels.
    #[serde(default = "default_font_size")]
    pub font_size: u32,
    /// Tab width in columns.
    #[serde(default = "default_tab_size")]
    pub tab_size: u32,
    /// Typewriter (centered active line) scrolling.
    #[serde(default = "default_true")]
    pub typewriter: bool,
    /// Line numbers in the editor gutter.
    #[serde(default = "default_true")]
    pub line_numbers: bool,
    /// Custom font family. Defaults to JetBrains Mono.
    #[serde(default = "default_font_family")]
    pub font_family: String,
    /// Max content column width in pixels (default: 1040).
    #[serde(default = "default_editor_max_width")]
    pub editor_max_width: u32,
    /// View shown on launch: "edit", "split" or "preview".
    #[serde(default = "default_view")]
    pub default_view: String,
    /// Keybinding overrides: action id -> key combos (e.g. `save = ["Ctrl+S"]`). Actions
    /// not listed keep the defaults that live in the frontend.
    #[serde(default)]
    pub keybindings: BTreeMap<String, Vec<String>>,
}

fn default_theme() -> String {
    if crate::omarchy::omarchy_dir().is_some() {
        "omarchy".to_string()
    } else {
        "catppuccin-mocha".to_string()
    }
}

fn default_font_size() -> u32 {
    15
}

fn default_tab_size() -> u32 {
    2
}

fn default_true() -> bool {
    true
}

fn default_view() -> String {
    "edit".to_string()
}

fn default_font_family() -> String {
    "JetBrains Mono".to_string()
}

fn default_editor_max_width() -> u32 {
    1040
}

impl Default for Prefs {
    fn default() -> Self {
        Prefs {
            theme: default_theme(),
            font_size: default_font_size(),
            tab_size: default_tab_size(),
            typewriter: true,
            line_numbers: true,
            font_family: default_font_family(),
            editor_max_width: default_editor_max_width(),
            default_view: default_view(),
            keybindings: BTreeMap::new(),
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
    // Properly quoted/escaped TOML string literals (including the surrounding quotes).
    let quote = |s: &str| toml::Value::String(s.to_string()).to_string();

    // Only overrides are written. Defaults live in the app, so a changed default reaches
    // everyone who hasn't customized that action. Must come last: it is a TOML table.
    let mut keybindings = String::new();
    if !prefs.keybindings.is_empty() {
        keybindings.push_str(
            "\n# Keybinding overrides. Only actions you change are listed; remove a line to\n\
             # restore its default. Format: Ctrl+Shift+S, Alt+Enter, F3, ...\n\
             # Actions: cycle_view, new, open, save, save_as, settings, find, find_next, find_prev\n\
             [keybindings]\n",
        );
        for (action, combos) in &prefs.keybindings {
            let key = if action.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                action.clone()
            } else {
                quote(action)
            };
            let list: Vec<String> = combos.iter().map(|c| quote(c)).collect();
            keybindings.push_str(&format!("{key} = [{}]\n", list.join(", ")));
        }
    }
    format!(
r#"# Cedilha Configuration (~/.config/cedilha/config.toml)
# Opinionated, distraction-free Markdown & LaTeX editor

# Active color theme.
# Options: "omarchy" (follows Omarchy Linux system theme),
#          "catppuccin-mocha", "catppuccin-latte", "tokyo-night", "gruvbox-dark"
theme = {}

# Typography
# font_family: e.g. "JetBrains Mono", "Iosevka", "Fira Code", "monospace"
font_family = {}
font_size = {}
tab_size = {}

# Layout
# Max content column width in pixels (default: 1040)
editor_max_width = {}

# Behavior
# View shown on launch: "edit", "split" or "preview"
default_view = {}

# Keep the active line vertically centered (typewriter scrolling)
typewriter = {}

# Display line numbers in the gutter
line_numbers = {}
{}"#,
        quote(&prefs.theme),
        quote(&prefs.font_family),
        prefs.font_size,
        prefs.tab_size,
        prefs.editor_max_width,
        quote(&prefs.default_view),
        prefs.typewriter,
        prefs.line_numbers,
        keybindings
    )
}

/// Load configuration from disk, with fallback and migration from prefs.json.
pub fn load_once() -> Result<Prefs, std::io::Error> {
    let toml_file = config_path();
    if toml_file.exists() {
        match std::fs::read_to_string(&toml_file)
            .map_err(|e| e.to_string())
            .and_then(|c| toml::from_str::<Prefs>(&c).map_err(|e| e.to_string()))
        {
            Ok(prefs) => return Ok(prefs),
            Err(e) => {
                // Leave the user's file alone; run with defaults until it is fixed.
                eprintln!(
                    "cedilha: could not read {}: {e}. Using defaults; the file was not modified.",
                    toml_file.display()
                );
                return Ok(Prefs::default());
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
    let path = config_path();
    if let Ok(existing) = std::fs::read_to_string(&path) {
        if toml::from_str::<Prefs>(&existing).is_err() {
            let _ = std::fs::write(path.with_extension("toml.bak"), existing);
        }
    }
    crate::commands::write_atomic(&path, toml_content.as_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_toml_round_trips_awkward_strings() {
        let prefs = Prefs {
            font_family: "Evil\" \n[injected]\nfont_size = 99 \\ \"".to_string(),
            ..Prefs::default()
        };
        let parsed: Prefs = toml::from_str(&generate_toml(&prefs)).expect("must stay valid TOML");
        assert_eq!(parsed, prefs);
    }

    #[test]
    fn keybinding_overrides_round_trip() {
        let mut prefs = Prefs::default();
        prefs.keybindings.insert("save".into(), vec!["Ctrl+Alt+S".into()]);
        prefs.keybindings.insert("find_next".into(), vec!["F3".into(), "Ctrl+\"".into()]);
        let toml = generate_toml(&prefs);
        assert!(toml.contains("[keybindings]"));
        let parsed: Prefs = toml::from_str(&toml).unwrap();
        assert_eq!(parsed, prefs);
        // no overrides -> no table at all
        assert!(!generate_toml(&Prefs::default()).contains("[keybindings]"));
    }

    #[test]
    fn line_numbers_default_on_when_missing() {
        let parsed: Prefs = toml::from_str("font_size = 20\n").unwrap();
        assert!(parsed.line_numbers);
    }

    #[test]
    fn partial_config_loads_with_defaults() {
        let parsed: Prefs = toml::from_str("font_size = 20\n").unwrap();
        assert_eq!(parsed.font_size, 20);
        assert_eq!(parsed.tab_size, 2);
        assert!(parsed.typewriter);
    }
}
