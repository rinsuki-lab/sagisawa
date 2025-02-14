use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::s3serv::error::S3Error;

pub async fn delete_bucket(pool: PgPool, bucket: String) -> Response {
    let res = sqlx::query!("DELETE FROM buckets WHERE name = $1", bucket)
        .execute(&pool)
        .await;

    match res {
        Ok(res) => {
            if res.rows_affected() == 0 {
                return S3Error::NoSuchBucket.into_response();
            }
            return StatusCode::NO_CONTENT.into_response();
        }
        Err(e) => {
            tracing::error!("failed to delete bucket {}: {:?}", bucket, e);
            return S3Error::InternalError.into_response();
        }
    };
}
