use anyhow::Result;
use axum::{
    extract::Extension,
    response::{IntoResponse, Json},
};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::{Column, Row};

pub async fn get_all_table_names(pool: &PgPool) -> Result<Vec<String>> {
    // 使用系统表查询（推荐方式）
    let rows = sqlx::query(
        r#"
        SELECT * FROM eventexperience LIMIT 1 OFFSET 1;
        "#,
    )
    .fetch_all(pool)
    .await?;

    // 获取所有列名
    let columns = rows[0].columns();
    let mut result = Vec::new();

    // 遍历所有列并获取数据
    for column in columns {
        let column_name = column.name();
        let value: String = rows[0].get(column_name);
        result.push(format!("{}: {}", column_name, value));
    }

    Ok(result)
}

// 使用示例（集成到 Axum 路由中）
pub async fn get_line_one(pool: Extension<PgPool>) -> impl IntoResponse {
    match get_all_table_names(&pool).await {
        Ok(tables) => Json(json!({ "data": tables })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
