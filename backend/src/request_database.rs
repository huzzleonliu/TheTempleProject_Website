use anyhow::Result;
use axum::{
    extract::Extension,
    http::StatusCode,
    response::{IntoResponse, Json},
};
use sqlx::postgres::{PgPool, PgPoolOptions, Row};

pub async fn get_all_table_names(pool: &PgPool) -> Result<Vec<String>> {
    // 使用系统表查询（推荐方式）
    let rows = sqlx::query(
        r#"
        SELECT tablename 
        FROM pg_tables 
        WHERE schemaname = 'public'
        "#,
    )
    .fetch_all(pool)
    .await?;

    // 提取表名到向量
    let table_names = rows
        .iter()
        .map(|row| row.get::<String, _>("tablename"))
        .collect();

    Ok(table_names)
}

// 使用示例（集成到 Axum 路由中）
pub async fn list_tables(pool: Extension<PgPool>) -> impl IntoResponse {
    match get_all_table_names(&pool).await {
        Ok(tables) => Json(json!({ "tables": tables })),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e.to_string() })),
        ),
    }
}
