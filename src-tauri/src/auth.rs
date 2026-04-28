use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;
use rusqlite::{params, Connection, OptionalExtension};

use crate::db::now_iso;
use crate::error::{AppError, AppResult};

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

fn validate_password(password: &str) -> AppResult<()> {
    if password.trim().len() < 8 {
        return Err(AppError::InvalidInput(
            "password must be at least 8 characters".to_string(),
        ));
    }
    Ok(())
}

