use axum::response::{IntoResponse, Response};
use serde::Serialize;
use sqlx::PgPool;

use crate::s3serv::error::S3Error;

#[derive(serde::Serialize)]
struct ListBucketResult {
    #[serde(rename = "Contents")]
    contents: Vec<Content>,
}

#[derive(serde::Serialize)]
struct Content {
    #[serde(rename = "Key")]
    key: String,
    #[serde(rename = "Size")]
    size: u64,
}

#[tracing::instrument(skip(pool))]
pub async fn list_objects(pool: PgPool, bucket: String, prefix: String) -> Response {
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
        SELECT files.key, COALESCE(file_data.size, 0) AS "size!"
        FROM files
            LEFT JOIN file_versions ON file_versions.id = files.current_version
            LEFT JOIN file_data ON file_data.id = file_versions.file_data_id
        WHERE
            files.bucket_id = $1
            AND files.key LIKE $2
    "#,
        result.id,
        format!("{}%", prefix)
    )
    .fetch_all(&pool)
    .await;

    let result = match result {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to fetch objects: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let result = ListBucketResult {
        contents: result
            .into_iter()
            .map(|object| Content {
                key: object.key,
                size: object.size as u64,
            })
            .collect(),
    };

    let mut buffer = String::new();
    let serializer = quick_xml::se::Serializer::new(&mut buffer);
    result.serialize(serializer).expect("Failed to serialize.");

    ([("Content-Type", "application/xml")], buffer).into_response()
}
