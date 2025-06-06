use axum::response::IntoResponse;

pub async fn print_code() -> impl IntoResponse {
    "Hello, world!"
}
