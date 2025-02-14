use axum::routing::get;
use sqlx::PgPool;

mod actions;
pub mod error;
mod routes;

pub async fn start_serv(pool: PgPool) {
    let app = axum::Router::new()
        .route("/", get(routes::get_top))
        .route("/{bucket}", routes::bucket_top())
        .route("/{bucket}/", routes::bucket_top())
        .route("/{bucket}/{*key}", routes::bucket_object());

    let app = app
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                |r: &axum::http::Request<_>| {
                    let request_id = r
                        .headers()
                        .get("x-request-id")
                        .and_then(|x| x.to_str().ok())
                        .map(|x| x.to_string())
                        .unwrap_or_default();
                    tracing::info_span!("request", request_id = %request_id)
                },
            ),
        )
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on 3000 port");
    axum::serve(listener, app).await.unwrap();
}
