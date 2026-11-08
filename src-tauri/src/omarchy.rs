//! Omarchy Linux theme detection and synchronization.
//!
//! Omarchy stores the active theme in:
//! - `$HOME/.local/state/omarchy/current/theme.name`
//! - `$HOME/.local/state/omarchy/current/theme/colors.toml`
//!
//! This module reads the active theme palette and provides helpers
//! to format CSS tokens and watch for desktop theme transitions in real time.

use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OmarchyTheme {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub is_light: bool,
    pub bg: String,
    pub bg_elev: String,
    pub bg_surface: String,
    pub fg: String,
    pub fg_dim: String,
    pub accent: String,
    pub accent2: String,
    pub danger: String,
    pub success: String,
    pub warning: String,
    pub border: String,
    pub selection: String,
    pub marker: String,
    pub active_line: String,
}

fn omarchy_dir() -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(home).join(".local/state/omarchy/current");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn get_theme_name() -> Option<String> {
    let dir = omarchy_dir()?;
    let name_file = dir.join("theme.name");
    if let Ok(name) = fs::read_to_string(name_file) {
        let trimmed = name.trim().to_string();
        if !trimmed.is_empty() {
            return Some(trimmed);
        }
    }
    None
}

/// Convert a kebab-case theme name to Title Case (e.g. "greek-noir" -> "Greek Noir").
fn format_display_name(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn parse_toml_key_values(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            let key = k.trim().to_string();
            let val = v.trim().trim_matches('"').trim_matches('\'').trim().to_string();
            map.insert(key, val);
        }
    }
    map
}

fn hex_to_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim().trim_start_matches('#');
    // Non-ASCII input would make the byte slices below panic on a char boundary.
    if !hex.is_ascii() {
        return None;
    }
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
        Some((r, g, b))
    } else if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
        Some((r, g, b))
    } else {
        None
    }
}

fn hex_to_rgba(hex: &str, alpha: f32) -> String {
    if let Some((r, g, b)) = hex_to_rgb(hex) {
        format!("rgba({}, {}, {}, {:.3})", r, g, b, alpha)
    } else {
        hex.to_string()
    }
}

fn blend_colors(fg_hex: &str, bg_hex: &str, fg_ratio: f32) -> String {
    if let (Some((r1, g1, b1)), Some((r2, g2, b2))) = (hex_to_rgb(fg_hex), hex_to_rgb(bg_hex)) {
        let r = ((r1 as f32 * fg_ratio) + (r2 as f32 * (1.0 - fg_ratio))).round() as u8;
        let g = ((g1 as f32 * fg_ratio) + (g2 as f32 * (1.0 - fg_ratio))).round() as u8;
        let b = ((b1 as f32 * fg_ratio) + (b2 as f32 * (1.0 - fg_ratio))).round() as u8;
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    } else {
        bg_hex.to_string()
    }
}

fn is_light_color(hex: &str) -> bool {
    if let Some((r, g, b)) = hex_to_rgb(hex) {
        let lum = 0.2126 * (r as f32) + 0.7152 * (g as f32) + 0.0722 * (b as f32);
        lum > 135.0
    } else {
        false
    }
}

