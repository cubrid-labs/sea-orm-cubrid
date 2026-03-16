//! Error helpers for converting CUBRID driver errors to SeaORM errors.

use sea_orm::{DbErr, RuntimeErr};
use thiserror::Error;

/// Error type for CUBRID proxy backend operations.
#[derive(Debug, Error)]
pub enum CubridProxyError {
    /// Error from the underlying async CUBRID client.
    #[error("CUBRID driver error: {0}")]
    Driver(#[from] cubrid_tokio::Error),
}

/// Convert a `cubrid_tokio::Error` into SeaORM's [`DbErr`].
pub fn into_db_err(error: cubrid_tokio::Error) -> DbErr {
    DbErr::Conn(RuntimeErr::Internal(error.to_string()))
}
