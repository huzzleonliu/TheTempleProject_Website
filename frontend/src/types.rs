use serde::{Deserialize, Serialize};

/// 目录节点数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
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


