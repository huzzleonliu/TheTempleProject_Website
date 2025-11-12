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

/// 统一的 Item 数据结构（包含数据和操作）
/// 
/// Item 是自包含的单元，包含数据和所有操作信号
/// 点击时直接调用 handle_click() 方法，触发内部信号更新
#[derive(Clone)]
pub struct Item {
    // 数据部分
    pub item_type: ItemType,
    pub path: String,
    pub has_subnodes: bool,
    pub raw_filename: String,
    
    // 操作信号（所有需要的 set 函数）
    pub set_overview_a_items: WriteSignal<Vec<Item>>,
    pub set_overview_b_items: WriteSignal<Vec<Item>>,
    pub set_overview_a_selected_path: WriteSignal<Option<String>>,
    pub set_preview_path: WriteSignal<Option<String>>,
    pub set_selected_path: WriteSignal<Option<String>>,
    pub set_selected_index: WriteSignal<Option<usize>>,
    
    // 上下文信息
    pub is_overview_a: bool,
    // 当前目录列表（用于某些操作，如进入子节点时需要知道当前列表）
    pub current_directories: ReadSignal<Vec<DirectoryNode>>,
}

impl Item {
    /// 从 DirectoryNode 创建 Item
    pub fn from_node(
        node: DirectoryNode,
        set_overview_a_items: WriteSignal<Vec<Item>>,
        set_overview_b_items: WriteSignal<Vec<Item>>,
        set_overview_a_selected_path: WriteSignal<Option<String>>,
        set_preview_path: WriteSignal<Option<String>>,
        set_selected_path: WriteSignal<Option<String>>,
        set_selected_index: WriteSignal<Option<usize>>,
        current_directories: ReadSignal<Vec<DirectoryNode>>,
        is_overview_a: bool,
    ) -> Self {
        Self {
            item_type: ItemType::Node,
            path: node.path,
            has_subnodes: node.has_subnodes,
            raw_filename: node.raw_filename,
            set_overview_a_items,
            set_overview_b_items,
            set_overview_a_selected_path,
            set_preview_path,
            set_selected_path,
            set_selected_index,
            is_overview_a,
            current_directories,
        }
    }
    
    /// 处理点击事件
    pub fn handle_click(&self) {
        match self.item_type {
            ItemType::Node => {
                if self.is_overview_a {
                    self.handle_overview_a_click();
                } else {
                    self.handle_overview_b_click();
                }
            }
        }
    }
    
