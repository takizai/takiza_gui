pub mod commands;
pub mod core;
pub mod logger;
pub mod state;

use commands::*;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new(None);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_agent_turn,
            cancel_agent,
            respond_permission,
            get_current_session,
            get_sessions,
            load_session,
            new_session,
            delete_session,
            get_config,
            save_preferences,
            get_git_info,
            list_files,
            get_file_content,
            set_workspace_dir,
            reset_to_harness_defaults,
            minimize_window,
            toggle_maximize_window,
            close_window,
            is_window_maximized,
            start_dragging_window,
            set_webview_zoom,
            set_effort,
            set_mode,
            set_theme,
            get_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
