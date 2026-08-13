use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

pub fn app_data_dir(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    let path = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::InvalidInput(format!("app_data_dir unavailable: {e}")))?;
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

pub fn db_path(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    let dir = app_data_dir(app)?;
    Ok(dir.join("project-funds-tracker.sqlite3"))
}

pub fn backups_dir(app: &AppHandle) -> AppResult<std::path::PathBuf> {
    let dir = app_data_dir(app)?.join("backups");
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}
