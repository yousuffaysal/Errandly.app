mod agents;
mod ai;
mod commands;
mod error;
mod keychain;
mod security;
mod storage;
mod tools;

use tauri::Manager;

use commands::AppState;
use storage::sqlite::Db;

/// On-disk folder name under ~/Library/Application Support. Kept separate from
/// the displayed product name so the app can be rebranded without moving data.
const DATA_DIR_NAME: &str = "Errandly";
const DEFAULT_ZOOM: f64 = 1.2;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let db_path = app
                .path()
                .data_dir()?
                .join(DATA_DIR_NAME)
                .join("database")
                .join("actiondesk.sqlite");
            let db = Db::open(&db_path)?;
            let settled = agents::executor::reconcile_interrupted(&db)?;
            if settled > 0 {
                eprintln!("settled {settled} task(s) interrupted by the last shutdown");
            }
            app.manage(AppState::new(db));
            // The design is drawn at a compact web scale; show it a little larger
            // on the desktop. Cmd +/- still adjusts it.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_zoom(DEFAULT_ZOOM);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ai_status,
            commands::install_models,
            commands::warm_up,
            commands::list_projects,
            commands::create_project,
            commands::update_project,
            commands::delete_project,
            commands::list_conversations,
            commands::create_conversation,
            commands::get_conversation,
            commands::rename_conversation,
            commands::delete_conversation,
            commands::set_persona,
            commands::set_instructions,
            commands::attach_folder,
            commands::detach_folder,
            commands::send_message,
            commands::stop_conversation,
            commands::export_conversation,
            commands::approve_task,
            commands::cancel_task,
            commands::undo_task,
            commands::get_task,
            keychain::secure_get,
            keychain::secure_set,
            keychain::secure_remove,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