pub fn read_current_omarchy_theme() -> Option<OmarchyTheme> {
    let dir = omarchy_dir()?;
    let colors_file = dir.join("theme/colors.toml");
    let content = fs::read_to_string(&colors_file).ok()?;
    let colors = parse_toml_key_values(&content);

    let raw_name = get_theme_name().unwrap_or_else(|| "omarchy".to_string());
    let display_name = format_display_name(&raw_name);

    let bg = colors
        .get("background")
        .cloned()
        .unwrap_or_else(|| "#171717".to_string());
    let fg = colors
        .get("foreground")
        .cloned()
        .unwrap_or_else(|| "#CCD0CF".to_string());
    let accent = colors
        .get("accent")
        .cloned()
        .unwrap_or_else(|| "#F25623".to_string());

    let is_light = colors
        .get("mode")
        .map(|m| m.eq_ignore_ascii_case("light"))
        .unwrap_or_else(|| is_light_color(&bg));

    let bg_elev = if is_light {
        colors
            .get("dark_background")
            .cloned()
            .unwrap_or_else(|| blend_colors(&fg, &bg, 0.05))
    } else {
        colors
            .get("lighter_background")
            .cloned()
            .unwrap_or_else(|| blend_colors(&fg, &bg, 0.05))
    };

    let bg_surface = if is_light {
        colors
            .get("darker_background")
            .cloned()
            .unwrap_or_else(|| blend_colors(&fg, &bg, 0.09))
    } else {
        blend_colors(&fg, &bg, 0.09)
    };

    let fg_dim = colors
        .get("muted")
        .or_else(|| colors.get("dark_foreground"))
        .cloned()
        .unwrap_or_else(|| blend_colors(&fg, &bg, 0.65));

    let accent2 = colors
        .get("active_border_color")
        .or_else(|| colors.get("color4"))
        .or_else(|| colors.get("color5"))
        .cloned()
        .unwrap_or_else(|| accent.clone());

    let danger = colors
        .get("color1")
        .or_else(|| colors.get("color9"))
        .cloned()
        .unwrap_or_else(|| if is_light { "#d20df3".to_string() } else { "#f38ba8".to_string() });

    let success = colors
        .get("color2")
        .or_else(|| colors.get("color4"))
        .or_else(|| colors.get("color10"))
        .cloned()
        .unwrap_or_else(|| if is_light { "#40a02b".to_string() } else { "#a6e3a1".to_string() });

    let warning = colors
        .get("color3")
        .or_else(|| colors.get("color11"))
        .cloned()
        .unwrap_or_else(|| if is_light { "#fe640b".to_string() } else { "#fab387".to_string() });

    let border = if let Some(b) = colors.get("active_border_color") {
        hex_to_rgba(b, 0.35)
    } else if let Some(c8) = colors.get("color8") {
        c8.clone()
    } else {
        hex_to_rgba(&fg, 0.15)
    };

    let selection = if let Some(sel) = colors.get("selection_background").or_else(|| colors.get("selection")) {
        hex_to_rgba(sel, 0.35)
    } else {
        hex_to_rgba(&accent, 0.3)
    };

    let marker = accent.clone();

    let active_line = if is_light {
        "rgba(0, 0, 0, 0.04)".to_string()
    } else {
        "rgba(255, 255, 255, 0.035)".to_string()
    };

    Some(OmarchyTheme {
        id: "omarchy".to_string(),
        name: raw_name,
        display_name,
        is_light,
        bg,
        bg_elev,
        bg_surface,
        fg,
        fg_dim,
        accent,
        accent2,
        danger,
        success,
        warning,
        border,
        selection,
        marker,
        active_line,
    })
}

pub fn start_theme_watcher(app_handle: AppHandle) {
    std::thread::spawn(move || {
        let mut last_theme = read_current_omarchy_theme();
        loop {
            std::thread::sleep(std::time::Duration::from_millis(1200));
            if let Some(new_theme) = read_current_omarchy_theme() {
                if last_theme.as_ref() != Some(&new_theme) {
                    let _ = app_handle.emit("omarchy-theme-changed", &new_theme);
                    last_theme = Some(new_theme);
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_parsing_never_panics_on_odd_input() {
        assert_eq!(hex_to_rgb("#fff"), Some((255, 255, 255)));
        assert_eq!(hex_to_rgb("#171717"), Some((23, 23, 23)));
        assert_eq!(hex_to_rgb("abçç"), None);
        assert_eq!(hex_to_rgb("ççç"), None);
        assert_eq!(hex_to_rgb(""), None);
    }

    #[test]
    fn display_name_is_title_case() {
        assert_eq!(format_display_name("greek-noir"), "Greek Noir");
    }
}
