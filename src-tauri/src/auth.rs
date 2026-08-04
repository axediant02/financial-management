use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use rand::rngs::OsRng;
use rand::Rng;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::now_iso;
use crate::error::{AppError, AppResult};
use crate::models::PasswordReplaceChallenge;

pub fn has_admin(conn: &Connection) -> AppResult<bool> {
    let count: i64 = conn.query_row("SELECT COUNT(1) FROM admins", [], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn set_admin_password(conn: &Connection, password: &str) -> AppResult<()> {
    if has_admin(conn)? {
        return Err(AppError::AdminAlreadyInitialized);
    }
    validate_password(password)?;

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::InvalidInput(format!("password hash failed: {e}")))?;

    conn.execute(
        "INSERT INTO admins (password_hash, created_at) VALUES (?1, ?2)",
        params![hash.to_string(), now_iso()],
    )?;
    Ok(())
}

pub fn verify_admin_password(conn: &Connection, password: &str) -> AppResult<()> {
    let hash: Option<String> = conn
        .query_row(
            "SELECT password_hash FROM admins ORDER BY id ASC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .optional()?;

    let Some(hash) = hash else {
        return Err(AppError::AdminNotInitialized);
    };

    let parsed_hash =
        PasswordHash::new(&hash).map_err(|_| AppError::InvalidCredentials)?;
    let argon2 = Argon2::default();
    argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| AppError::InvalidCredentials)?;

    Ok(())
}

pub fn issue_password_replace_code(conn: &Connection) -> AppResult<PasswordReplaceChallenge> {
    if !has_admin(conn)? {
        return Err(AppError::AdminNotInitialized);
    }

    conn.execute("DELETE FROM admin_password_resets WHERE used_at IS NULL", [])?;

    let code = format!("{:06}", rand::thread_rng().gen_range(0..=999_999));
    let expires_at = (Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();
    conn.execute(
        r#"
INSERT INTO admin_password_resets (code, created_at, expires_at, used_at)
VALUES (?1, ?2, ?3, NULL)
"#,
        params![code, now_iso(), expires_at.clone()],
    )?;

    Ok(PasswordReplaceChallenge { code, expires_at })
}

pub fn replace_admin_password(conn: &Connection, code: &str, new_password: &str) -> AppResult<()> {
    validate_password(new_password)?;

    let now = now_iso();
    let row: Option<i64> = conn
        .query_row(
            r#"
SELECT id
FROM admin_password_resets
WHERE code=?1 AND used_at IS NULL AND expires_at > ?2
ORDER BY id DESC
LIMIT 1
"#,
            params![code.trim(), now.clone()],
            |row| row.get(0),
        )
        .optional()?;

    let Some(reset_id) = row else {
        return Err(AppError::InvalidCredentials);
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| AppError::InvalidInput(format!("password hash failed: {e}")))?;

    conn.execute("DELETE FROM admins", [])?;
    conn.execute(
        "INSERT INTO admins (password_hash, created_at) VALUES (?1, ?2)",
        params![hash.to_string(), now.clone()],
    )?;
    conn.execute(
        "UPDATE admin_password_resets SET used_at=?1 WHERE id=?2",
        params![now, reset_id],
    )?;
    Ok(())
}

fn validate_password(password: &str) -> AppResult<()> {
    if password.trim().len() < 8 {
        return Err(AppError::InvalidInput(
            "password must be at least 8 characters".to_string(),
        ));
    }
    Ok(())
}

