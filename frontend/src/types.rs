use serde::{Deserialize, Serialize};

/// 目录节点数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub path: String,
    pub has_layout: bool,
    pub has_visual_assets: bool,
    pub has_text: i32,
    pub has_images: i32,
    pub has_subnodes: bool,
    pub raw_filename: String,
}

/// API 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoriesResponse {
    pub directories: Vec<DirectoryNode>,
}