    /// OverviewA 中的点击处理
    fn handle_overview_a_click(&self) {
        use leptos::task::spawn_local;
        use crate::api::{get_child_directories, get_root_directories};
        use web_sys::console;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        
        let path = self.path.clone();
        console::log_2(&"[OverviewA 点击] 路径:".into(), &path.clone().into());
        
        // 获取父路径（上一级）
        let parent_path = if path.contains('.') {
            let parts: Vec<&str> = path.split('.').collect();
            if parts.len() > 1 {
                Some(parts[0..parts.len()-1].join("."))
            } else {
                None
            }
        } else {
            None
        };
        
        // 设置选中的路径
        self.set_selected_path.set(Some(path.clone()));
        
        // 获取被点击节点的信息，检查是否有子节点
        let path_for_info = path.clone();
        let set_preview_path = self.set_preview_path.clone();
        spawn_local(async move {
            match get_child_directories(&path_for_info).await {
                Ok(children) => {
                    if !children.is_empty() {
                        set_preview_path.set(Some(path_for_info.clone()));
                    } else {
                        set_preview_path.set(None);
                    }
                }
                Err(_) => {
                    set_preview_path.set(None);
                }
            }
        });
        
        // 加载兄弟节点到 OverviewB
        let parent_for_b = parent_path.clone();
        let clicked_path = path.clone();
        let set_overview_b_items = self.set_overview_b_items.clone();
        let set_overview_a_items = self.set_overview_a_items.clone();
        let set_overview_a_selected_path = self.set_overview_a_selected_path.clone();
        let set_preview_path = self.set_preview_path.clone();
        let set_selected_path = self.set_selected_path.clone();
        let set_selected_index = self.set_selected_index.clone();
        let current_directories = self.current_directories.clone();
        
        spawn_local(async move {
            let result = if let Some(p) = parent_for_b {
                get_child_directories(&p).await
            } else {
                get_root_directories().await
            };
            
            if let Ok(data_dirs) = result {
                // 创建新的 Item 列表
                let new_items: Vec<Item> = data_dirs
                    .iter()
                    .map(|node| {
                        Item::from_node(
                            node.clone(),
                            set_overview_a_items.clone(),
                            set_overview_b_items.clone(),
                            set_overview_a_selected_path.clone(),
                            set_preview_path.clone(),
                            set_selected_path.clone(),
                            set_selected_index.clone(),
                            current_directories.clone(),
                            false, // OverviewB
                        )
                    })
                    .collect();
                set_overview_b_items.set(new_items.clone());
                
                // 找到被点击的节点在兄弟节点列表中的索引
                if let Some(index) = data_dirs.iter().position(|d| d.path == clicked_path) {
                    console::log_2(&"[OverviewA] 找到被点击节点在兄弟节点列表中的索引:".into(), &index.into());
                    
                    // 使用 request_animation_frame 延迟设置索引，确保 items 已更新
                    if let Some(window) = web_sys::window() {
                        let closure1 = Closure::once_into_js(move || {
                            if let Some(window2) = web_sys::window() {
                                let closure2 = Closure::once_into_js(move || {
                                    set_selected_index.set(Some(index));
                                    
                                    // 立即设置 Preview
                                    if let Some(dir) = data_dirs.get(index) {
                                        if dir.has_subnodes {
                                            set_preview_path.set(Some(dir.path.clone()));
                                        } else {
                                            set_preview_path.set(None);
                                        }
                                    }
                                });
                                let _ = window2.request_animation_frame(closure2.as_ref().unchecked_ref());
                            }
                        });
                        let _ = window.request_animation_frame(closure1.as_ref().unchecked_ref());
                    } else {
                        set_selected_index.set(Some(index));
                        if let Some(dir) = data_dirs.get(index) {
                            if dir.has_subnodes {
                                set_preview_path.set(Some(dir.path.clone()));
                            } else {
                                set_preview_path.set(None);
                            }
                        }
                    }
                }
            }
        });
        
        // 加载父节点到 OverviewA
        if let Some(pp) = parent_path {
            let parent_for_a = if pp.contains('.') {
                let parts: Vec<&str> = pp.split('.').collect();
                if parts.len() > 1 {
                    Some(parts[0..parts.len()-1].join("."))
                } else {
                    None
                }
            } else {
                None
            };
            
            let set_overview_a_items = self.set_overview_a_items.clone();
            let set_overview_a_selected_path = self.set_overview_a_selected_path.clone();
            let set_preview_path = self.set_preview_path.clone();
            let set_selected_path = self.set_selected_path.clone();
            let set_selected_index = self.set_selected_index.clone();
            let set_overview_b_items = self.set_overview_b_items.clone();
            let current_directories = self.current_directories.clone();
            
            spawn_local(async move {
                let result = if let Some(p) = parent_for_a {
                    get_child_directories(&p).await
                } else {
                    get_root_directories().await
                };
                if let Ok(data_dirs) = result {
                    // 创建新的 Item 列表
                    // 注意：current_directories 应该使用新的父节点列表
                    let (data_dirs_signal, _) = signal::<Vec<DirectoryNode>>(data_dirs.clone());
                    let data_dirs_read = data_dirs_signal;
                    let new_items: Vec<Item> = data_dirs
                        .iter()
                        .map(|node| {
                            Item::from_node(
                                node.clone(),
                                set_overview_a_items.clone(),
                                set_overview_b_items.clone(),
                                set_overview_a_selected_path.clone(),
                                set_preview_path.clone(),
                                set_selected_path.clone(),
                                set_selected_index.clone(),
                                data_dirs_read.clone(),
                                true, // OverviewA
                            )
                        })
                        .collect();
                    set_overview_a_items.set(new_items);
                }
            });
        } else {
            // 如果父路径是根，OverviewA 应该显示空列表（只有 "/"）
            self.set_overview_a_items.set(Vec::new());
        }
    }
    
