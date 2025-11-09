use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::ev::KeyboardEvent;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

use crate::components::mouse_handlers::DirectoryNode;

/// API 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
struct DirectoriesResponse {
    directories: Vec<DirectoryNode>,
}

/// 处理键盘导航事件
/// 
/// # 参数
/// - `event`: 键盘事件
/// - `directories`: 当前目录列表
/// - `selected_index`: 当前选中的索引
/// - `overview_a_directories`: OverviewA 的目录列表
/// - `set_selected_index`: 设置选中索引的函数
/// - `set_selected_path`: 设置选中路径的函数
/// - `set_overview_a_directories`: 设置 OverviewA 目录列表的函数
/// - `set_overview_b_directories`: 设置 OverviewB 目录列表的函数
/// - `set_preview_path`: 设置 Preview 路径的函数
/// - `set_directories`: 设置 directories 的函数（用于返回时定位）
/// - `preview_scroll_ref`: Preview 滚动容器的引用
pub fn handle_keyboard_navigation(
    event: KeyboardEvent,
    directories: Vec<DirectoryNode>,
    selected_index: Option<usize>,
    overview_a_directories: Vec<String>,
    set_selected_index: WriteSignal<Option<usize>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_directories: WriteSignal<Vec<DirectoryNode>>,
    preview_scroll_ref: NodeRef<leptos::html::Div>,
) {
    let key = event.key();
    let shift_pressed = event.shift_key();
    
    // 处理 Shift+J 和 Shift+K 滚动 Preview
    if shift_pressed {
        handle_preview_scroll(&key, &event, &preview_scroll_ref);
        return;
    }
    
    // 只处理 j/k/l/h 键（不带 Shift）
    match key.as_str() {
        "j" | "k" | "l" | "h" => {
            event.prevent_default();
            event.stop_propagation();
        }
        _ => {
            return;
        }
    }
    
    if directories.is_empty() {
        return;
    }
    
    let current_index = selected_index.unwrap_or(0);
    let max_index = directories.len() - 1;
    
    match key.as_str() {
        "j" => {
            // 向下移动
            handle_move_down(current_index, max_index, set_selected_index);
        }
        "k" => {
            // 向上移动
            handle_move_up(current_index, set_selected_index);
        }
        "l" => {
            // 进入当前选中的节点（相当于点击）
            handle_enter_node(
                current_index,
                &directories,
                set_selected_path,
                set_overview_a_directories,
                set_preview_path,
                set_overview_b_directories,
                set_selected_index,
            );
        }
        "h" => {
            // 回到上层节点（相当于点击 OverviewA 中的最后一个节点或 "/"）
            handle_go_back(
                &overview_a_directories,
                set_selected_path,
                set_preview_path,
                set_overview_b_directories,
                set_overview_a_directories,
                set_selected_index,
                set_directories,
            );
        }
        _ => {}
    }
}

/// 处理 Preview 滚动（Shift+J/K）
fn handle_preview_scroll(
    key: &str,
    event: &KeyboardEvent,
    preview_scroll_ref: &NodeRef<leptos::html::Div>,
) {
    match key {
        "J" | "j" => {
            // Shift+J: 向下滚动 Preview
            event.prevent_default();
            event.stop_propagation();
            
            if let Some(container) = preview_scroll_ref.get() {
                let scroll_amount = 100.0; // 每次滚动 100px
                let current_scroll = container.scroll_top() as f64;
                let max_scroll = (container.scroll_height() - container.client_height()) as f64;
                let new_scroll = (current_scroll + scroll_amount).min(max_scroll);
                container.set_scroll_top(new_scroll as i32);
            }
        }
        "K" | "k" => {
            // Shift+K: 向上滚动 Preview
            event.prevent_default();
            event.stop_propagation();
            
            if let Some(container) = preview_scroll_ref.get() {
                let scroll_amount = 100.0; // 每次滚动 100px
                let current_scroll = container.scroll_top() as f64;
                let new_scroll = (current_scroll - scroll_amount).max(0.0);
                container.set_scroll_top(new_scroll as i32);
            }
        }
        _ => {}
    }
}

/// 处理向下移动（j 键）
fn handle_move_down(
    current_index: usize,
    max_index: usize,
    set_selected_index: WriteSignal<Option<usize>>,
) {
    let new_index = if current_index < max_index {
        current_index + 1
    } else {
        max_index
    };
    set_selected_index.set(Some(new_index));
}

/// 处理向上移动（k 键）
fn handle_move_up(
    current_index: usize,
    set_selected_index: WriteSignal<Option<usize>>,
) {
    let new_index = if current_index > 0 {
        current_index - 1
    } else {
        0
    };
    set_selected_index.set(Some(new_index));
}

/// 处理进入节点（l 键）
fn handle_enter_node(
    current_index: usize,
    directories: &[DirectoryNode],
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) {
    if let Some(dir) = directories.get(current_index) {
        if dir.has_subnodes {
            // 设置选中的路径
            set_selected_path.set(Some(dir.path.clone()));
            
            // 将当前 OverviewB 的内容移到 OverviewA
            let current_dirs: Vec<String> = directories.iter()
                .map(|d| d.path.clone())
                .collect();
            set_overview_a_directories.set(current_dirs);
            
            // 设置 Preview 显示被点击节点的子节点
            set_preview_path.set(Some(dir.path.clone()));
            
            // 加载被点击节点的子目录到 OverviewB
            let path_clone = dir.path.clone();
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
        }
    }
}

