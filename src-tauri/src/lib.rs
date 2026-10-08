mod agents;
mod ai;
mod commands;
mod error;
mod security;
mod storage;
mod tools;

use tauri::Manager;

use commands::AppState;
use storage::sqlite::Db;

/// On-disk folder name under ~/Library/Application Support. Kept separate from
/// the displayed product name so the app can be rebranded without moving data.
const DATA_DIR_NAME: &str = "Errandly";

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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ai_status,
            commands::list_conversations,
            commands::create_conversation,
            commands::get_conversation,
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
