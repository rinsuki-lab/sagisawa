use axum::response::{IntoResponse, Response};
use sqlx::PgPool;

use crate::s3serv::error::S3Error;

#[tracing::instrument]
pub async fn create_bucket(pool: PgPool, bucket: String) -> Response {
    let res = sqlx::query!("INSERT INTO buckets(name) VALUES ($1)", bucket)
        .execute(&pool)
        .await;

    match res {
        Ok(_) => {
            return [("Location", format!("/{}", bucket))].into_response();
        }
        Err(e) => {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    tracing::warn!("bucket {} already exists", bucket);
                    return S3Error::BucketAlreadyExists.into_response();
                }
            }
            tracing::error!("failed to create bucket {}: {:?}", bucket, e);
            return S3Error::InternalError.into_response();
        }
    };
}
