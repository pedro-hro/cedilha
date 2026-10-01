//! Rust backend for Cedilha — file I/O, dialogs, and window helpers.

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use tauri::window::Window;

/// Shared application state threaded through the command handlers.
pub struct AppState {
    /// The file currently open in the editor, if any.
    pub current_path: Mutex<Option<PathBuf>>,
    /// Persisted user preferences (theme, font size, ...).
    pub prefs: Mutex<crate::config::Prefs>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoadResult {
    pub path: Option<String>,
    pub content: String,
    pub prefs: crate::config::Prefs,
}

/// Extension filters offered by the open / save dialogs.
const MD_FILTERS: &[(&str, &[&str])] = &[
    ("Markdown", &["md", "mdown", "markdown", "mdx"]),
    ("Text", &["txt", "text"]),
    ("All Files", &["*"]),
];

/// Run an asynchronous file-open dialog.
async fn pick_async(title: &'static str) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title(title);
    for &(name, exts) in MD_FILTERS {
        dialog = dialog.add_filter(name, exts);
    }
    dialog.pick_file().await.map(|f| f.path().to_path_buf())
}

/// Run an asynchronous save-as dialog.
async fn save_async(title: &'static str) -> Option<PathBuf> {
    let mut dialog = rfd::AsyncFileDialog::new().set_title(title);
    for &(name, exts) in MD_FILTERS {
        dialog = dialog.add_filter(name, exts);
    }
    dialog.save_file().await.map(|f| f.path().to_path_buf())
}

/// Open a file through the native OS dialog.
#[tauri::command]
pub async fn open_file(state: tauri::State<'_, AppState>) -> Result<LoadResult, String> {
    let path = pick_async("Open Document").await;
    let Some(path) = path else {
        return Ok(LoadResult {
            path: None,
            content: String::new(),
            prefs: state.prefs.lock().unwrap().clone(),
        });
    };

    let content = fs::read_to_string(&path).map_err(|e| format!("read failed: {e}"))?;
    *state.current_path.lock().unwrap() = Some(path.clone());

    Ok(LoadResult {
        path: Some(path.to_string_lossy().into_owned()),
        content,
        prefs: state.prefs.lock().unwrap().clone(),
    })
}

/// Read a document from an explicit path (no dialog).
#[tauri::command]
pub async fn read_document(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("read failed for {path}: {e}"))
}

/// Persist document content to disk.
#[tauri::command]
pub async fn save_document(
    path: String,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    {
        let mut guard = state.current_path.lock().unwrap();
        *guard = Some(PathBuf::from(&path));
    }

    fs::File::create(&path)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| format!("write failed for {path}: {e}"))
}

/// Save under a new location using the native save dialog.
#[tauri::command]
pub async fn save_as_file(content: String, state: tauri::State<'_, AppState>) -> Result<Option<String>, String> {
    let path = save_async("Save Document").await;
    let Some(path) = path else {
        return Ok(None);
    };

    fs::File::create(&path)
        .and_then(|mut f| f.write_all(content.as_bytes()))
        .map_err(|e| format!("write failed for {}: {e}", path.display()))?;

    *state.current_path.lock().unwrap() = Some(path.clone());
    Ok(Some(path.to_string_lossy().into_owned()))
}

/// Create a new empty document.
#[tauri::command]
pub fn new_document(state: tauri::State<AppState>) -> String {
    *state.current_path.lock().unwrap() = None;
    String::new()
}

/// Toggle the window between minimized and restored/visible. Handy for
/// keyboard-first workflows and quick window switching.
#[tauri::command(window = "window")]
pub async fn toggle_minimize(window: Window) {
    if window.is_minimized().unwrap_or(false) {
        window.unminimize().ok();
    } else {
        window.minimize().ok();
    }
    window.set_focus().ok();
}

/// Fetch the currently persisted preferences.
#[tauri::command]
pub fn get_prefs(state: tauri::State<AppState>) -> crate::config::Prefs {
    state.prefs.lock().unwrap().clone()
}

/// Persist a new set of preferences.
#[tauri::command]
pub fn save_prefs(
    new_prefs: crate::config::Prefs,
    state: tauri::State<AppState>,
) -> Result<(), String> {
    crate::config::save(&new_prefs).map_err(|e| e.to_string())?;
    *state.prefs.lock().unwrap() = new_prefs;
    Ok(())
}

/// Fetch the active Omarchy Linux theme, if present on the system.
#[tauri::command]
pub fn get_omarchy_theme() -> Option<crate::omarchy::OmarchyTheme> {
    crate::omarchy::read_current_omarchy_theme()
}

/// High-performance asynchronous markdown renderer using native pulldown-cmark in Rust.
/// Capable of parsing megabyte-scale documents in milliseconds off the main UI thread.
#[tauri::command]
pub async fn render_markdown(content: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut options = pulldown_cmark::Options::empty();
        options.insert(pulldown_cmark::Options::ENABLE_TABLES);
        options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);
        options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
        options.insert(pulldown_cmark::Options::ENABLE_TASKLISTS);
        options.insert(pulldown_cmark::Options::ENABLE_HEADING_ATTRIBUTES);
        options.insert(pulldown_cmark::Options::ENABLE_MATH);

        let parser = pulldown_cmark::Parser::new_ext(&content, options);
        let mut html = String::with_capacity(content.len() * 3 / 2);
        pulldown_cmark::html::push_html(&mut html, parser);
        html
    })
    .await
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_markdown_math() {
        tauri::async_runtime::block_on(async {
            let input = "Math with symbols: $x < y$ and $a > b$ and $a & b$\n";
            let res = render_markdown(input.to_string()).await.unwrap();
            println!("RENDERED HTML:\n{}", res);
        });
    }

    #[test]
    fn test_render_huge_galaxy_book() {
        let path = "/tmp/sample.md";
        if let Ok(content) = std::fs::read_to_string(path) {
            let start = std::time::Instant::now();
            let html = tauri::async_runtime::block_on(async {
                render_markdown(content).await.unwrap()
            });
            let elapsed = start.elapsed();
            println!("Huge file parsed in {:?}! HTML len: {}", elapsed, html.len());
        }
    }
}

