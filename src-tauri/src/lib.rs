mod agents;
mod ai;
mod commands;
mod crash;
mod error;
mod oauth;
mod security;
mod session;
mod storage;
mod tools;

use tauri::Manager;

use commands::AppState;
use storage::sqlite::Db;

/// On-disk folder name under ~/Library/Application Support. Kept separate from
/// the displayed product name so the app can be rebranded without moving data.
const DATA_DIR_NAME: &str = "Errandly";
const DEFAULT_ZOOM: f64 = 1.2;

/// Early builds named the database `actiondesk.sqlite`; move it (and its WAL
/// files) to the new name once, so existing conversations carry over.
fn adopt_legacy_database(dir: &std::path::Path, db_path: &std::path::Path) {
    let legacy = dir.join("actiondesk.sqlite");
    if db_path.exists() || !legacy.exists() {
        return;
    }
    for suffix in ["", "-wal", "-shm"] {
        let from = dir.join(format!("actiondesk.sqlite{suffix}"));
        if from.exists() {
            let _ = std::fs::rename(&from, dir.join(format!("errandly.sqlite{suffix}")));
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    crash::install_panic_hook();
    ai::ollama::ensure_tls_provider();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .setup(|app| {
            let db_dir = app.path().data_dir()?.join(DATA_DIR_NAME).join("database");
            let db_path = db_dir.join("errandly.sqlite");
            adopt_legacy_database(&db_dir, &db_path);
            let db = Db::open(&db_path)?;
            let zoom = storage::settings::preferences(&db).map(|p| p.zoom()).unwrap_or(DEFAULT_ZOOM);
            let settled = agents::executor::reconcile_interrupted(&db)?;
            if settled > 0 {
                eprintln!("settled {settled} task(s) interrupted by the last shutdown");
            }
            let _ = storage::usage::bump(&db, storage::usage::Counter::Open);
            app.manage(AppState::new(db, db_path));
            ai::runtime::start(app.handle());
            // The design is drawn at a compact web scale; show it a little larger
            // on the desktop. Cmd +/- still adjusts it.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_zoom(zoom);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::ai_status,
            commands::install_models,
            commands::get_settings,
            commands::usage_pending,
            commands::usage_sent,
            commands::save_profile,
            commands::set_active_account,
            commands::save_preferences,
            commands::storage_report,
            commands::clear_cache,
            commands::list_grants,
            commands::revoke_grant,
            commands::export_all_data,
            commands::delete_all_conversations,
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
            commands::export_card,
            commands::approve_task,
            commands::cancel_task,
            commands::undo_task,
            commands::get_task,
            session::session_get,
            session::session_set,
            session::session_remove,
            oauth::start_oauth_listener,
            oauth::open_auth_url,
            crash::log_crash,
            crash::unsent_crashes,
            crash::mark_crash_sent,
            crash::open_crash_folder,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_, event| {
            if let tauri::RunEvent::Exit = event {
                ai::runtime::stop();
            }
        });
}
