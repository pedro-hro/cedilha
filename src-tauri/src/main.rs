//! Application entry point. Wires up Tauri and registers IPC commands.

#![cfg_attr(windows, windows_subsystem = "windows")]

mod commands;
mod config;
mod omarchy;

fn main() {
    let initial_prefs = config::load_once().unwrap_or_default();

    tauri::Builder::default()
        .setup(|app| {
            crate::omarchy::start_theme_watcher(app.handle().clone());
            crate::config::start_config_watcher(app.handle().clone());
            Ok(())
        })
        .manage(crate::commands::AppState {
            current_path: std::sync::Mutex::new(None),
            prefs: std::sync::Mutex::new(initial_prefs),
        })
        .invoke_handler(tauri::generate_handler!(
            crate::commands::open_file,
            crate::commands::read_document,
            crate::commands::save_document,
            crate::commands::save_as_file,
            crate::commands::new_document,
            crate::commands::toggle_minimize,
            crate::commands::get_prefs,
            crate::commands::save_prefs,
            crate::commands::get_omarchy_theme
        ))
        .run(tauri::generate_context!())
        .expect("error while running Cedilha application");
}
