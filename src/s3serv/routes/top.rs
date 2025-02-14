use axum::{extract::State, response::Response};
use sqlx::PgPool;

use crate::s3serv::actions;

pub async fn get_top(State(pool): State<PgPool>) -> Response {
    actions::list_buckets(pool).await
}
