use std::path::Path;

use rusqlite::{params, Connection};
use tauri::{AppHandle, State};

use crate::auth;
use crate::backup;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::export;
use crate::models::*;
use crate::paths;
use crate::state::AppState;

fn map_err(e: AppError) -> String {
    e.to_string()
}

fn with_conn<T>(state: &AppState, f: impl FnOnce(&Connection) -> AppResult<T>) -> AppResult<T> {
    let conn = db::open_db(&state.db_path)?;
    db::migrate(&conn)?;
    f(&conn)
}

fn require_session(state: &AppState, token: &str) -> AppResult<()> {
    if state.sessions.is_valid(token) {
        Ok(())
    } else {
        Err(AppError::Unauthorized)
    }
}

#[tauri::command]
pub fn app_status(app: AppHandle, state: State<'_, AppState>) -> Result<AppStatus, String> {
    let app_data = paths::app_data_dir(&app).map_err(map_err)?;
    let db_path = state.db_path.clone();
    let has_admin = with_conn(&state, |conn| auth::has_admin(conn)).map_err(map_err)?;

    Ok(AppStatus {
        has_admin,
        db_path: db_path.to_string_lossy().to_string(),
        app_data_dir: app_data.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub fn bootstrap_admin(password: String, state: State<'_, AppState>) -> Result<(), String> {
    with_conn(&state, |conn| auth::set_admin_password(conn, &password)).map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn request_admin_password_replace(
    state: State<'_, AppState>,
) -> Result<PasswordReplaceChallenge, String> {
    let challenge = with_conn(&state, |conn| auth::issue_password_replace_code(conn)).map_err(map_err)?;
    Ok(challenge)
}

#[tauri::command]
pub fn complete_admin_password_replace(
    code: String,
    new_password: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    with_conn(&state, |conn| auth::replace_admin_password(conn, &code, &new_password))
        .map_err(map_err)?;
    state.sessions.clear();
    Ok(())
}

#[tauri::command]
pub fn login(password: String, state: State<'_, AppState>) -> Result<AuthResult, String> {
    with_conn(&state, |conn| auth::verify_admin_password(conn, &password)).map_err(map_err)?;
    Ok(AuthResult {
        session_token: state.sessions.create_session(),
    })
}

#[tauri::command]
pub fn logout(session_token: String, state: State<'_, AppState>) -> Result<(), String> {
    state.sessions.invalidate(&session_token);
    Ok(())
}

#[tauri::command]
pub fn donors_list(session_token: String, state: State<'_, AppState>) -> Result<Vec<Donor>, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let items = with_conn(&state, |conn| {
        let mut stmt =
            conn.prepare("SELECT id, name, notes, created_at FROM donors ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Donor {
                id: row.get(0)?,
                name: row.get(1)?,
                notes: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
    .map_err(map_err)?;
    Ok(items)
}

#[tauri::command]
pub fn donors_create(
    session_token: String,
    payload: DonorCreate,
    state: State<'_, AppState>,
) -> Result<IdResult, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    if payload.name.trim().is_empty() {
        return Err("invalid input: donor name required".to_string());
    }
    let id = with_conn(&state, |conn| {
        conn.execute(
            "INSERT INTO donors (name, notes, created_at) VALUES (?1, ?2, ?3)",
            params![payload.name.trim(), payload.notes, db::now_iso()],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .map_err(map_err)?;
    Ok(IdResult { id })
}

#[tauri::command]
pub fn donors_delete(
    session_token: String,
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    with_conn(&state, |conn| {
        conn.execute("DELETE FROM donors WHERE id=?1", params![id])?;
        Ok(())
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn categories_list(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<Vec<Category>, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let items = with_conn(&state, |conn| {
        let mut stmt = conn.prepare(
            "SELECT id, name, created_at FROM categories ORDER BY name ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Category {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
    .map_err(map_err)?;
    Ok(items)
}

#[tauri::command]
pub fn categories_create(
    session_token: String,
    payload: CategoryCreate,
    state: State<'_, AppState>,
) -> Result<IdResult, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    if payload.name.trim().is_empty() {
        return Err("invalid input: category name required".to_string());
    }
    let id = with_conn(&state, |conn| {
        conn.execute(
            "INSERT INTO categories (name, created_at) VALUES (?1, ?2)",
            params![payload.name.trim(), db::now_iso()],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .map_err(map_err)?;
    Ok(IdResult { id })
}

#[tauri::command]
pub fn projects_list(
    session_token: String,
    state: State<'_, AppState>,
) -> Result<Vec<Project>, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let items = with_conn(&state, |conn| {
        let mut stmt = conn.prepare(
            r#"
SELECT id, name, description, target_amount_cents, status, start_date, end_date, created_at
FROM projects
ORDER BY status ASC, name ASC
"#,
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                target_amount_cents: row.get(3)?,
                status: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
    .map_err(map_err)?;
    Ok(items)
}

#[tauri::command]
pub fn projects_create(
    session_token: String,
    payload: ProjectCreate,
    state: State<'_, AppState>,
) -> Result<IdResult, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    if payload.name.trim().is_empty() {
        return Err("invalid input: project name required".to_string());
    }
    let status = payload.status.unwrap_or_else(|| "active".to_string());
    let id = with_conn(&state, |conn| {
        conn.execute(
            r#"
INSERT INTO projects (name, description, target_amount_cents, status, start_date, end_date, created_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#,
            params![
                payload.name.trim(),
                payload.description,
                payload.target_amount_cents,
                status,
                payload.start_date,
                payload.end_date,
                db::now_iso()
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .map_err(map_err)?;
    Ok(IdResult { id })
}

#[tauri::command]
pub fn projects_update(
    session_token: String,
    id: i64,
    payload: ProjectUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    if payload.name.trim().is_empty() {
        return Err("invalid input: project name required".to_string());
    }
    with_conn(&state, |conn| {
        conn.execute(
            r#"
UPDATE projects
SET name=?1, description=?2, target_amount_cents=?3, status=?4, start_date=?5, end_date=?6
WHERE id=?7
"#,
            params![
                payload.name.trim(),
                payload.description,
                payload.target_amount_cents,
                payload.status,
                payload.start_date,
                payload.end_date,
                id
            ],
        )?;
        Ok(())
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn projects_delete(
    session_token: String,
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    with_conn(&state, |conn| {
        conn.execute("DELETE FROM projects WHERE id=?1", params![id])?;
        Ok(())
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn donations_list(
    session_token: String,
    filter: DateRangeFilter,
    state: State<'_, AppState>,
) -> Result<Vec<Donation>, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let items = with_conn(&state, |conn| {
        let mut stmt = conn.prepare(
            r#"
SELECT id, donated_at, amount_cents, donor_id, anonymous, notes, project_id, created_at
FROM donations
WHERE (?1 IS NULL OR donated_at >= ?1)
  AND (?2 IS NULL OR donated_at <= ?2)
  AND (?3 IS NULL OR project_id = ?3)
ORDER BY donated_at DESC, id DESC
LIMIT 500
"#,
        )?;
        let rows = stmt.query_map(params![filter.from, filter.to, filter.project_id], |row| {
            Ok(Donation {
                id: row.get(0)?,
                donated_at: row.get(1)?,
                amount_cents: row.get(2)?,
                donor_id: row.get(3)?,
                anonymous: row.get::<_, i64>(4)? != 0,
                notes: row.get(5)?,
                project_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
    .map_err(map_err)?;
    Ok(items)
}

#[tauri::command]
pub fn donations_create(
    session_token: String,
    payload: DonationCreate,
    state: State<'_, AppState>,
) -> Result<IdResult, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    validate_date(&payload.donated_at)?;
    if payload.amount_cents <= 0 {
        return Err("invalid input: donation amount must be > 0".to_string());
    }
    let id = with_conn(&state, |conn| {
        conn.execute(
            r#"
INSERT INTO donations (donated_at, amount_cents, donor_id, anonymous, notes, project_id, created_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#,
            params![
                payload.donated_at,
                payload.amount_cents,
                payload.donor_id,
                if payload.anonymous { 1 } else { 0 },
                payload.notes,
                payload.project_id,
                db::now_iso()
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .map_err(map_err)?;
    Ok(IdResult { id })
}

#[tauri::command]
pub fn donations_update(
    session_token: String,
    id: i64,
    payload: DonationUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    validate_date(&payload.donated_at)?;
    if payload.amount_cents <= 0 {
        return Err("invalid input: donation amount must be > 0".to_string());
    }
    with_conn(&state, |conn| {
        conn.execute(
            r#"
UPDATE donations
SET donated_at=?1, amount_cents=?2, donor_id=?3, anonymous=?4, notes=?5, project_id=?6
WHERE id=?7
"#,
            params![
                payload.donated_at,
                payload.amount_cents,
                payload.donor_id,
                if payload.anonymous { 1 } else { 0 },
                payload.notes,
                payload.project_id,
                id
            ],
        )?;
        Ok(())
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn donations_delete(
    session_token: String,
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    with_conn(&state, |conn| {
        conn.execute("DELETE FROM donations WHERE id=?1", params![id])?;
        Ok(())
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn expenses_list(
    session_token: String,
    filter: DateRangeFilter,
    state: State<'_, AppState>,
) -> Result<Vec<Expense>, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let items = with_conn(&state, |conn| {
        let mut stmt = conn.prepare(
            r#"
SELECT id, spent_at, amount_cents, category_id, payee, notes, project_id, created_at
FROM expenses
WHERE (?1 IS NULL OR spent_at >= ?1)
  AND (?2 IS NULL OR spent_at <= ?2)
  AND (?3 IS NULL OR project_id = ?3)
ORDER BY spent_at DESC, id DESC
LIMIT 500
"#,
        )?;
        let rows = stmt.query_map(params![filter.from, filter.to, filter.project_id], |row| {
            Ok(Expense {
                id: row.get(0)?,
                spent_at: row.get(1)?,
                amount_cents: row.get(2)?,
                category_id: row.get(3)?,
                payee: row.get(4)?,
                notes: row.get(5)?,
                project_id: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
    .map_err(map_err)?;
    Ok(items)
}

#[tauri::command]
pub fn expenses_create(
    session_token: String,
    payload: ExpenseCreate,
    state: State<'_, AppState>,
) -> Result<IdResult, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    validate_date(&payload.spent_at)?;
    if payload.amount_cents <= 0 {
        return Err("invalid input: expense amount must be > 0".to_string());
    }
    let id = with_conn(&state, |conn| {
        conn.execute(
            r#"
INSERT INTO expenses (spent_at, amount_cents, category_id, payee, notes, project_id, created_at)
VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
"#,
            params![
                payload.spent_at,
                payload.amount_cents,
                payload.category_id,
                payload.payee,
                payload.notes,
                payload.project_id,
                db::now_iso()
            ],
        )?;
        Ok(conn.last_insert_rowid())
    })
    .map_err(map_err)?;
    Ok(IdResult { id })
}

#[tauri::command]
pub fn expenses_update(
    session_token: String,
    id: i64,
    payload: ExpenseUpdate,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    validate_date(&payload.spent_at)?;
    if payload.amount_cents <= 0 {
        return Err("invalid input: expense amount must be > 0".to_string());
    }
    with_conn(&state, |conn| {
        conn.execute(
            r#"
UPDATE expenses
SET spent_at=?1, amount_cents=?2, category_id=?3, payee=?4, notes=?5, project_id=?6
WHERE id=?7
"#,
            params![
                payload.spent_at,
                payload.amount_cents,
                payload.category_id,
                payload.payee,
                payload.notes,
                payload.project_id,
                id
            ],
        )?;
        Ok(())
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn expenses_delete(
    session_token: String,
    id: i64,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    with_conn(&state, |conn| {
        conn.execute("DELETE FROM expenses WHERE id=?1", params![id])?;
        Ok(())
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn ledger_summary(
    session_token: String,
    filter: DateRangeFilter,
    state: State<'_, AppState>,
) -> Result<LedgerSummary, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let summary = with_conn(&state, |conn| export::compute_summary(conn, &filter)).map_err(map_err)?;
    Ok(summary)
}

#[tauri::command]
pub fn project_balances(
    session_token: String,
    filter: DateRangeFilter,
    state: State<'_, AppState>,
) -> Result<Vec<ProjectBalanceRow>, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let items = with_conn(&state, |conn| {
        let mut stmt = conn.prepare(
            r#"
SELECT p.id, p.name,
  COALESCE((SELECT SUM(d.amount_cents) FROM donations d
            WHERE d.project_id=p.id
              AND (?1 IS NULL OR d.donated_at >= ?1)
              AND (?2 IS NULL OR d.donated_at <= ?2)), 0) AS donations_cents,
  COALESCE((SELECT SUM(e.amount_cents) FROM expenses e
            WHERE e.project_id=p.id
              AND (?1 IS NULL OR e.spent_at >= ?1)
              AND (?2 IS NULL OR e.spent_at <= ?2)), 0) AS expenses_cents
FROM projects p
ORDER BY p.status ASC, p.name ASC
"#,
        )?;
        let rows = stmt.query_map(params![filter.from, filter.to], |row| {
            let donations_cents: i64 = row.get(2)?;
            let expenses_cents: i64 = row.get(3)?;
            Ok(ProjectBalanceRow {
                project_id: row.get(0)?,
                project_name: row.get(1)?,
                donations_cents,
                expenses_cents,
                balance_cents: donations_cents - expenses_cents,
            })
        })?;
        let mut out = vec![];
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    })
    .map_err(map_err)?;
    Ok(items)
}

#[tauri::command]
pub fn project_report(
    session_token: String,
    project_id: i64,
    filter: DateRangeFilter,
    state: State<'_, AppState>,
) -> Result<ProjectReport, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let report = with_conn(&state, |conn| build_project_report(conn, project_id, &filter))
        .map_err(map_err)?;
    Ok(report)
}

#[tauri::command]
pub fn export_csv_command(
    session_token: String,
    req: ExportCsvRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    with_conn(&state, |conn| export::export_csv(conn, req)).map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn export_pdf_command(
    session_token: String,
    req: ExportPdfRequest,
    state: State<'_, AppState>,
) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    with_conn(&state, |conn| {
        let summary = export::compute_summary(conn, &req.filter)?;
        export::export_pdf_summary(conn, req, summary)
    })
    .map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn backup_list(session_token: String, app: AppHandle, state: State<'_, AppState>) -> Result<Vec<BackupInfo>, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let dir = paths::backups_dir(&app).map_err(map_err)?;
    backup::list_backups(&dir).map_err(map_err)
}

#[tauri::command]
pub fn backup_create(session_token: String, app: AppHandle, state: State<'_, AppState>) -> Result<String, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let dir = paths::backups_dir(&app).map_err(map_err)?;
    let path = backup::create_backup_file(&state.db_path, &dir).map_err(map_err)?;
    backup::rotate_backups(&dir).map_err(map_err)?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub fn backup_restore(session_token: String, src_path: String, state: State<'_, AppState>) -> Result<(), String> {
    require_session(&state, &session_token).map_err(map_err)?;
    backup::restore_backup(Path::new(&src_path), &state.db_path).map_err(map_err)?;
    // Validate/migrate after restore
    let conn = db::open_db(&state.db_path).map_err(map_err)?;
    db::migrate(&conn).map_err(map_err)?;
    Ok(())
}

#[tauri::command]
pub fn database_health(session_token: String, state: State<'_, AppState>) -> Result<DatabaseHealth, String> {
    require_session(&state, &session_token).map_err(map_err)?;
    let health = with_conn(&state, |conn| {
        let integrity_result: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        let integrity_ok = integrity_result.trim().eq_ignore_ascii_case("ok");

        let mut record_count = 0i64;
        for table in ["meta", "admins", "donors", "projects", "categories", "donations", "expenses"] {
            let sql = format!("SELECT COUNT(*) FROM {table}");
            let count: i64 = conn.query_row(&sql, [], |row| row.get(0))?;
            record_count += count;
        }

        Ok(DatabaseHealth {
            integrity_ok,
            checked_at: db::now_iso(),
            record_count,
        })
    })
    .map_err(map_err)?;
    Ok(health)
}

fn validate_date(date: &str) -> Result<(), String> {
    // Minimal YYYY-MM-DD validation
    if date.len() != 10 {
        return Err("invalid input: date must be YYYY-MM-DD".to_string());
    }
    let bytes = date.as_bytes();
    let ok = bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[0..4].iter().all(|b| b.is_ascii_digit())
        && bytes[5..7].iter().all(|b| b.is_ascii_digit())
        && bytes[8..10].iter().all(|b| b.is_ascii_digit());
    if !ok {
        return Err("invalid input: date must be YYYY-MM-DD".to_string());
    }
    Ok(())
}

fn build_project_report(
    conn: &Connection,
    project_id: i64,
    filter: &DateRangeFilter,
) -> AppResult<ProjectReport> {
    let project = conn.query_row(
        r#"
SELECT id, name, description, target_amount_cents, status, start_date, end_date, created_at
FROM projects WHERE id=?1
"#,
        params![project_id],
        |row| {
            Ok(Project {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                target_amount_cents: row.get(3)?,
                status: row.get(4)?,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                created_at: row.get(7)?,
            })
        },
    )?;

    let donations_cents: i64 = conn.query_row(
        r#"
SELECT COALESCE(SUM(amount_cents), 0)
FROM donations
WHERE project_id=?1
  AND (?2 IS NULL OR donated_at >= ?2)
  AND (?3 IS NULL OR donated_at <= ?3)
"#,
        params![project_id, filter.from, filter.to],
        |row| row.get(0),
    )?;
    let expenses_cents: i64 = conn.query_row(
        r#"
SELECT COALESCE(SUM(amount_cents), 0)
FROM expenses
WHERE project_id=?1
  AND (?2 IS NULL OR spent_at >= ?2)
  AND (?3 IS NULL OR spent_at <= ?3)
"#,
        params![project_id, filter.from, filter.to],
        |row| row.get(0),
    )?;
    let balance_cents = donations_cents - expenses_cents;
    let target_amount_cents = project.target_amount_cents;
    let remaining_to_target_cents = (target_amount_cents - donations_cents).max(0);

    let mut donations_stmt = conn.prepare(
        r#"
SELECT d.id, d.donated_at, d.amount_cents,
       CASE WHEN d.anonymous=1 THEN NULL ELSE donors.name END AS donor_name,
       d.anonymous, d.notes
FROM donations d
LEFT JOIN donors ON donors.id = d.donor_id
WHERE d.project_id=?1
  AND (?2 IS NULL OR d.donated_at >= ?2)
  AND (?3 IS NULL OR d.donated_at <= ?3)
ORDER BY d.donated_at DESC, d.id DESC
LIMIT 500
"#,
    )?;
    let donation_rows = donations_stmt.query_map(
        params![project_id, filter.from, filter.to],
        |row| {
            Ok(DonationRow {
                id: row.get(0)?,
                donated_at: row.get(1)?,
                amount_cents: row.get(2)?,
                donor_name: row.get(3)?,
                anonymous: row.get::<_, i64>(4)? != 0,
                notes: row.get(5)?,
            })
        },
    )?;
    let mut donations = vec![];
    for r in donation_rows {
        donations.push(r?);
    }

    let mut expenses_stmt = conn.prepare(
        r#"
SELECT e.id, e.spent_at, e.amount_cents, c.name as category_name, e.payee, e.notes
FROM expenses e
LEFT JOIN categories c ON c.id = e.category_id
WHERE e.project_id=?1
  AND (?2 IS NULL OR e.spent_at >= ?2)
  AND (?3 IS NULL OR e.spent_at <= ?3)
ORDER BY e.spent_at DESC, e.id DESC
LIMIT 500
"#,
    )?;
    let expense_rows = expenses_stmt.query_map(
        params![project_id, filter.from, filter.to],
        |row| {
            Ok(ExpenseRow {
                id: row.get(0)?,
                spent_at: row.get(1)?,
                amount_cents: row.get(2)?,
                category_name: row.get(3)?,
                payee: row.get(4)?,
                notes: row.get(5)?,
            })
        },
    )?;
    let mut expenses = vec![];
    for r in expense_rows {
        expenses.push(r?);
    }

    let mut top_stmt = conn.prepare(
        r#"
SELECT donors.name, COALESCE(SUM(d.amount_cents), 0) as total
FROM donations d
JOIN donors ON donors.id = d.donor_id
WHERE d.project_id=?1
  AND d.anonymous=0
  AND (?2 IS NULL OR d.donated_at >= ?2)
  AND (?3 IS NULL OR d.donated_at <= ?3)
GROUP BY donors.name
ORDER BY total DESC
LIMIT 10
"#,
    )?;
    let top_rows = top_stmt.query_map(params![project_id, filter.from, filter.to], |row| {
        Ok(TopDonorRow {
            donor_name: row.get(0)?,
            total_cents: row.get(1)?,
        })
    })?;
    let mut top_donors = vec![];
    for r in top_rows {
        top_donors.push(r?);
    }

    Ok(ProjectReport {
        project,
        donations_cents,
        expenses_cents,
        balance_cents,
        target_amount_cents,
        remaining_to_target_cents,
        donations,
        expenses,
        top_donors,
    })
}
