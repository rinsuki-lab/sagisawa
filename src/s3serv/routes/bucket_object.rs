use axum::{
    body::Body,
    extract::{Path, State},
    response::Response,
};
use sqlx::PgPool;

use crate::s3serv::actions;

pub fn bucket_object() -> axum::routing::MethodRouter<PgPool> {
    axum::routing::head(head_bucket_object)
        .get(get_bucket_object)
        .put(put_bucket_object)
}

pub async fn head_bucket_object(
    Path((bucket, key)): Path<(String, String)>,
    State(pool): State<PgPool>,
) -> Response {
    tracing::debug!("bucket: {}, key: {}", bucket, key);

    actions::head_object(pool, bucket, key).await
}

pub async fn get_bucket_object(
    Path((bucket, key)): Path<(String, String)>,
    State(pool): State<PgPool>,
) -> Response {
    tracing::debug!("bucket: {}, key: {}", bucket, key);

    actions::get_object(pool, bucket, key).await
}

pub async fn put_bucket_object(
    Path((bucket, key)): Path<(String, String)>,
    State(pool): State<PgPool>,
    body: Body,
) -> Response {
    tracing::debug!("bucket: {}, key: {}", bucket, key);

    let mut body = body.into_data_stream();

    actions::put_object(pool, bucket, key, &mut body).await
}
