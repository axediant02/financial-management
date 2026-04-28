use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("admin already initialized")]
    AdminAlreadyInitialized,

    #[error("admin not initialized")]
    AdminNotInitialized,

    #[error("unauthorized")]
    Unauthorized,

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("csv error: {0}")]
    Csv(#[from] csv::Error),

    #[error("pdf error: {0}")]
    Pdf(#[from] printpdf::Error),
}

pub type AppResult<T> = Result<T, AppError>;
