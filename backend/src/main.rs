use axum::{routing::get, Router};
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};


#[tokio::main]
async fn main() {

    let app = Router::new()
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
        )
        .route("/print", get(print_code));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8081));
    println!("Server running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn print_code() -> impl IntoResponse {
    "Hello, world!"
}
