use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::console;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::DirectoryNode;
use crate::api::get_child_directories;

// 类型已移动到 crate::types

/// 处理鼠标点击节点时的导航逻辑
/// 
/// # 参数
/// - `path`: 被点击的节点路径
/// - `has_subnodes`: 节点是否有子节点
/// - `directories`: 当前目录列表（用于获取完整路径列表）
/// - `set_overview_a_directories`: 设置 OverviewA 的目录列表
/// - `set_overview_a_selected_path`: 设置 OverviewA 中高亮路径的函数
/// - `set_overview_b_directories`: 设置 OverviewB 的目录列表
/// - `set_preview_path`: 设置 Preview 显示的路径
/// - `set_selected_path`: 设置当前选中的路径
/// - `set_selected_index`: 设置当前选中的索引
pub fn handle_node_click(
    path: String,
    has_subnodes: bool,
    directories: Vec<DirectoryNode>,
    set_overview_a_directories: WriteSignal<Vec<DirectoryNode>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<DirectoryNode>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) {
    console::log_2(&"[鼠标点击] 路径:".into(), &path.clone().into());
    console::log_2(&"[鼠标点击] 有子节点:".into(), &has_subnodes.into());
    
    // 设置选中的路径（用于高亮显示）
    set_selected_path.set(Some(path.clone()));
    
    // 只有当节点有子节点时才执行跳转
    if has_subnodes {
        console::log_1(&"[鼠标点击] 进入子节点".into());
        // 将当前 OverviewB 的内容移到 OverviewA（作为父级节点）
        set_overview_a_directories.set(directories.clone());
        
        // 高亮 OverviewA 中的当前节点（作为父级）
        set_overview_a_selected_path.set(Some(path.clone()));
        
        // 先清空 Preview 和 selected_index，等待子节点加载完成后再根据 selected_index 更新
        set_preview_path.set(None);
        set_selected_index.set(None);
        
        // 先清空 directories，避免旧的 directories 触发 Preview 更新
        // 注意：这里需要传递 set_directories，但 mouse_handlers 没有这个参数
        // 所以我们需要在 overview_b.rs 中处理这个问题
        
        // 加载被点击节点的子目录到 OverviewB
        let path_clone = path.clone();
        let set_selected_index_clone = set_selected_index.clone();
        spawn_local(async move {
            console::log_2(&"[鼠标点击] 请求子节点:".into(), &path_clone.clone().into());
            match get_child_directories(&path_clone).await {
                Ok(children) => {
                    console::log_2(&"[鼠标点击] 加载子节点成功，数量:".into(), &children.len().into());
                    set_overview_b_directories.set(children);
                    
                    // 等待 overview_b.rs 的 effect 加载完新的 directories 后再设置 selected_index
                    // 使用 request_animation_frame 延迟，确保 directories 已经更新
                    use wasm_bindgen::prelude::*;
                    use wasm_bindgen::JsCast;
                    if let Some(window) = web_sys::window() {
                        let closure = Closure::once_into_js(move || {
                            // 设置 selected_index 为 0，这会触发 overview_b.rs 的 effect 更新 Preview
                            set_selected_index_clone.set(Some(0));
                        });
                        let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
                    } else {
                        // 如果无法使用 request_animation_frame，直接设置
                        set_selected_index_clone.set(Some(0));
                    }
                }
                Err(e) => {
                    console::log_2(&"[鼠标点击] 请求失败:".into(), &e.into());
                }
            }
        });
    } else {
        // 如果没有子节点，不设置 Preview，也不跳转
        console::log_1(&"[鼠标点击] 节点没有子节点，不跳转".into());
        set_preview_path.set(None);
    }
}

