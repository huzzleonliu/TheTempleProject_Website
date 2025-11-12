use serde::{Deserialize, Serialize};
use leptos::prelude::*;

/// 目录节点数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectoryNode {
    pub path: String,
    pub has_subnodes: bool,
    pub raw_filename: String,
}

/// Item 类型枚举
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemType {
    /// 目录节点
    Node,
    // 未来可以添加：
    // File,   // 文件
    // Image,  // 图片
    // Text,   // 文本
}

/// 统一的 Item 数据结构
#[derive(Debug, Clone)]
pub struct Item {
    pub item_type: ItemType,
    pub path: String,
    pub has_subnodes: bool,
    pub raw_filename: String,
}

impl Item {
    /// 从 DirectoryNode 创建 Item
    pub fn from_node(node: DirectoryNode) -> Self {
        Self {
            item_type: ItemType::Node,
            path: node.path,
            has_subnodes: node.has_subnodes,
            raw_filename: node.raw_filename,
        }
    }
    
    /// 从路径字符串创建 Item（用于 OverviewA，需要从路径提取显示名称）
    /// 注意：这个方法创建的 Item 没有完整的 DirectoryNode 信息，has_subnodes 默认为 true
    /// 实际使用时需要从 API 获取完整信息
    pub fn from_path(path: String) -> Self {
        // 从路径提取显示名称（最后一个点号后的部分）
        let raw_filename = path.split('.').last().unwrap_or(&path).to_string();
        Self {
            item_type: ItemType::Node,
            path,
            has_subnodes: true, // 默认值，实际使用时需要从 API 获取
            raw_filename,
        }
    }
}

/// 导航相关的信号
#[derive(Clone)]
pub struct NavigationSignals {
    pub set_overview_a_directories: WriteSignal<Vec<String>>,
    pub set_overview_a_selected_path: WriteSignal<Option<String>>,
    pub set_overview_b_directories: WriteSignal<Vec<String>>,
    pub set_preview_path: WriteSignal<Option<String>>,
    pub set_selected_path: WriteSignal<Option<String>>,
    pub set_selected_index: WriteSignal<Option<usize>>,
}

/// 数据相关的信号
#[derive(Clone)]
pub struct DataSignals {
    pub directories: ReadSignal<Vec<DirectoryNode>>,
}

/// Item 上下文：包含 Item 数据和所有相关的信号
/// 
/// 这个结构将 Item 和所有相关的信号打包在一起，使得代码更清晰、易于管理
#[derive(Clone)]
pub struct ItemContext {
    pub item: Item,
    /// 导航信号
    pub nav: NavigationSignals,
    /// 数据信号（根据 item_type 决定是否需要）
    pub data: DataSignals,
    /// 上下文信息：是否在 OverviewA 中使用
    pub is_overview_a: bool,
}

impl ItemContext {
    /// 从 DirectoryNode 创建 ItemContext（用于 OverviewB）
    pub fn from_node(
        node: DirectoryNode,
        nav: NavigationSignals,
        data: DataSignals,
        is_overview_a: bool,
    ) -> Self {
        Self {
            item: Item::from_node(node),
            nav,
            data,
            is_overview_a,
        }
    }
    
    /// 从路径创建 ItemContext（用于 OverviewA）
    pub fn from_path(
        path: String,
        nav: NavigationSignals,
        data: DataSignals,
    ) -> Self {
        Self {
            item: Item::from_path(path),
            nav,
            data,
            is_overview_a: true,
        }
    }
}

/// API 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoriesResponse {
    pub directories: Vec<DirectoryNode>,
}


