use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 根路径常量
pub const ROOT_PATH: &str = "";

/// 目录节点数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DirectoryNode {
    pub path: String,
    pub has_subnodes: bool,
    pub raw_filename: String,
}

/// 资源文件数据结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AssetNode {
    pub file_path: String,
    pub raw_path: String,
    pub raw_filename: String,
}

/// UI 列表节点类型
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum NodeKind {
    Overview,
    Directory,
    Video,
    Image,
    Pdf,
    Markdown,
    Other,
}

/// UI 列表节点
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct UiNode {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub directory_path: Option<String>,
    pub raw_path: Option<String>,
    pub has_children: bool,
}

/// 预览区域需要的节点/资源描述
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PreviewItem {
    pub id: String,
    pub label: String,
    pub kind: NodeKind,
    pub directory_path: Option<String>,
    pub raw_path: Option<String>,
    pub has_children: bool,
    pub content: Option<String>,
    pub display_as_entry: bool,
}

/// 目录列表 API 响应体
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoriesResponse {
    pub directories: Vec<DirectoryNode>,
}

/// 资源文件 API 响应体
#[derive(Debug, Serialize, Deserialize)]
pub struct AssetsResponse {
    pub assets: Vec<AssetNode>,
}

/// 缓存结构：key 为父路径，value 为该路径下的直接子节点列表
pub type NodesCache = HashMap<String, Vec<DirectoryNode>>;

/// 资源缓存结构：key 为父路径，value 为该路径下的文件列表
pub type AssetsCache = HashMap<String, Vec<AssetNode>>;

/// 按层级拆分路径，例如 "a.b.c" -> ["a", "a.b", "a.b.c"]
pub fn split_levels(path: &str) -> Vec<String> {
    if path.is_empty() {
        return Vec::new();
    }
    let mut levels = Vec::new();
    let mut current = Vec::new();
    for segment in path.split('.') {
        current.push(segment);
        levels.push(current.join("."));
    }
    levels
}

/// 获取父路径
pub fn parent_path(path: &str) -> Option<String> {
    if path.is_empty() {
        return None;
    }
    let mut parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        None
    } else {
        parts.pop();
        if parts.is_empty() {
            Some(String::new())
        } else {
            Some(parts.join("."))
        }
    }
}
