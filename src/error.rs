use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Db(#[from] rusqlite::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("data directory unavailable on this platform")]
    NoDataDir,
}

pub type Result<T> = std::result::Result<T, AppError>;
