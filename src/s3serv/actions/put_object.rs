use axum::{
    body::BodyDataStream,
    response::{IntoResponse, Response},
};
use reqwest::StatusCode;
use sqlx::{postgres::types::PgRange, PgPool};

use crate::{drivers, s3serv::error::S3Error};

#[tracing::instrument(skip(pool, body))]
pub async fn put_object(
    pool: PgPool,
    bucket: String,
    key: String,
    body: &mut BodyDataStream,
) -> Response {
    let tx = pool.begin().await;

    let mut tx = match tx {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to start transaction: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let result = sqlx::query!("SELECT id FROM buckets WHERE name = $1 LIMIT 1", bucket)
        .fetch_one(&mut *tx)
        .await;

    let bucket_id = match result {
        Ok(v) => v.id,
        Err(e) => {
            if let sqlx::Error::RowNotFound = e {
                return S3Error::NoSuchBucket.into_response();
            }
            tracing::error!("Failed to fetch bucket: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let result = drivers::ton::upload_from_stream(&tx, body).await;

    let result = match result {
        Err(e) => {
            tracing::error!("Failed to upload object");
            return e;
        }
        Ok(v) => v,
    };

    let file_data_id = match result {
        None => None,
        Some(result) => {
            let data_id = sqlx::query!(
                "INSERT INTO file_data(size, md5) VALUES($1, $2) RETURNING id",
                result.size as i64,
                &result.md5
            )
            .fetch_one(&mut *tx)
            .await;

            let data_id = match data_id {
                Ok(v) => v.id,
                Err(e) => {
                    tracing::error!("Failed to insert file data: {:?}", e);
                    return S3Error::InternalError.into_response();
                }
            };

            let part_id = sqlx::query!(
                "INSERT INTO file_data_parts(file_data_id, backend_key, range) VALUES($1, $2, $3) RETURNING id",
                data_id,
                result.r#ref,
                PgRange::from(0..(result.size as i64))
            )
            .fetch_one(&mut *tx)
            .await;

            let part_id = match part_id {
                Ok(rec) => rec.id,
                Err(e) => {
                    tracing::error!("Failed to insert file data part: {:?}", e);
                    return S3Error::InternalError.into_response();
                }
            };

            let mut builder = sqlx::QueryBuilder::new(
                "INSERT INTO file_data_part_chunk_info (part_id, range, md5, sha256) ",
            );

            builder.push_values(result.chunks, |mut b, chunk| {
                b.push_bind(part_id)
                    .push_bind(PgRange::from(chunk.range.clone()))
                    .push_bind(chunk.md5.clone())
                    .push_bind(chunk.sha256.clone());
            });

            let insert_chunk = builder.build().execute(&mut *tx).await;

            if let Err(e) = insert_chunk {
                tracing::error!("Failed to insert chunk info: {:?}", e);
                return S3Error::InternalError.into_response();
            }

            Some(data_id)
        }
    };

    let result = sqlx::query!("SET CONSTRAINTS ALL DEFERRED")
        .execute(&mut *tx)
        .await;

    match result {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to set constraints deferred: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    }

    let file_id = sqlx::query!(
        r#"
        INSERT INTO
            files(bucket_id, key, current_version, current_version_is_delete_marker)
        VALUES ($1, $2, -1, FALSE)
        ON CONFLICT DO NOTHING RETURNING id
    "#,
        bucket_id,
        key
    )
    .fetch_optional(&mut *tx)
    .await;

    let file_id = match file_id {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("Failed to insert file: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let file_id = match file_id {
        Some(v) => v.id,
        None => {
            let file_id = sqlx::query!(
                "SELECT id FROM files WHERE bucket_id = $1 AND key = $2 LIMIT 1 FOR UPDATE",
                bucket_id,
                key
            )
            .fetch_one(&mut *tx)
            .await;

            match file_id {
                Ok(v) => v.id,
                Err(e) => {
                    tracing::error!("Failed to fetch file: {:?}", e);
                    return S3Error::InternalError.into_response();
                }
            }
        }
    };

    let file_version_id = sqlx::query!(
        "INSERT INTO file_versions(file_id, file_data_id) VALUES($1, $2) RETURNING id",
        file_id,
        file_data_id
    )
    .fetch_one(&mut *tx)
    .await;

    let file_version_id = match file_version_id {
        Ok(v) => v.id,
        Err(e) => {
            tracing::error!("Failed to insert file version: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let update_file_result = sqlx::query!("UPDATE files SET current_version = $1, current_version_is_delete_marker = FALSE WHERE id = $2", file_version_id, file_id)
        .execute(&mut *tx)
        .await;

    match update_file_result {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to update file: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    match tx.commit().await {
        Ok(_) => (),
        Err(e) => {
            tracing::error!("Failed to commit transaction: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    StatusCode::NO_CONTENT.into_response()
}
