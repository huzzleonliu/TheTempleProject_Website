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

#[component]
pub fn OverviewB(
    overview_b_directories: ReadSignal<Vec<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    selected_path: ReadSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_preview_path: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (directories, set_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // 当 overview_b_directories 改变时，加载对应的目录信息
    create_effect(move |_| {
        let dir_paths = overview_b_directories.get();
        let dir_paths_clone = dir_paths.clone();
        
        spawn_local(async move {
            if dir_paths_clone.is_empty() {
                // 初始加载一级目录
                set_loading.set(true);
                set_error.set(None);

                match Request::get("/api/directories/root").send().await {
                    Ok(resp) => {
                        match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                set_directories.set(data.directories.clone());
                                
                                // 设置第一个有子节点的目录用于 Preview
                                if let Some(first_dir) = data.directories.iter().find(|d| d.has_subnodes) {
                                    set_preview_path.set(Some(first_dir.path.clone()));
                                } else {
                                    set_preview_path.set(None);
                                }
                                set_loading.set(false);
                            }
                            Err(e) => {
                                set_error.set(Some(format!("解析错误: {e}")));
                                set_loading.set(false);
                            }
                        }
                    }
                    Err(e) => {
                        set_error.set(Some(format!("请求失败: {e}")));
                        set_loading.set(false);
                    }
                }
            } else {
                // 根据路径列表，获取父路径，然后加载兄弟节点
                if let Some(first_path) = dir_paths_clone.first() {
                    // 获取父路径
                    let parent_path = if first_path.contains('.') {
                        let parts: Vec<&str> = first_path.split('.').collect();
                        if parts.len() > 1 {
                            Some(parts[0..parts.len()-1].join("."))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    set_loading.set(true);
                    set_error.set(None);
                    
                    let url = if let Some(p) = parent_path {
                        let encoded_path = urlencoding::encode(&p);
                        format!("/api/directories/children/{}", encoded_path)
                    } else {
                        "/api/directories/root".to_string()
                    };
                    
                    match Request::get(&url).send().await {
                        Ok(resp) => {
                            match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                set_directories.set(data.directories.clone());
                                
                                // 设置第一个有子节点的目录用于 Preview
                                if let Some(first_dir) = data.directories.iter().find(|d| d.has_subnodes) {
                                    set_preview_path.set(Some(first_dir.path.clone()));
                                } else {
                                    set_preview_path.set(None);
                                }
                                set_loading.set(false);
                            }
                                Err(e) => {
                                    set_error.set(Some(format!("解析错误: {e}")));
                                    set_loading.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            set_error.set(Some(format!("请求失败: {e}")));
                            set_loading.set(false);
                        }
                    }
                }
            }
        });
    });

    view! {
        <ul class="text-2xl text-gray-500">
            <Show
                when=move || loading.get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || error.get().is_some()
                            fallback=move || {
                                view! {
                                    <For
                                        each=move || directories.get()
                                        key=|dir| dir.path.clone()
                                        children=move |dir: DirectoryNode| {
                                            let path = dir.path.clone();
                                            let path_for_selected = path.clone();
                                            let has_subnodes = dir.has_subnodes;
                                            let display_name = path.split('.').last().unwrap_or(&path).to_string();
                                            let is_selected = move || selected_path.get().as_ref() == Some(&path_for_selected);
                                            
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
                                                            // 设置选中的路径
                                                            set_selected_path.set(Some(path.clone()));
                                                            
                                                            // 只有当节点有子节点时才执行跳转
                                                            if has_subnodes {
                                                                // 将当前 OverviewB 的内容移到 OverviewA
                                                                let current_dirs: Vec<String> = directories.get()
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
                                                    >
                                                        {display_name}
                                                    </button>
                                                </li>
                                            }
                                        }
                                    />
                                }
                            }
                        >
                            <li class="text-red-500">{move || error.get().unwrap_or_else(|| "未知错误".to_string())}</li>
                        </Show>
                    }
                }
            >
                <li>"加载中..."</li>
            </Show>
        </ul>
    }
}
