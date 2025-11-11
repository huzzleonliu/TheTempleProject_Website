use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::console;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::DirectoryNode;
use crate::api::{get_child_directories, get_root_directories};

// 类型已移动到 crate::types

/// OverviewA 组件：显示父级节点列表
/// 
/// # 功能
/// - 显示父级节点列表（当前节点的祖先节点）
/// - 支持高亮显示当前节点的父级
/// - 支持鼠标点击导航
#[component]
pub fn OverviewA(
    overview_a_directories: ReadSignal<Vec<String>>,
    overview_a_selected_path: ReadSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500">
            <li class="w-full min-w-0">
                <button
                    class="w-full h-full text-left hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400 truncate"
                    on:click=move |_| {
                        // 点击 "/" 时，加载一级目录到 OverviewB，但不移动内容
                        spawn_local(async move {
                            if let Ok(directories) = get_root_directories().await {
                                let dir_paths: Vec<String> = directories.iter()
                                    .map(|d| d.path.clone())
                                    .collect();
                                set_overview_b_directories.set(dir_paths);
                                
                                // 设置第一个目录用于 Preview
                                if let Some(first_dir) = directories.first() {
                                    set_preview_path.set(Some(first_dir.path.clone()));
                                }
                                
                                // OverviewA 保持为空（只有 "/"）
                                set_overview_a_directories.set(Vec::new());
                                set_selected_path.set(None);
                            }
                        });
                    }
                >
                    "/"
                </button>
            </li>
            <For
                each=move || overview_a_directories.get()
                key=|path| path.clone()
                children=move |path: String| {
                    let path_clone = path.clone();
                    let path_for_selected = path_clone.clone();
                    // 注意：OverviewA 显示的是路径字符串，不是 DirectoryNode
                    // 所以这里仍然从路径提取，但实际应该从数据库获取 raw_filename
                    // 暂时保持原逻辑，因为 OverviewA 使用的是路径列表，不是完整的 DirectoryNode
                    let display_name = path.split('.').last().unwrap_or(&path).to_string();
                    // 判断当前节点是否被选中（用于高亮显示）
                    let is_selected = move || {
                        overview_a_selected_path.get().as_ref() == Some(&path_for_selected)
                    };
                    
                    view! {
                        <li class="w-full min-w-0">
                            <button
                                class=move || {
                                    if is_selected() {
                                        "w-full h-full text-left text-white bg-gray-800 truncate"
                                    } else {
                                        "w-full h-full text-left hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400 truncate"
                                    }
                                }
                                on:click=move |_| {
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
                                    let clicked_path = path_clone.clone();
                                    let set_selected_index_clone = set_selected_index.clone();
                                    let set_preview_path_clone = set_preview_path.clone();
                                    spawn_local(async move {
                                        let result = if let Some(p) = parent_for_b {
                                            get_child_directories(&p).await
                                        } else {
                                            get_root_directories().await
                                        };
                                        
                                        if let Ok(data_dirs) = result {
                                            let dir_paths: Vec<String> = data_dirs.iter()
                                                .map(|d| d.path.clone())
                                                .collect();
                                            // 先设置 overview_b_directories，这会触发 overview_b.rs 的 effect 加载新的 directories
                                            set_overview_b_directories.set(dir_paths.clone());
                                            
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
                                                get_root_directories().await
                                            };
                                            if let Ok(data_dirs) = result {
                                                let dir_paths: Vec<String> = data_dirs.iter()
                                                    .map(|d| d.path.clone())
                                                    .collect();
                                                set_overview_a_directories.set(dir_paths);
                                            }
                                        });
                                    } else {
                                        // 如果父路径是根，OverviewA 应该显示空列表（只有 "/"）
                                        set_overview_a_directories.set(Vec::new());
                                    }
                                }
                            >
                                {display_name}
                            </button>
                        </li>
                    }
                }
            />
        </ul>
    }
}
