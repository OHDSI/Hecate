use actix_web::{HttpResponse, ResponseError};
use deadpool_postgres::PoolError;
use thiserror::Error;
use tokio_pg_mapper::Error as PGMError;
use tokio_postgres::error::Error as PGError;

#[derive(Debug, Error)]
pub enum PgError {
    #[error("resource not found")]
    NotFound,
    #[error("database error: {0}")]
    PGError(#[from] PGError),
    #[error("row mapping error: {0}")]
    PGMError(#[from] PGMError),
    #[error("connection pool error: {0}")]
    PoolError(#[from] PoolError),
}

impl ResponseError for PgError {
    fn error_response(&self) -> HttpResponse {
        match self {
            PgError::NotFound => HttpResponse::NotFound().finish(),
            PgError::PoolError(err) => HttpResponse::InternalServerError().body(err.to_string()),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
