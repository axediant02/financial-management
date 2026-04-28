use std::fs;
use std::path::{Path, PathBuf};

use chrono::{Datelike, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, AppResult};
use crate::models::BackupInfo;

const MAX_BACKUPS: usize = 7;

fn today_key() -> String {
    let now = Utc::now().date_naive();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}

pub fn create_daily_backup_if_needed(
    conn: &Connection,
    db_path: &Path,
    backups_dir: &Path,
) -> AppResult<()> {
    let last_backup: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key='last_backup_date'",
            [],
            |row| row.get(0),
        )
        .optional()?;

    let today = today_key();
    if last_backup.as_deref() == Some(today.as_str()) {
        return Ok(());
    }

    create_backup_file(db_path, backups_dir)?;
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('last_backup_date', ?1)",
        params![today],
    )?;

    rotate_backups(backups_dir)?;
    Ok(())
}

pub fn create_backup_file(db_path: &Path, backups_dir: &Path) -> AppResult<PathBuf> {
    if !db_path.exists() {
        return Err(AppError::InvalidInput("database not found".to_string()));
    }
    fs::create_dir_all(backups_dir)?;
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let file_name = format!("backup-{timestamp}.sqlite3");
    let dest = backups_dir.join(file_name);
    fs::copy(db_path, &dest)?;
    Ok(dest)
}

pub fn rotate_backups(backups_dir: &Path) -> AppResult<()> {
    let mut entries: Vec<_> = fs::read_dir(backups_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
        .collect();
    entries.sort_by_key(|e| e.file_name());
    entries.reverse(); // newest first (lexicographic ok due to timestamp)

    for entry in entries.into_iter().skip(MAX_BACKUPS) {
        let _ = fs::remove_file(entry.path());
    }
    Ok(())
}

pub fn list_backups(backups_dir: &Path) -> AppResult<Vec<BackupInfo>> {
    if !backups_dir.exists() {
        return Ok(vec![]);
    }
    let mut items = vec![];
    for entry in fs::read_dir(backups_dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if !meta.is_file() {
            continue;
        }
        let file_name = entry.file_name().to_string_lossy().to_string();
        let created_at = meta
            .created()
            .ok()
            .and_then(|t| {
                let dt: chrono::DateTime<Utc> = t.into();
                Some(dt.to_rfc3339())
            })
            .unwrap_or_else(|| Utc::now().to_rfc3339());
        items.push(BackupInfo {
            file_name,
            full_path: entry.path().to_string_lossy().to_string(),
            created_at,
            bytes: meta.len(),
        });
    }
    items.sort_by(|a, b| b.file_name.cmp(&a.file_name));
    Ok(items)
}

pub fn restore_backup(src_path: &Path, dest_db_path: &Path) -> AppResult<()> {
    if !src_path.exists() {
        return Err(AppError::InvalidInput("backup file not found".to_string()));
    }
    fs::copy(src_path, dest_db_path)?;
    Ok(())
}