/// 处理返回上层节点（h 键）
fn handle_go_back(
    overview_a_directories: &[String],
    set_selected_path: WriteSignal<Option<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
    set_directories: WriteSignal<Vec<DirectoryNode>>,
) {
    if !overview_a_directories.is_empty() {
        // 点击 OverviewA 中的最后一个节点
        if let Some(last_path) = overview_a_directories.last() {
            handle_navigate_to_parent(
                last_path.clone(),
                set_selected_path,
                set_preview_path,
                set_overview_b_directories,
                set_overview_a_directories,
                set_selected_index,
                set_directories,
            );
        }
    } else {
        // 如果 OverviewA 为空，点击 "/"
        handle_navigate_to_root(
            set_overview_b_directories,
            set_preview_path,
            set_overview_a_directories,
            set_selected_path,
            set_selected_index,
        );
    }
}

/// 导航到父级节点
fn handle_navigate_to_parent(
    path: String,
    set_selected_path: WriteSignal<Option<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
    set_directories: WriteSignal<Vec<DirectoryNode>>,
) {
    let path_clone = path.clone();
    // 保存这个路径，用于定位
    let path_to_select = path_clone.clone();
    
    // 获取父路径（上一级）
    let parent_path = if path_clone.contains('.') {
        let parts: Vec<&str> = path_clone.split('.').collect();
        if parts.len() > 1 {
            Some(parts[0..parts.len()-1].join("."))
        } else {
            None
        }
    } else {
        None
    };
    
    // 设置选中的路径
    set_selected_path.set(Some(path_clone.clone()));
    
    // 先获取被点击节点的信息，检查是否有子节点
    let path_for_info = path_clone.clone();
    spawn_local(async move {
        let encoded_path = urlencoding::encode(&path_for_info);
        let url = format!("/api/directories/children/{}", encoded_path);
        
        match Request::get(&url).send().await {
            Ok(resp) => {
                match resp.json::<DirectoriesResponse>().await {
                    Ok(data) => {
                        // 只有当节点有子节点时才设置 Preview
                        if !data.directories.is_empty() {
                            set_preview_path.set(Some(path_for_info.clone()));
                        } else {
                            set_preview_path.set(None);
                        }
                    }
                    Err(_) => {
                        set_preview_path.set(None);
                    }
                }
            }
            Err(_) => {
                set_preview_path.set(None);
            }
        }
    });
    
    // 加载兄弟节点到 OverviewB
    let parent_for_b = parent_path.clone();
    let path_to_select_clone = path_to_select.clone();
    spawn_local(async move {
        let url = if let Some(p) = parent_for_b {
            let encoded_path = urlencoding::encode(&p);
            format!("/api/directories/children/{}", encoded_path)
        } else {
            "/api/directories/root".to_string()
        };
        
        match Request::get(&url).send().await {
            Ok(resp) => {
                match resp.json::<DirectoriesResponse>().await {
                    Ok(data) => {
                        let dir_paths: Vec<String> = data.directories.iter()
                            .map(|d| d.path.clone())
                            .collect();
                        set_overview_b_directories.set(dir_paths);
                        
                        // 直接设置 directories，然后定位到之前选中的节点
                        set_directories.set(data.directories.clone());
                        
                        // 在 directories 设置后，定位到之前选中的节点
                        if let Some(index) = data.directories.iter().position(|d| d.path == path_to_select_clone) {
                            set_selected_index.set(Some(index));
                            set_selected_path.set(Some(path_to_select_clone));
                        } else {
                            set_selected_index.set(Some(0));
                        }
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    });
    
    // 加载上一级节点到 OverviewA
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
        
        spawn_local(async move {
            let url = if let Some(p) = parent_for_a {
                let encoded_path = urlencoding::encode(&p);
                format!("/api/directories/children/{}", encoded_path)
            } else {
                "/api/directories/root".to_string()
            };
            
            match Request::get(&url).send().await {
                Ok(resp) => {
                    match resp.json::<DirectoriesResponse>().await {
                        Ok(data) => {
                            let dir_paths: Vec<String> = data.directories.iter()
                                .map(|d| d.path.clone())
                                .collect();
                            set_overview_a_directories.set(dir_paths);
                        }
                        Err(_) => {}
                    }
                }
                Err(_) => {}
            }
        });
    } else {
        // 如果父路径是根，OverviewA 应该显示空列表（只有 "/"）
        set_overview_a_directories.set(Vec::new());
    }
}

/// 导航到根节点
fn handle_navigate_to_root(
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) {
    spawn_local(async move {
        match Request::get("/api/directories/root").send().await {
            Ok(resp) => {
                match resp.json::<DirectoriesResponse>().await {
                    Ok(data) => {
                        let dir_paths: Vec<String> = data.directories.iter()
                            .map(|d| d.path.clone())
                            .collect();
                        set_overview_b_directories.set(dir_paths);
                        
                        // 设置第一个目录用于 Preview
                        if let Some(first_dir) = data.directories.first() {
                            set_preview_path.set(Some(first_dir.path.clone()));
                        }
                        
                        // OverviewA 保持为空（只有 "/"）
                        set_overview_a_directories.set(Vec::new());
                        set_selected_path.set(None);
                        // 重置选中索引
                        set_selected_index.set(Some(0));
                    }
                    Err(_) => {}
                }
            }
            Err(_) => {}
        }
    });
}

