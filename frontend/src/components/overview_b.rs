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

/// OverviewB 组件：显示当前层级的目录列表
/// 
/// # 功能
/// - 显示当前层级的目录列表
/// - 支持鼠标点击导航
/// - 支持键盘导航（j/k/l/h 键）
/// - 支持 Shift+J/K 滚动 Preview
/// - 自动聚焦以接收键盘事件
/// 
/// # 参数
/// - `overview_b_directories`: 当前层级的目录路径列表（字符串）
/// - `set_overview_b_directories`: 设置当前层级目录列表的函数
/// - `set_overview_a_directories`: 设置 OverviewA 目录列表的函数
/// - `selected_path`: 当前选中的路径（用于高亮显示）
/// - `set_selected_path`: 设置选中路径的函数
/// - `set_preview_path`: 设置 Preview 显示路径的函数
/// - `selected_index`: 当前选中的索引（用于键盘导航）
/// - `set_selected_index`: 设置选中索引的函数
/// - `overview_a_directories`: OverviewA 的目录列表（用于返回导航）
/// - `overview_a_selected_path`: OverviewA 中高亮的路径（用于返回导航）
/// - `set_overview_a_selected_path`: 设置 OverviewA 高亮路径的函数
/// - `directories`: 当前目录的完整信息（从外部传入，供全局键盘事件使用）
/// - `set_directories`: 设置目录信息的函数
/// - `preview_scroll_ref`: Preview 滚动容器的引用（用于 Shift+J/K 滚动）
#[component]
pub fn OverviewB(
    overview_b_directories: ReadSignal<Vec<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    selected_path: ReadSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    selected_index: ReadSignal<Option<usize>>,
    set_selected_index: WriteSignal<Option<usize>>,
    overview_a_directories: ReadSignal<Vec<String>>,
    overview_a_selected_path: ReadSignal<Option<String>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
    directories: ReadSignal<Vec<DirectoryNode>>,
    set_directories: WriteSignal<Vec<DirectoryNode>>,
    preview_scroll_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    // 加载状态
    let (loading, set_loading) = signal(false);
    // 错误信息
    let (error, set_error) = signal::<Option<String>>(None);

    // 当 directories 改变时，如果索引未设置或超出范围，则重置为 0
    // 注意：这个 effect 不应该覆盖已经正确设置的索引（比如返回父级节点时）
    create_effect(move |_| {
        let dirs = directories.get();
        if !dirs.is_empty() {
            // 如果当前索引超出范围或未设置，则重置为 0
            if let Some(current_idx) = selected_index.get() {
                if current_idx >= dirs.len() {
                    console::log_2(&"[OverviewB] 索引超出范围，重置为 0。当前索引:".into(), &current_idx.into());
                    console::log_2(&"[OverviewB] 列表长度:".into(), &dirs.len().into());
                    set_selected_index.set(Some(0));
                }
            } else {
                console::log_1(&"[OverviewB] 索引未设置，重置为 0".into());
                set_selected_index.set(Some(0));
            }
        }
    });

    // 当选中索引改变时，更新 Preview 显示的内容
    create_effect(move |_| {
        if let Some(index) = selected_index.get() {
            let dirs = directories.get();
            if let Some(dir) = dirs.get(index) {
                // 设置选中的路径（用于高亮显示）
                set_selected_path.set(Some(dir.path.clone()));
                
                // 只有当节点有子节点时才更新 Preview
                if dir.has_subnodes {
                    console::log_2(&"[OverviewB] 选中索引改变，更新 Preview:".into(), &dir.path.clone().into());
                    set_preview_path.set(Some(dir.path.clone()));
                } else {
                    console::log_2(&"[OverviewB] 选中索引改变，节点无子节点:".into(), &dir.path.clone().into());
                    set_preview_path.set(None);
                }
            }
        }
    });

    // 键盘事件处理已移至 home.rs 的全局监听器

    // 当 overview_b_directories 改变时，从 API 加载对应的完整目录信息
    // 这个 effect 负责将路径列表转换为包含完整信息的 DirectoryNode 列表
    create_effect(move |_| {
        let dir_paths = overview_b_directories.get();
        let dir_paths_clone = dir_paths.clone();
        
        spawn_local(async move {
            if dir_paths_clone.is_empty() {
                // 初始加载一级目录
                console::log_1(&"[OverviewB] 初始加载一级目录".into());
                set_loading.set(true);
                set_error.set(None);

                match Request::get("/api/directories/root").send().await {
                    Ok(resp) => {
                        match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                console::log_2(&"[OverviewB] 加载根目录成功，数量:".into(), &data.directories.len().into());
                                set_directories.set(data.directories.clone());
                                
                                // 设置第一个有子节点的目录用于 Preview
                                if let Some(first_dir) = data.directories.iter().find(|d| d.has_subnodes) {
                                    console::log_2(&"[OverviewB] 设置 Preview 路径:".into(), &first_dir.path.clone().into());
                                    set_preview_path.set(Some(first_dir.path.clone()));
                                } else {
                                    console::log_1(&"[OverviewB] 没有找到有子节点的目录".into());
                                    set_preview_path.set(None);
                                }
                                set_loading.set(false);
                            }
                            Err(e) => {
                                console::log_2(&"[OverviewB] 解析响应失败:".into(), &format!("{:?}", e).into());
                                set_error.set(Some(format!("解析错误: {e}")));
                                set_loading.set(false);
                            }
                        }
                    }
                    Err(e) => {
                        console::log_2(&"[OverviewB] 请求失败:".into(), &format!("{:?}", e).into());
                        set_error.set(Some(format!("请求失败: {e}")));
                        set_loading.set(false);
                    }
                }
            } else {
                // 根据路径列表，获取父路径，然后加载兄弟节点
                if let Some(first_path) = dir_paths_clone.first() {
                    console::log_2(&"[OverviewB] 加载目录信息，第一个路径:".into(), &first_path.clone().into());
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
                    console::log_2(&"[OverviewB] 请求 URL:".into(), &url.clone().into());
                    
                    match Request::get(&url).send().await {
                        Ok(resp) => {
                            match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                console::log_2(&"[OverviewB] 加载目录信息成功，数量:".into(), &data.directories.len().into());
                                set_directories.set(data.directories.clone());
                                
                                // 如果 selected_index 已设置，立即根据 selected_index 更新 Preview
                                // 这样可以确保在进入新节点时，Preview 能第一时间刷新
                                if let Some(index) = selected_index.get() {
                                    if let Some(dir) = data.directories.get(index) {
                                        if dir.has_subnodes {
                                            console::log_2(&"[OverviewB] directories 加载完成，根据 selected_index 更新 Preview:".into(), &dir.path.clone().into());
                                            set_preview_path.set(Some(dir.path.clone()));
                                        } else {
                                            console::log_2(&"[OverviewB] directories 加载完成，节点无子节点:".into(), &dir.path.clone().into());
                                            set_preview_path.set(None);
                                        }
                                    } else {
                                        console::log_2(&"[OverviewB] selected_index 超出范围，重置 Preview".into(), &index.into());
                                        set_preview_path.set(None);
                                    }
                                } else {
                                    // 如果 selected_index 未设置，设置第一个有子节点的目录用于 Preview
                                    if let Some(first_dir) = data.directories.iter().find(|d| d.has_subnodes) {
                                        console::log_2(&"[OverviewB] 设置 Preview 路径:".into(), &first_dir.path.clone().into());
                                        set_preview_path.set(Some(first_dir.path.clone()));
                                    } else {
                                        console::log_1(&"[OverviewB] 没有找到有子节点的目录".into());
                                        set_preview_path.set(None);
                                    }
                                }
                                set_loading.set(false);
                            }
                                Err(e) => {
                                    console::log_2(&"[OverviewB] 解析响应失败:".into(), &format!("{:?}", e).into());
                                    set_error.set(Some(format!("解析错误: {e}")));
                                    set_loading.set(false);
                                }
                            }
                        }
                        Err(e) => {
                            console::log_2(&"[OverviewB] 请求失败:".into(), &format!("{:?}", e).into());
                            set_error.set(Some(format!("请求失败: {e}")));
                            set_loading.set(false);
                        }
                    }
                }
            }
        });
    });

    // 自动聚焦逻辑已移除，键盘事件现在在 home.rs 中全局处理

    view! {
        <ul 
            class="text-2xl text-gray-500 outline-none"
        >
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
                                            // 提取目录信息
                                            let path = dir.path.clone();
                                            let path_for_selected = path.clone();
                                            let has_subnodes = dir.has_subnodes;
                                            // 显示名称：取路径的最后一部分（ltree 格式用点分隔）
                                            let display_name = path.split('.').last().unwrap_or(&path).to_string();
                                            
                                            // 判断当前节点是否被选中（用于高亮显示）
                                            let is_selected = move || {
                                                if let Some(index) = selected_index.get() {
                                                    let dirs = directories.get();
                                                    if let Some(dir_idx) = dirs.iter().position(|d| d.path == path_for_selected) {
                                                        index == dir_idx
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    selected_path.get().as_ref() == Some(&path_for_selected)
                                                }
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
                                                            // 鼠标点击处理 - 委托给 mouse_handlers 模块
                                                            crate::components::mouse_handlers::handle_node_click(
                                                                path.clone(),
                                                                has_subnodes,
                                                                directories.get(),
                                                                set_overview_a_directories,
                                                                set_overview_a_selected_path,
                                                                set_overview_b_directories,
                                                                set_preview_path,
                                                                set_selected_path,
                                                                set_selected_index,
                                                            );
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