    /// OverviewB 中的点击处理
    fn handle_overview_b_click(&self) {
        use leptos::task::spawn_local;
        use crate::api::get_child_directories;
        use web_sys::console;
        use wasm_bindgen::prelude::*;
        use wasm_bindgen::JsCast;
        
        let path = self.path.clone();
        console::log_2(&"[鼠标点击] 路径:".into(), &path.clone().into());
        console::log_2(&"[鼠标点击] 有子节点:".into(), &self.has_subnodes.into());
        
        // 设置选中的路径（用于高亮显示）
        self.set_selected_path.set(Some(path.clone()));
        
        // 只有当节点有子节点时才执行跳转
        if self.has_subnodes {
            console::log_1(&"[鼠标点击] 进入子节点".into());
            
            // 将当前 OverviewB 的内容移到 OverviewA（作为父级节点）
            let current_dirs = self.current_directories.get();
            let set_overview_a_items = self.set_overview_a_items.clone();
            let set_overview_b_items = self.set_overview_b_items.clone();
            let set_overview_a_selected_path = self.set_overview_a_selected_path.clone();
            let set_preview_path = self.set_preview_path.clone();
            let set_selected_path = self.set_selected_path.clone();
            let set_selected_index = self.set_selected_index.clone();
            
            // 为 OverviewA 创建新的 current_directories 信号
            let (current_dirs_signal, _) = signal::<Vec<DirectoryNode>>(current_dirs.clone());
            let current_dirs_read = current_dirs_signal;
            
            let current_items: Vec<Item> = current_dirs
                .iter()
                .map(|node| {
                    Item::from_node(
                        node.clone(),
                        set_overview_a_items.clone(),
                        set_overview_b_items.clone(),
                        set_overview_a_selected_path.clone(),
                        set_preview_path.clone(),
                        set_selected_path.clone(),
                        set_selected_index.clone(),
                        current_dirs_read.clone(),
                        true, // 移到 OverviewA
                    )
                })
                .collect();
            self.set_overview_a_items.set(current_items);
            
            // 高亮 OverviewA 中的当前节点（作为父级）
            self.set_overview_a_selected_path.set(Some(path.clone()));
            
            // 先清空 Preview 和 selected_index，等待子节点加载完成后再根据 selected_index 更新
            self.set_preview_path.set(None);
            self.set_selected_index.set(None);
            
            // 加载子节点
            let path_clone = path.clone();
            let set_overview_b_items = self.set_overview_b_items.clone();
            let set_preview_path = self.set_preview_path.clone();
            let set_selected_path = self.set_selected_path.clone();
            let set_selected_index = self.set_selected_index.clone();
            let set_overview_a_selected_path = self.set_overview_a_selected_path.clone();
            let set_overview_a_items = self.set_overview_a_items.clone();
            let current_directories = self.current_directories.clone();
            
            spawn_local(async move {
                console::log_2(&"[鼠标点击] 请求子节点:".into(), &path_clone.clone().into());
                match get_child_directories(&path_clone).await {
                    Ok(children) => {
                        console::log_2(&"[鼠标点击] 加载子节点成功，数量:".into(), &children.len().into());
                        
                        // 创建新的 Item 列表
                        // 注意：current_directories 应该使用新的子节点列表
                        let (children_signal, _) = signal::<Vec<DirectoryNode>>(children.clone());
                        let children_read = children_signal;
                        let new_items: Vec<Item> = children
                            .iter()
                            .map(|node| {
                                Item::from_node(
                                    node.clone(),
                                    set_overview_a_items.clone(),
                                    set_overview_b_items.clone(),
                                    set_overview_a_selected_path.clone(),
                                    set_preview_path.clone(),
                                    set_selected_path.clone(),
                                    set_selected_index.clone(),
                                    children_read.clone(),
                                    false, // OverviewB
                                )
                            })
                            .collect();
                        set_overview_b_items.set(new_items);
                        
                        // 等待 items 更新后再设置 selected_index
                        let children_for_closure = children.clone();
                        if let Some(window) = web_sys::window() {
                            let closure1 = Closure::once_into_js(move || {
                                if let Some(window2) = web_sys::window() {
                                    let closure2 = Closure::once_into_js(move || {
                                        set_selected_index.set(Some(0));
                                        
                                        // 设置第一个节点的 Preview
                                        if let Some(first_dir) = children_for_closure.first() {
                                            if first_dir.has_subnodes {
                                                set_preview_path.set(Some(first_dir.path.clone()));
                                            }
                                        }
                                    });
                                    let _ = window2.request_animation_frame(closure2.as_ref().unchecked_ref());
                                }
                            });
                            let _ = window.request_animation_frame(closure1.as_ref().unchecked_ref());
                        } else {
                            set_selected_index.set(Some(0));
                            if let Some(first_dir) = children.first() {
                                if first_dir.has_subnodes {
                                    set_preview_path.set(Some(first_dir.path.clone()));
                                }
                            }
                        }
                    }
                    Err(e) => {
                        console::log_2(&"[鼠标点击] 请求失败:".into(), &format!("{e}").into());
                    }
                }
            });
        }
    }
}

// 注意：NavigationSignals 和 DataSignals 已被移除
// Item 现在直接包含所有需要的信号

/// API 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
pub struct DirectoriesResponse {
    pub directories: Vec<DirectoryNode>,
}


