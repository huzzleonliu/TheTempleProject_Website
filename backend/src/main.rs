use axum::{routing::get, Router};
use axum::response::IntoResponse;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use http::header::{AUTHORIZATION, ACCEPT};
use http::Method;


#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/print", get(print_code))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([AUTHORIZATION, ACCEPT])
                // .allow_credentials(true)
        );

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
