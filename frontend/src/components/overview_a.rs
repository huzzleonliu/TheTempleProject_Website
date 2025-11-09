use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DirectoryNode {
    path: String,
    has_layout: bool,
    has_visual_assets: bool,
    has_text: i32,
    has_images: i32,
    has_subnodes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoriesResponse {
    directories: Vec<DirectoryNode>,
}

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
) -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500">
            <li>
                <button
                    class="w-full h-full text-left hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400"
                    on:click=move |_| {
                        // 点击 "/" 时，加载一级目录到 OverviewB，但不移动内容
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
                                        }
                                        Err(_) => {}
                                    }
                                }
                                Err(_) => {}
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
                    let display_name = path.split('.').last().unwrap_or(&path).to_string();
                    // 判断当前节点是否被选中（用于高亮显示）
                    let is_selected = move || {
                        overview_a_selected_path.get().as_ref() == Some(&path_for_selected)
                    };
                    
                    view! {
                        <li>
                            <button
                                class=move || {
                                    if is_selected() {
                                        "w-full h-full text-left text-white bg-gray-800"
                                    } else {
                                        "w-full h-full text-left hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400"
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
