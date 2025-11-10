use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use web_sys::console;

use crate::components::mouse_handlers::DirectoryNode;

/// API 响应数据结构
#[derive(Debug, Serialize, Deserialize)]
struct DirectoriesResponse {
    directories: Vec<DirectoryNode>,
}

/// 处理键盘导航事件（模仿 ranger 的导航方式）
/// 
/// # 键盘操作
/// - `j`: 向下移动光标（在 OverviewB 中）
/// - `k`: 向上移动光标（在 OverviewB 中）
/// - `l`: 进入子级节点
/// - `h`: 返回父级节点
/// - `Shift+J`: 向下滚动 Preview
/// - `Shift+K`: 向上滚动 Preview
/// 
/// # 参数
/// - `event`: 键盘事件
/// - `directories`: 当前目录列表（OverviewB）
/// - `selected_index`: 当前选中的索引（OverviewB）
/// - `overview_a_directories`: OverviewA 的目录列表（父级节点）
/// - `overview_a_selected_path`: OverviewA 中高亮的路径（当前节点的父级）
/// - `set_selected_index`: 设置选中索引的函数
/// - `set_selected_path`: 设置选中路径的函数（OverviewB）
/// - `set_overview_a_selected_path`: 设置 OverviewA 中高亮路径的函数
/// - `set_overview_a_directories`: 设置 OverviewA 目录列表的函数
/// - `set_overview_b_directories`: 设置 OverviewB 目录列表的函数
/// - `set_preview_path`: 设置 Preview 路径的函数
/// - `set_directories`: 设置 directories 的函数（用于返回时定位）
/// - `preview_scroll_ref`: Preview 滚动容器的引用
pub fn handle_keyboard_navigation(
    event: &web_sys::KeyboardEvent,
    directories: Vec<DirectoryNode>,
    selected_index: Option<usize>,
    overview_a_directories: Vec<String>,
    overview_a_selected_path: Option<String>,
    set_selected_index: WriteSignal<Option<usize>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
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
        match key.as_str() {
            "J" | "j" => {
                console::log_1(&"[键盘] Shift+J: 向下滚动 Preview".into());
                event.prevent_default();
                event.stop_propagation();
                scroll_preview_down(&preview_scroll_ref);
            }
            "K" | "k" => {
                console::log_1(&"[键盘] Shift+K: 向上滚动 Preview".into());
                event.prevent_default();
                event.stop_propagation();
                scroll_preview_up(&preview_scroll_ref);
            }
            _ => {}
        }
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
            // 向下移动光标（在 OverviewB 中）
            let new_index = if current_index < max_index {
                current_index + 1
            } else {
                max_index
            };
            console::log_2(&"[键盘] j: 向下移动".into(), &format!("索引: {} -> {}", current_index, new_index).into());
            set_selected_index.set(Some(new_index));
        }
        "k" => {
            // 向上移动光标（在 OverviewB 中）
            let new_index = if current_index > 0 {
                current_index - 1
            } else {
                0
            };
            console::log_2(&"[键盘] k: 向上移动".into(), &format!("索引: {} -> {}", current_index, new_index).into());
            set_selected_index.set(Some(new_index));
        }
        "l" => {
            // 进入子级节点
            console::log_1(&"[键盘] l: 进入子级节点".into());
            if let Some(dir) = directories.get(current_index) {
                console::log_2(&"  目标路径:".into(), &dir.path.clone().into());
            }
            handle_enter_node(
                current_index,
                &directories,
                set_selected_path,
                set_overview_a_selected_path,
                set_overview_a_directories,
                set_preview_path,
                set_overview_b_directories,
                set_selected_index,
            );
        }
        "h" => {
            // 返回父级节点
            console::log_1(&"[键盘] h: 返回父级节点".into());
            handle_go_back(
                &overview_a_directories,
                overview_a_selected_path,
                set_selected_path,
                set_overview_a_selected_path,
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

/// 向下滚动 Preview
fn scroll_preview_down(preview_scroll_ref: &NodeRef<leptos::html::Div>) {
    if let Some(container) = preview_scroll_ref.get() {
        let scroll_amount = 100.0;
        let current_scroll = container.scroll_top() as f64;
        let max_scroll = (container.scroll_height() - container.client_height()) as f64;
        let new_scroll = (current_scroll + scroll_amount).min(max_scroll);
        container.set_scroll_top(new_scroll as i32);
    }
}

/// 向上滚动 Preview
fn scroll_preview_up(preview_scroll_ref: &NodeRef<leptos::html::Div>) {
    if let Some(container) = preview_scroll_ref.get() {
        let scroll_amount = 100.0;
        let current_scroll = container.scroll_top() as f64;
        let new_scroll = (current_scroll - scroll_amount).max(0.0);
        container.set_scroll_top(new_scroll as i32);
    }
}

/// 处理进入节点（l 键）
/// 进入子级节点时，OverviewA 中的父级节点要高亮显示
fn handle_enter_node(
    current_index: usize,
    directories: &[DirectoryNode],
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) {
    if let Some(dir) = directories.get(current_index) {
        if dir.has_subnodes {
            let current_path = dir.path.clone();
            console::log_2(&"[进入节点] 路径:".into(), &current_path.clone().into());
            
            // 设置 OverviewB 中选中的路径
            set_selected_path.set(Some(current_path.clone()));
            
            // 将当前 OverviewB 的内容移到 OverviewA（作为父级节点）
            let current_dirs: Vec<String> = directories.iter()
                .map(|d| d.path.clone())
                .collect();
            console::log_2(&"[进入节点] 移动到 OverviewA 的节点数:".into(), &current_dirs.len().into());
            set_overview_a_directories.set(current_dirs);
            
            // 高亮 OverviewA 中的当前节点（作为父级）
            set_overview_a_selected_path.set(Some(current_path.clone()));
            console::log_2(&"[进入节点] OverviewA 高亮路径:".into(), &current_path.clone().into());
            
            // 设置 Preview 显示被点击节点的子节点
            set_preview_path.set(Some(current_path.clone()));
            
            // 加载被点击节点的子目录到 OverviewB
            let path_clone = current_path.clone();
            spawn_local(async move {
                let encoded_path = urlencoding::encode(&path_clone);
                let url = format!("/api/directories/children/{}", encoded_path);
                console::log_2(&"[进入节点] 请求子节点:".into(), &url.clone().into());
                
                match Request::get(&url).send().await {
                    Ok(resp) => {
                        match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                let dir_paths: Vec<String> = data.directories.iter()
                                    .map(|d| d.path.clone())
                                    .collect();
                                console::log_2(&"[进入节点] 加载子节点成功，数量:".into(), &data.directories.len().into());
                                set_overview_b_directories.set(dir_paths);
                                // 重置选中索引为 0
                                set_selected_index.set(Some(0));
                            }
                            Err(e) => {
                                console::log_2(&"[进入节点] 解析响应失败:".into(), &format!("{:?}", e).into());
                            }
                        }
                    }
                    Err(e) => {
                        console::log_2(&"[进入节点] 请求失败:".into(), &format!("{:?}", e).into());
                    }
                }
            });
        } else {
            console::log_1(&"[进入节点] 节点没有子节点，无法进入".into());
        }
    }
}

