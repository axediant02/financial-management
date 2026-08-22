mod auth;
mod backup;
mod commands;
mod db;
mod error;
mod export;
mod models;
mod paths;
mod state;

use state::{AppState, SessionStore};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db_path = paths::db_path(app.handle())?;
            let backups_dir = paths::backups_dir(app.handle())?;

            let conn = db::open_db(&db_path)?;
            db::migrate(&conn)?;
            backup::create_daily_backup_if_needed(&conn, &db_path, &backups_dir)?;

            app.manage(AppState {
                db_path,
                sessions: std::sync::Arc::new(SessionStore::new()),
            });
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::app_status,
            commands::bootstrap_admin,
            commands::request_admin_password_replace,
            commands::complete_admin_password_replace,
            commands::login,
            commands::logout,
            commands::audit_trail_list,
            commands::donors_list,
            commands::donors_create,
            commands::donors_delete,
            commands::categories_list,
            commands::categories_create,
            commands::projects_list,
            commands::projects_create,
            commands::projects_update,
            commands::projects_delete,
            commands::documentations_list,
            commands::documentation_detail,
            commands::documentations_create,
            commands::documentation_expenses_create,
            commands::documentation_expenses_delete,
            commands::documentations_delete,
            commands::donations_list,
            commands::donations_create,
            commands::donations_update,
            commands::donations_delete,
            commands::expenses_list,
            commands::expenses_create,
            commands::expenses_update,
            commands::expenses_delete,
            commands::ledger_summary,
            commands::project_balances,
            commands::project_report,
            commands::export_csv_command,
            commands::export_pdf_command,
            commands::backup_list,
            commands::backup_create,
            commands::backup_restore,
            commands::database_health,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
