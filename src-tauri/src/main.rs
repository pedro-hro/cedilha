//! Application entry point. Wires up Tauri and registers IPC commands.

#![cfg_attr(windows, windows_subsystem = "windows")]

mod commands;
mod config;
mod omarchy;

use tauri::{Emitter, Manager};

fn main() {
    #[cfg(target_os = "linux")]
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");

    let initial_prefs = config::load_once().unwrap_or_default();

    tauri::Builder::default()
        .setup(|app| {
            crate::omarchy::start_theme_watcher(app.handle().clone());
            crate::config::start_config_watcher(app.handle().clone());
            Ok(())
        })
        .manage({
            let state = crate::commands::AppState::new(initial_prefs);
            // `cedilha notes.md` opens that file (anything starting with '-' is a flag).
            *state.initial_file.lock().unwrap() = std::env::args()
                .nth(1)
                .filter(|a| !a.starts_with('-'))
                .map(|a| crate::commands::path_from_arg(&a));
            state
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Let the frontend check for unsaved changes first. If it never answers and
                // the user asks to close again, the second request goes through.
                let state = window.state::<crate::commands::AppState>();
                if !state.close_requested.swap(true, std::sync::atomic::Ordering::SeqCst) {
                    api.prevent_close();
                    let _ = window.emit("close-requested", ());
                }
            }
        })
        .invoke_handler(tauri::generate_handler!(
            crate::commands::open_file,
            crate::commands::initial_file,
            crate::commands::read_document,
            crate::commands::save_document,
            crate::commands::save_as_file,
            crate::commands::new_document,
            crate::commands::minimize_window,
            crate::commands::toggle_maximize,
            crate::commands::open_url,
            crate::commands::is_tiling_wm,
            crate::commands::get_prefs,
            crate::commands::save_prefs,
            crate::commands::get_omarchy_theme,
            crate::commands::render_markdown,
            crate::commands::exit_app,
            crate::commands::cancel_close
        ))
        .run(tauri::generate_context!())
        .expect("error while running Cedilha application");
}
