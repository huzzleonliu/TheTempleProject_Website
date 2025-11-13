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

/// API 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoriesResponse {
    pub directories: Vec<DirectoryNode>,
}

/// 缓存结构：key 为父路径，value 为该路径下的直接子节点列表
pub type NodesCache = HashMap<String, Vec<DirectoryNode>>;

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