/// 处理 OverviewA 中点击父级节点时的导航逻辑
/// 
/// # 参数
/// - `path`: 被点击的父级节点路径
/// - `set_selected_path`: 设置选中路径的函数
/// - `set_overview_b_directories`: 设置 OverviewB 目录列表的函数
/// - `set_overview_a_directories`: 设置 OverviewA 目录列表的函数
/// - `set_preview_path`: 设置 Preview 显示路径的函数
/// - `set_selected_index`: 设置选中索引的函数
pub fn handle_overview_a_click(
    path: String,
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<DirectoryNode>>,
    set_overview_a_directories: WriteSignal<Vec<DirectoryNode>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) {
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
    set_selected_path.set(Some(path.clone()));
    
    // 先获取被点击节点的信息，检查是否有子节点
    let path_for_info = path.clone();
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
    let set_selected_index_clone = set_selected_index.clone();
    let set_preview_path_clone = set_preview_path.clone();
    spawn_local(async move {
        let result = if let Some(p) = parent_for_b {
            get_child_directories(&p).await
        } else {
            // 如果父路径为空，加载 "1_OnceAndOnceAgain" 的子节点
            get_child_directories("1_OnceAndOnceAgain").await
        };
        
        if let Ok(data_dirs) = result {
            // 先设置 overview_b_directories，这会触发 overview_b.rs 的 effect 加载新的 directories
            set_overview_b_directories.set(data_dirs.clone());
            
            // 找到被点击的节点在兄弟节点列表中的索引
            if let Some(index) = data_dirs.iter().position(|d| d.path == clicked_path) {
                console::log_2(&"[OverviewA] 找到被点击节点在兄弟节点列表中的索引:".into(), &index.into());
                
                // 等待 overview_b.rs 的 effect 加载完新的 directories 后再设置 selected_index
                // 使用双重 request_animation_frame 延迟，确保 directories 已经更新
                let set_selected_index_delayed = set_selected_index_clone.clone();
                let set_preview_path_delayed = set_preview_path_clone.clone();
                let clicked_dir = data_dirs.get(index).cloned();
                
                if let Some(window) = web_sys::window() {
                    // 第一次延迟：等待 overview_b_directories 的 effect 触发
                    let closure1 = Closure::once_into_js(move || {
                        // 第二次延迟：等待 directories 加载完成
                        if let Some(window2) = web_sys::window() {
                            let closure2 = Closure::once_into_js(move || {
                                // 设置 selected_index，这会触发 overview_b.rs 的 effect 更新 Preview
                                set_selected_index_delayed.set(Some(index));
                                
                                // 同时立即设置 Preview，确保第一时间显示
                                if let Some(dir) = clicked_dir {
                                    if dir.has_subnodes {
                                        console::log_2(&"[OverviewA] 立即设置 Preview 路径:".into(), &dir.path.clone().into());
                                        set_preview_path_delayed.set(Some(dir.path.clone()));
                                    } else {
                                        set_preview_path_delayed.set(None);
                                    }
                                }
                            });
                            let _ = window2.request_animation_frame(closure2.as_ref().unchecked_ref());
                        } else {
                            // 如果无法使用 request_animation_frame，直接设置
                            set_selected_index_delayed.set(Some(index));
                            if let Some(dir) = clicked_dir.as_ref() {
                                if dir.has_subnodes {
                                    set_preview_path_delayed.set(Some(dir.path.clone()));
                                } else {
                                    set_preview_path_delayed.set(None);
                                }
                            }
                        }
                    });
                    let _ = window.request_animation_frame(closure1.as_ref().unchecked_ref());
                } else {
                    // 如果无法使用 request_animation_frame，直接设置
                    set_selected_index_clone.set(Some(index));
                    if let Some(dir) = data_dirs.get(index) {
                        if dir.has_subnodes {
                            set_preview_path_clone.set(Some(dir.path.clone()));
                        } else {
                            set_preview_path_clone.set(None);
                        }
                    }
                }
            } else {
                console::log_1(&"[OverviewA] 未找到被点击节点在兄弟节点列表中".into());
                // 如果找不到，设置第一个有子节点的目录
                if let Some(first_dir) = data_dirs.iter().find(|d| d.has_subnodes) {
                    set_selected_index_clone.set(Some(0));
                    set_preview_path_clone.set(Some(first_dir.path.clone()));
                } else {
                    set_selected_index_clone.set(Some(0));
                    set_preview_path_clone.set(None);
                }
            }
        } else {
            // ignore errors here
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
            let result = if let Some(p) = parent_for_a {
                get_child_directories(&p).await
            } else {
                // 如果父路径为空，加载 "1_OnceAndOnceAgain" 的子节点
                get_child_directories("1_OnceAndOnceAgain").await
            };
            if let Ok(data_dirs) = result {
                set_overview_a_directories.set(data_dirs);
            }
        });
    } else {
        // 如果父路径为空，OverviewA 应该显示空列表
        set_overview_a_directories.set(Vec::new());
    }
}

