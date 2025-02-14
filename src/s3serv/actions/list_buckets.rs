use axum::response::{IntoResponse, Response};
use chrono::SecondsFormat;
use serde::Serialize;
use sqlx::PgPool;

use crate::s3serv::error::S3Error;

#[derive(serde::Serialize)]
struct ListAllMyBucketsResult {
    #[serde(rename = "Buckets")]
    buckets: Buckets,
    #[serde(rename = "Owner")]
    owner: Owner,
}

#[derive(serde::Serialize)]
struct Buckets {
    #[serde(rename = "Bucket")]
    buckets: Vec<Bucket>,
}
#[derive(serde::Serialize)]
struct Bucket {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "CreationDate")]
    creation_date: String,
}

#[derive(serde::Serialize)]
struct Owner {}

#[tracing::instrument]
pub async fn list_buckets(pool: PgPool) -> Response {
    let buckets = sqlx::query!("SELECT name, created_at FROM buckets")
        .fetch_all(&pool)
        .await;

    let buckets = match buckets {
        Ok(v) => v,
        Err(e) => {
            tracing::error!("failed to fetch buckets: {:?}", e);
            return S3Error::InternalError.into_response();
        }
    };

    let buckets = ListAllMyBucketsResult {
        buckets: Buckets {
            buckets: buckets
                .into_iter()
                .map(|bucket| Bucket {
                    name: bucket.name,
                    creation_date: bucket.created_at.to_rfc3339_opts(SecondsFormat::Secs, true),
                })
                .collect(),
        },
        owner: Owner {},
    };

    let mut buffer = String::new();
    let serializer = quick_xml::se::Serializer::new(&mut buffer);
    buckets.serialize(serializer).expect("Failed to serialize.");

    ([("Content-Type", "application/xml")], buffer).into_response()
}
