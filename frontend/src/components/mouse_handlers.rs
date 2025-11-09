use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
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
}

/// API 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
struct DirectoriesResponse {
    directories: Vec<DirectoryNode>,
}

/// 处理鼠标点击节点时的导航逻辑
/// 
/// # 参数
/// - `path`: 被点击的节点路径
/// - `has_subnodes`: 节点是否有子节点
/// - `directories`: 当前目录列表（用于获取完整路径列表）
/// - `set_overview_a_directories`: 设置 OverviewA 的目录列表
/// - `set_overview_b_directories`: 设置 OverviewB 的目录列表
/// - `set_preview_path`: 设置 Preview 显示的路径
/// - `set_selected_path`: 设置当前选中的路径
/// - `set_selected_index`: 设置当前选中的索引
pub fn handle_node_click(
    path: String,
    has_subnodes: bool,
    directories: Vec<DirectoryNode>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) {
    // 设置选中的路径和索引
    let dirs = directories.clone();
    if let Some(idx) = dirs.iter().position(|d| d.path == path) {
        set_selected_index.set(Some(idx));
    }
    set_selected_path.set(Some(path.clone()));
    
    // 只有当节点有子节点时才执行跳转
    if has_subnodes {
        // 将当前 OverviewB 的内容移到 OverviewA
        let current_dirs: Vec<String> = directories
            .iter()
            .map(|d| d.path.clone())
            .collect();
        set_overview_a_directories.set(current_dirs);
        
        // 设置 Preview 显示被点击节点的子节点
        set_preview_path.set(Some(path.clone()));
        
        // 加载被点击节点的子目录到 OverviewB
        let path_clone = path.clone();
        spawn_local(async move {
            let encoded_path = urlencoding::encode(&path_clone);
            let url = format!("/api/directories/children/{}", encoded_path);
            
            match Request::get(&url).send().await {
                Ok(resp) => {
                    match resp.json::<DirectoriesResponse>().await {
                        Ok(data) => {
                            let dir_paths: Vec<String> = data.directories.iter()
                                .map(|d| d.path.clone())
                                .collect();
                            set_overview_b_directories.set(dir_paths);
                            // 重置选中索引（会在effect中根据OverviewA的最后一个节点定位）
                            set_selected_index.set(None);
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
        });
    } else {
        // 如果没有子节点，不设置 Preview，也不跳转
        set_preview_path.set(None);
    }
}

