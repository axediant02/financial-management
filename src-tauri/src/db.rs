use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{AppError, AppResult};

pub const SCHEMA_VERSION: i64 = 4;

pub fn open_db(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(conn)
}

pub fn migrate(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        r#"
CREATE TABLE IF NOT EXISTS meta (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);
"#,
    )?;

    let current_version: i64 = conn
        .query_row(
            "SELECT value FROM meta WHERE key='schema_version'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()?
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    if current_version > SCHEMA_VERSION {
        return Err(AppError::InvalidInput(format!(
            "db schema_version {current_version} is newer than app {SCHEMA_VERSION}"
        )));
    }

    if current_version < 1 {
        conn.execute_batch(
            r#"
CREATE TABLE IF NOT EXISTS admins (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  password_hash TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS donors (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  notes TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS projects (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  description TEXT,
  target_amount_cents INTEGER NOT NULL DEFAULT 0,
  status TEXT NOT NULL DEFAULT 'active',
  start_date TEXT,
  end_date TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS documentations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_name TEXT NOT NULL,
  event_date TEXT NOT NULL,
  registration_fee_cents INTEGER NOT NULL DEFAULT 0,
  registration_collected_cents INTEGER NOT NULL DEFAULT 0,
  notes TEXT,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS donations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  donated_at TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  donor_id INTEGER,
  anonymous INTEGER NOT NULL DEFAULT 0,
  notes TEXT,
  project_id INTEGER,
  created_at TEXT NOT NULL,
  FOREIGN KEY(donor_id) REFERENCES donors(id) ON DELETE SET NULL,
  FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS expenses (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  spent_at TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  category_id INTEGER,
  payee TEXT,
  notes TEXT,
  project_id INTEGER,
  created_at TEXT NOT NULL,
  FOREIGN KEY(category_id) REFERENCES categories(id) ON DELETE SET NULL,
  FOREIGN KEY(project_id) REFERENCES projects(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_donations_donated_at ON donations(donated_at);
CREATE INDEX IF NOT EXISTS idx_donations_project_id ON donations(project_id);
CREATE INDEX IF NOT EXISTS idx_expenses_spent_at ON expenses(spent_at);
CREATE INDEX IF NOT EXISTS idx_expenses_project_id ON expenses(project_id);
CREATE INDEX IF NOT EXISTS idx_documentations_event_date ON documentations(event_date);

INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', '1');
"#,
        )?;

        let now = now_iso();
        for name in [
            "Utilities",
            "Supplies",
            "Maintenance",
            "Events",
            "Instruments",
            "Other",
        ] {
            conn.execute(
                "INSERT OR IGNORE INTO categories (name, created_at) VALUES (?1, ?2)",
                params![name, now],
            )?;
        }
    }

    if current_version < 2 {
        conn.execute_batch(
            r#"
CREATE TABLE IF NOT EXISTS admin_password_resets (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  code TEXT NOT NULL,
  created_at TEXT NOT NULL,
  expires_at TEXT NOT NULL,
  used_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_admin_password_resets_code ON admin_password_resets(code);
CREATE INDEX IF NOT EXISTS idx_admin_password_resets_expires_at ON admin_password_resets(expires_at);

INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', '2');
"#,
        )?;
    }

    if current_version < 3 {
        conn.execute_batch(
            r#"
CREATE TABLE IF NOT EXISTS documentations (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  event_name TEXT NOT NULL,
  event_date TEXT NOT NULL,
  registration_fee_cents INTEGER NOT NULL DEFAULT 0,
  registration_collected_cents INTEGER NOT NULL DEFAULT 0,
  notes TEXT,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_documentations_event_date ON documentations(event_date);

INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', '3');
"#,
        )?;
    }

    if current_version < 4 {
        if !column_exists(conn, "documentations", "registration_collected_cents")? {
            conn.execute(
                "ALTER TABLE documentations ADD COLUMN registration_collected_cents INTEGER NOT NULL DEFAULT 0",
                [],
            )?;
        }
        conn.execute(
            "UPDATE documentations SET registration_collected_cents = COALESCE(registration_fee_cents, 0) WHERE registration_collected_cents = 0",
            [],
        )?;
        conn.execute_batch(
            r#"
CREATE TABLE IF NOT EXISTS documentation_expenses (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  documentation_id INTEGER NOT NULL,
  spent_at TEXT NOT NULL,
  amount_cents INTEGER NOT NULL,
  payee TEXT,
  notes TEXT,
  created_at TEXT NOT NULL,
  FOREIGN KEY(documentation_id) REFERENCES documentations(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_documentation_expenses_documentation_id ON documentation_expenses(documentation_id);
CREATE INDEX IF NOT EXISTS idx_documentation_expenses_spent_at ON documentation_expenses(spent_at);

INSERT OR REPLACE INTO meta (key, value) VALUES ('schema_version', '4');
"#,
        )?;
    }

    Ok(())
}

fn column_exists(conn: &Connection, table: &str, column: &str) -> AppResult<bool> {
    let sql = format!("PRAGMA table_info({table})");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
    for row in rows {
        if row?.eq_ignore_ascii_case(column) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn now_iso() -> String {
    let now: DateTime<Utc> = Utc::now();
    now.to_rfc3339()
}
