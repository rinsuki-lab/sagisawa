use axum::{
    body::Body,
    response::{IntoResponse, Response},
};
use sqlx::{postgres::types::PgRange, PgPool};

use crate::s3serv::error::S3Error;

#[tracing::instrument]
pub async fn head_object(pool: PgPool, bucket: String, key: String) -> Response {
    let result = sqlx::query!("SELECT id FROM buckets WHERE name = $1 LIMIT 1", bucket)
        .fetch_one(&pool)
        .await;

    let result = match result {
        Ok(v) => v,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                return S3Error::NoSuchBucket.into_response();
            }
            tracing::error!("Failed to fetch bucket: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let result = sqlx::query!(
        r#"
        SELECT
            file_versions.id as version_id,
            file_data.id as data_id,
            file_data.size, file_data.md5
        FROM files
            JOIN file_versions ON files.current_version = file_versions.id
            JOIN file_data ON file_versions.file_data_id = file_data.id
        WHERE
            files.bucket_id = $1
            AND files.key = $2
            AND files.current_version_is_delete_marker = FALSE
        LIMIT 1
    "#,
        result.id,
        key
    )
    .fetch_one(&pool)
    .await;

    let result = match result {
        Ok(v) => v,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                return S3Error::NoSuchKey.into_response();
            }
            tracing::error!("Failed to fetch file: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    println!("{:?}", result);

    if result.size == 0 {
        return axum::http::StatusCode::OK.into_response();
    }

    let requested_range = 0..result.size; // TODO: HTTP Range header handling

    (
        axum::http::StatusCode::OK,
        [
            ("Content-Type", "video/mp4"),
            ("Content-Length", result.size.to_string().as_str()),
            ("ETag", format!("\"{}\"", hex::encode(result.md5)).as_str()),
        ],
    )
        .into_response()
}

#[tracing::instrument]
pub async fn get_object(pool: PgPool, bucket: String, key: String) -> Response {
    let result = sqlx::query!("SELECT id FROM buckets WHERE name = $1 LIMIT 1", bucket)
        .fetch_one(&pool)
        .await;

    let result = match result {
        Ok(v) => v,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                return S3Error::NoSuchBucket.into_response();
            }
            tracing::error!("Failed to fetch bucket: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let result = sqlx::query!(
        r#"
        SELECT
            file_versions.id as version_id,
            file_data.id as data_id,
            file_data.size, file_data.md5
        FROM files
            JOIN file_versions ON files.current_version = file_versions.id
            JOIN file_data ON file_versions.file_data_id = file_data.id
        WHERE
            files.bucket_id = $1
            AND files.key = $2
            AND files.current_version_is_delete_marker = FALSE
        LIMIT 1
    "#,
        result.id,
        key
    )
    .fetch_one(&pool)
    .await;

    let result = match result {
        Ok(v) => v,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                return S3Error::NoSuchKey.into_response();
            }
            tracing::error!("Failed to fetch file: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    println!("{:?}", result);

    if result.size == 0 {
        return axum::http::StatusCode::OK.into_response();
    }

    let requested_range = 0..result.size; // TODO: HTTP Range header handling

    let parts = sqlx::query!(
        "SELECT backend_key, range FROM file_data_parts WHERE file_data_id = $1 AND range && $2",
        result.data_id,
        PgRange::from(requested_range.clone())
    )
    .fetch_one(&pool)
    .await;

    let parts = match parts {
        Ok(v) => v,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                tracing::error!("Requested range not found: {:?}", requested_range);
                return S3Error::InternalError.into_response();
            }
            tracing::error!("Failed to fetch file parts: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let stream = async_stream::stream! {
        let mut offset = requested_range.start;
        let client = reqwest::Client::new();
        while offset < requested_range.end {
            let res = client.get(format!("http://localhost:4000/v1/files/{}/chunks/{}", parts.backend_key, offset))
                .send()
                .await;

            let res = match res {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to fetch file part: {:?}", e);
                    yield Err(e);
                    return;
                }
            };

            let res = match res.error_for_status() {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to fetch file part: {:?}", e);
                    yield Err(e);
                    return;
                }
            };

            let res = res.bytes().await;

            let res = match res {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to read file part: {:?}", e);
                    yield Err(e);
                    return;
                }
            };

            offset += res.len() as i64;
            yield Ok(res);
        }
    };

    (
        axum::http::StatusCode::OK,
        [
            ("Content-Type", "video/mp4"), // TODO: correctly handles metadata on both of get_ and put_object
            ("Content-Length", result.size.to_string().as_str()),
            ("ETag", format!("\"{}\"", hex::encode(result.md5)).as_str()),
        ],
        Body::from_stream(stream),
    )
        .into_response()
}
