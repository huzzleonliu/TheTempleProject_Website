use anyhow::Result;
use axum::{
    extract::{Extension, Path},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPool;
use sqlx::Row;

#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub path: String,
    pub has_layout: bool,
    pub has_visual_assets: bool,
    pub has_text: i32,
    pub has_images: i32,
    pub has_subnodes: bool,
}

/// 获取一级目录（路径深度为 1 的节点）
pub async fn get_root_directories(pool: &PgPool) -> Result<Vec<DirectoryNode>> {
    let rows = sqlx::query(
        r#"
        SELECT path, has_layout, has_visual_assets, has_text, has_images, has_subnodes
        FROM directories
        WHERE nlevel(path) = 1
        ORDER BY path;
        "#,
    )
    .fetch_all(pool)
    .await?;

    let directories = rows
        .iter()
        .map(|row| DirectoryNode {
            path: row.get::<String, _>("path"),
            has_layout: row.get::<bool, _>("has_layout"),
            has_visual_assets: row.get::<bool, _>("has_visual_assets"),
            has_text: row.get::<i32, _>("has_text"),
            has_images: row.get::<i32, _>("has_images"),
            has_subnodes: row.get::<bool, _>("has_subnodes"),
        })
        .collect();

    Ok(directories)
}

/// 获取指定路径的直接子目录
pub async fn get_child_directories(pool: &PgPool, parent_path: &str) -> Result<Vec<DirectoryNode>> {
    let rows = sqlx::query(
        r#"
        SELECT path, has_layout, has_visual_assets, has_text, has_images, has_subnodes
        FROM directories
        WHERE path ~ ($1::ltree || '.*{1}'::lquery)
        ORDER BY path;
        "#,
    )
    .bind(parent_path)
    .fetch_all(pool)
    .await?;

    let directories = rows
        .iter()
        .map(|row| DirectoryNode {
            path: row.get::<String, _>("path"),
            has_layout: row.get::<bool, _>("has_layout"),
            has_visual_assets: row.get::<bool, _>("has_visual_assets"),
            has_text: row.get::<i32, _>("has_text"),
            has_images: row.get::<i32, _>("has_images"),
            has_subnodes: row.get::<bool, _>("has_subnodes"),
        })
        .collect();

    Ok(directories)
}

/// API 处理函数：获取一级目录
pub async fn api_get_root_directories(pool: Extension<PgPool>) -> impl IntoResponse {
    match get_root_directories(&pool).await {
        Ok(directories) => Json(json!({ "directories": directories })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

/// API 处理函数：获取子目录
pub async fn api_get_child_directories(
    Path(parent_path): Path<String>,
    pool: Extension<PgPool>,
) -> impl IntoResponse {
    match get_child_directories(&pool, &parent_path).await {
        Ok(directories) => Json(json!({ "directories": directories })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}

