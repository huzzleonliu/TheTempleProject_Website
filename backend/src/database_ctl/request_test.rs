use anyhow::Result;
use axum::{
    extract::Extension,
    response::{IntoResponse, Json},
};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Row;

pub async fn get_all_table_names(pool: &PgPool) -> Result<Vec<String>> {
    // 使用系统表查询（推荐方式）
    let rows = sqlx::query(
        r#"
        SELECT schemaname, tablename 
        FROM pg_catalog.pg_tables 
        WHERE tablename IN (
          'education_experience', 
          'eventexperience',
          'exhibitionexperience',
          'foundingexperience',
          'internexperience'
        );
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
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
