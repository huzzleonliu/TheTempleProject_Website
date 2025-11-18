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
    pub has_subnodes: bool,
    pub raw_filename: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssetNode {
    pub file_path: String,
    pub raw_path: String,
    pub raw_filename: String,
}

/// 获取一级目录（路径深度为 1 的节点）
pub async fn get_root_directories(pool: &PgPool) -> Result<Vec<DirectoryNode>> {
    let rows = sqlx::query(
        r#"
        SELECT path::text as path, has_subnodes, raw_filename
        FROM directory_nodes
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
            has_subnodes: row.get::<bool, _>("has_subnodes"),
            raw_filename: row.get::<String, _>("raw_filename"),
        })
        .collect();

    Ok(directories)
}

/// 获取指定路径的直接子目录
pub async fn get_child_directories(pool: &PgPool, parent_path: &str) -> Result<Vec<DirectoryNode>> {
    // 构建 lquery 模式：parent_path.*{1} 表示匹配 parent_path 的直接子节点（深度为 1）
    let lquery_pattern = format!("{}.*{{1}}", parent_path);

    let rows = sqlx::query(
        r#"
        SELECT path::text as path, has_subnodes, raw_filename
        FROM directory_nodes
        WHERE path ~ $1::lquery
        ORDER BY path;
        "#,
    )
    .bind(&lquery_pattern)
    .fetch_all(pool)
    .await?;

    let directories = rows
        .iter()
        .map(|row| DirectoryNode {
            path: row.get::<String, _>("path"),
            has_subnodes: row.get::<bool, _>("has_subnodes"),
            raw_filename: row.get::<String, _>("raw_filename"),
        })
        .collect();

    Ok(directories)
}

/// 获取指定目录下的资源文件（目前默认读取 visual_assets 中的直接文件）
pub async fn get_node_assets(pool: &PgPool, parent_path: &str) -> Result<Vec<AssetNode>> {
    if parent_path.is_empty() {
        return Ok(Vec::new());
    }

    // file_nodes.file_path 的父路径形如 "<node>.visual_assets"
    let assets_parent = format!("{}.visual_assets", parent_path);

    let rows = sqlx::query(
        r#"
        SELECT file_path::text as file_path, raw_path, raw_filename
        FROM file_nodes
        WHERE subpath(file_path, 0, nlevel(file_path) - 1) = $1::ltree
        ORDER BY file_path;
        "#,
    )
    .bind(&assets_parent)
    .fetch_all(pool)
    .await?;

    let assets = rows
        .iter()
        .map(|row| AssetNode {
            file_path: row.get::<String, _>("file_path"),
            raw_path: row.get::<String, _>("raw_path"),
            raw_filename: row.get::<String, _>("raw_filename"),
        })
        .collect();

    Ok(assets)
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

/// API 处理函数：获取节点资源文件
pub async fn api_get_node_assets(
    Path(parent_path): Path<String>,
    pool: Extension<PgPool>,
) -> impl IntoResponse {
    match get_node_assets(&pool, &parent_path).await {
        Ok(assets) => Json(json!({ "assets": assets })),
        Err(e) => Json(json!({ "error": e.to_string() })),
    }
}
