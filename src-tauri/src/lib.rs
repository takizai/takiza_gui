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
        .plugin(tauri_plugin_dialog::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            start_agent_turn,
            regenerate_agent_turn,
            list_checkpoints,
            preview_restore,
            restore_checkpoint,
            import_image,
            import_text_file,
            import_paths,
            paste_attachments,
            cancel_agent,
            respond_permission,
            respond_question,
            get_available_skills,
            read_skill_content,
            get_current_session,
            get_sessions,
            load_session,
            new_session,
            delete_session,
            get_config,
            save_preferences,
            list_provider_models,
            get_git_info,
            list_files,
            get_file_content,
            set_workspace_dir,
            workspace_has_takiza,
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
            set_language,
            get_usage,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