/// 处理返回父级节点（h 键）
/// 后退到父级节点时，光标要在高亮的节点上
fn handle_go_back(
    overview_a_directories: &[String],
    overview_a_selected_path: Option<String>,
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
    set_directories: WriteSignal<Vec<DirectoryNode>>,
) {
    // 使用 overview_a_selected_path 作为目标路径（当前节点的父级）
    // 这是进入节点时设置的，比 overview_a_directories.last() 更准确
    if let Some(parent_path) = overview_a_selected_path {
        let path_to_select = parent_path.clone();
        console::log_2(&"[返回父级] 目标路径（来自 overview_a_selected_path）:".into(), &path_to_select.clone().into());
        
        // 获取父路径（上一级）
        let grandparent_path = if path_to_select.contains('.') {
            let parts: Vec<&str> = path_to_select.split('.').collect();
            if parts.len() > 1 {
                Some(parts[0..parts.len()-1].join("."))
            } else {
                None
            }
        } else {
            None
        };
        
        // 设置 OverviewA 中高亮的路径（父级的父级）
        if let Some(ref gp) = grandparent_path {
            console::log_2(&"[返回父级] OverviewA 高亮路径:".into(), &gp.clone().into());
            set_overview_a_selected_path.set(Some(gp.clone()));
        } else {
            console::log_1(&"[返回父级] OverviewA 高亮路径: None (根节点)".into());
            set_overview_a_selected_path.set(None);
        }
        
        // 设置 Preview 显示父级节点的子节点
        let path_for_preview = path_to_select.clone();
        spawn_local(async move {
            let encoded_path = urlencoding::encode(&path_for_preview);
            let url = format!("/api/directories/children/{}", encoded_path);
            
            match Request::get(&url).send().await {
                Ok(resp) => {
                    match resp.json::<DirectoriesResponse>().await {
                        Ok(data) => {
                            if !data.directories.is_empty() {
                                set_preview_path.set(Some(path_for_preview.clone()));
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
        
        // 加载兄弟节点到 OverviewB（父级节点的子节点）
        let parent_for_b = grandparent_path.clone();
        let path_to_select_clone = path_to_select.clone();
        spawn_local(async move {
            let url = if let Some(p) = parent_for_b {
                let encoded_path = urlencoding::encode(&p);
                format!("/api/directories/children/{}", encoded_path)
            } else {
                "/api/directories/root".to_string()
            };
            console::log_2(&"[返回父级] 请求兄弟节点:".into(), &url.clone().into());
            
            match Request::get(&url).send().await {
                Ok(resp) => {
                    match resp.json::<DirectoriesResponse>().await {
                        Ok(data) => {
                            let dir_paths: Vec<String> = data.directories.iter()
                                .map(|d| d.path.clone())
                                .collect();
                            console::log_2(&"[返回父级] 加载兄弟节点成功，数量:".into(), &data.directories.len().into());
                            
                            // 先定位到之前选中的节点（父级节点），再设置 directories
                            // 这样可以避免 overview_b.rs 中的 effect 重置索引
                            let target_index = if let Some(index) = data.directories.iter().position(|d| d.path == path_to_select_clone) {
                                console::log_2(&"[返回父级] 定位到索引:".into(), &index.into());
                                Some(index)
                            } else {
                                console::log_1(&"[返回父级] 未找到目标节点，定位到索引 0".into());
                                Some(0)
                            };
                            
                            // 先设置索引，避免 effect 重置
                            if let Some(idx) = target_index {
                                set_selected_index.set(Some(idx));
                                set_selected_path.set(Some(path_to_select_clone.clone()));
                            }
                            
                            // 然后设置 directories 和 overview_b_directories
                            set_overview_b_directories.set(dir_paths);
                            set_directories.set(data.directories.clone());
                        }
                        Err(e) => {
                            console::log_2(&"[返回父级] 解析响应失败:".into(), &format!("{:?}", e).into());
                        }
                    }
                }
                Err(e) => {
                    console::log_2(&"[返回父级] 请求失败:".into(), &format!("{:?}", e).into());
                }
            }
        });
        
        // 加载上一级节点到 OverviewA
        if let Some(ref gp) = grandparent_path {
            let parent_for_a = if gp.contains('.') {
                let parts: Vec<&str> = gp.split('.').collect();
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
    } else if !overview_a_directories.is_empty() {
        // 如果 overview_a_selected_path 为空，但 overview_a_directories 不为空，使用最后一个节点作为后备
        if let Some(parent_path) = overview_a_directories.last() {
            let path_to_select = parent_path.clone();
            console::log_2(&"[返回父级] 目标路径（后备，来自 overview_a_directories.last）:".into(), &path_to_select.clone().into());
            
            // 获取父路径（上一级）
            let grandparent_path = if path_to_select.contains('.') {
                let parts: Vec<&str> = path_to_select.split('.').collect();
                if parts.len() > 1 {
                    Some(parts[0..parts.len()-1].join("."))
                } else {
                    None
                }
            } else {
                None
            };
            
            // 设置 OverviewA 中高亮的路径（父级的父级）
            if let Some(ref gp) = grandparent_path {
                console::log_2(&"[返回父级] OverviewA 高亮路径:".into(), &gp.clone().into());
                set_overview_a_selected_path.set(Some(gp.clone()));
            } else {
                console::log_1(&"[返回父级] OverviewA 高亮路径: None (根节点)".into());
                set_overview_a_selected_path.set(None);
            }
            
            // 设置 Preview 显示父级节点的子节点
            let path_for_preview = path_to_select.clone();
            spawn_local(async move {
                let encoded_path = urlencoding::encode(&path_for_preview);
                let url = format!("/api/directories/children/{}", encoded_path);
                
                match Request::get(&url).send().await {
                    Ok(resp) => {
                        match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                if !data.directories.is_empty() {
                                    set_preview_path.set(Some(path_for_preview.clone()));
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
            
            // 加载兄弟节点到 OverviewB（父级节点的子节点）
            let parent_for_b = grandparent_path.clone();
            let path_to_select_clone = path_to_select.clone();
            spawn_local(async move {
                let url = if let Some(p) = parent_for_b {
                    let encoded_path = urlencoding::encode(&p);
                    format!("/api/directories/children/{}", encoded_path)
                } else {
                    "/api/directories/root".to_string()
                };
                console::log_2(&"[返回父级] 请求兄弟节点:".into(), &url.clone().into());
                
                match Request::get(&url).send().await {
                    Ok(resp) => {
                        match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                let dir_paths: Vec<String> = data.directories.iter()
                                    .map(|d| d.path.clone())
                                    .collect();
                                console::log_2(&"[返回父级] 加载兄弟节点成功，数量:".into(), &data.directories.len().into());
                                
                                // 先定位到之前选中的节点（父级节点），再设置 directories
                                let target_index = if let Some(index) = data.directories.iter().position(|d| d.path == path_to_select_clone) {
                                    console::log_2(&"[返回父级] 定位到索引:".into(), &index.into());
                                    Some(index)
                                } else {
                                    console::log_1(&"[返回父级] 未找到目标节点，定位到索引 0".into());
                                    Some(0)
                                };
                                
                                // 先设置索引，避免 effect 重置
                                if let Some(idx) = target_index {
                                    set_selected_index.set(Some(idx));
                                    set_selected_path.set(Some(path_to_select_clone.clone()));
                                }
                                
                                // 然后设置 directories 和 overview_b_directories
                                set_overview_b_directories.set(dir_paths);
                                set_directories.set(data.directories.clone());
                            }
                            Err(e) => {
                                console::log_2(&"[返回父级] 解析响应失败:".into(), &format!("{:?}", e).into());
                            }
                        }
                    }
                    Err(e) => {
                        console::log_2(&"[返回父级] 请求失败:".into(), &format!("{:?}", e).into());
                    }
                }
            });
            
            // 加载上一级节点到 OverviewA
            if let Some(ref gp) = grandparent_path {
                let parent_for_a = if gp.contains('.') {
                    let parts: Vec<&str> = gp.split('.').collect();
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
    } else {
        // 如果 OverviewA 为空，导航到根节点
        console::log_1(&"[返回父级] OverviewA 为空，导航到根节点".into());
        spawn_local(async move {
            match Request::get("/api/directories/root").send().await {
                Ok(resp) => {
                    match resp.json::<DirectoriesResponse>().await {
                        Ok(data) => {
                            let dir_paths: Vec<String> = data.directories.iter()
                                .map(|d| d.path.clone())
                                .collect();
                            console::log_2(&"[返回父级] 加载根节点成功，数量:".into(), &data.directories.len().into());
                            set_overview_b_directories.set(dir_paths);
                            
                            if let Some(first_dir) = data.directories.first() {
                                set_preview_path.set(Some(first_dir.path.clone()));
                            }
                            
                            set_overview_a_directories.set(Vec::new());
                            set_overview_a_selected_path.set(None);
                            set_selected_path.set(None);
                            set_selected_index.set(Some(0));
                        }
                        Err(e) => {
                            console::log_2(&"[返回父级] 解析响应失败:".into(), &format!("{:?}", e).into());
                        }
                    }
                }
                Err(e) => {
                    console::log_2(&"[返回父级] 请求失败:".into(), &format!("{:?}", e).into());
                }
            }
        });
    }
}
