use axum::{
    extract::{Path, Query, State},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::s3serv::{actions, error::S3Error};

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum PutBucketQuery {
    PutBucketVersioning {
        versioning: String,
    },
    ObjectLock {
        #[serde(rename = "object-lock")]
        object_lock: String,
    },
    CreateBucket {},
}

async fn put_bucket_top(
    Path(bucket): Path<String>,
    State(pool): State<PgPool>,
    Query(query): Query<PutBucketQuery>,
) -> Response {
    match query {
        PutBucketQuery::PutBucketVersioning { versioning: _ } => {
            S3Error::NotImplemented.into_response()
        }
        PutBucketQuery::ObjectLock { object_lock: _ } => S3Error::NotImplemented.into_response(),
        PutBucketQuery::CreateBucket {} => actions::create_bucket(pool, bucket).await,
    }
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
pub enum GetBucketTopQuery {
    ListObjects { prefix: Option<String> },
}

async fn get_bucket_top(
    Path(bucket): Path<String>,
    State(pool): State<PgPool>,
    Query(query): Query<GetBucketTopQuery>,
) -> Response {
    match query {
        GetBucketTopQuery::ListObjects { prefix } => {
            actions::list_objects(pool, bucket, prefix.unwrap_or_default()).await
        }
    }
}

async fn delete_bucket_top(Path(bucket): Path<String>, State(pool): State<PgPool>) -> Response {
    actions::delete_bucket(pool, bucket).await
}

pub fn bucket_top() -> axum::routing::MethodRouter<PgPool> {
    axum::routing::put(put_bucket_top)
        .get(get_bucket_top)
        .delete(delete_bucket_top)
}
