use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::ev::KeyboardEvent;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
    selected_index: ReadSignal<Option<usize>>,
    set_selected_index: WriteSignal<Option<usize>>,
    overview_a_directories: ReadSignal<Vec<String>>,
    preview_scroll_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let (directories, set_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // 当 directories 改变时，如果索引未设置，则重置为 0
    create_effect(move |_| {
        let dirs = directories.get();
        if !dirs.is_empty() {
            // 如果当前索引超出范围或未设置，则重置为 0
            if let Some(current_idx) = selected_index.get() {
                if current_idx >= dirs.len() {
                    set_selected_index.set(Some(0));
                }
            } else {
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
                    set_preview_path.set(Some(dir.path.clone()));
                } else {
                    set_preview_path.set(None);
                }
            }
        }
    });

    // 键盘事件处理函数
    let handle_keydown = move |event: KeyboardEvent| {
        let key = event.key();
        let shift_pressed = event.shift_key();
        let dirs = directories.get();
        
        // 处理 Shift+J 和 Shift+K 滚动 Preview
        if shift_pressed {
            match key.as_str() {
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
                    return;
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
                    return;
                }
                _ => {}
            }
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
        
        if dirs.is_empty() {
            return;
        }
        
        let current_index = selected_index.get().unwrap_or(0);
        let max_index = dirs.len() - 1;
        
        match key.as_str() {
            "j" => {
                // 向下移动
                let new_index = if current_index < max_index {
                    current_index + 1
                } else {
                    max_index
                };
                set_selected_index.set(Some(new_index));
            }
            "k" => {
                // 向上移动
                let new_index = if current_index > 0 {
                    current_index - 1
                } else {
                    0
                };
                set_selected_index.set(Some(new_index));
            }
            "l" => {
                // 进入当前选中的节点（相当于点击）
                if let Some(dir) = dirs.get(current_index) {
                    if dir.has_subnodes {
                        // 设置选中的路径
                        set_selected_path.set(Some(dir.path.clone()));
                        
                        // 将当前 OverviewB 的内容移到 OverviewA
                        let current_dirs: Vec<String> = dirs.iter()
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
            "h" => {
                // 回到上层节点（相当于点击 OverviewA 中的最后一个节点或 "/"）
                let overview_a_dirs = overview_a_directories.get();
                if !overview_a_dirs.is_empty() {
                    // 点击 OverviewA 中的最后一个节点
                    if let Some(last_path) = overview_a_dirs.last() {
                        let path_clone = last_path.clone();
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
                } else {
                    // 如果 OverviewA 为空，点击 "/"
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
            }
            _ => {}
        }
    };

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

    // 使用 NodeRef 来在组件挂载时聚焦到 ul 元素
    let ul_ref = NodeRef::<leptos::html::Ul>::new();
    
    // 当 directories 加载完成时，聚焦到 ul 元素
    create_effect(move |_| {
        let dirs = directories.get();
        if !dirs.is_empty() {
            // 使用 requestAnimationFrame 确保 DOM 已渲染
            if let Some(window) = web_sys::window() {
                let ul_ref_clone = ul_ref.clone();
                let closure = Closure::once_into_js(move || {
                    if let Some(ul) = ul_ref_clone.get() {
                        let _ = ul.focus();
                    }
                });
                let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
            }
        }
    });

    view! {
        <ul 
            node_ref=ul_ref
            class="text-2xl text-gray-500 outline-none focus:outline-none"
            tabindex="0"
            on:keydown=handle_keydown
            on:focus=move |_| {
                // 当元素获得焦点时，确保有选中的索引
                if selected_index.get().is_none() && !directories.get().is_empty() {
                    set_selected_index.set(Some(0));
                }
            }
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
                                            let path = dir.path.clone();
                                            let path_for_selected = path.clone();
                                            let has_subnodes = dir.has_subnodes;
                                            let display_name = path.split('.').last().unwrap_or(&path).to_string();
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
                                                            // 设置选中的路径和索引
                                                            let dirs = directories.get();
                                                            if let Some(idx) = dirs.iter().position(|d| d.path == path) {
                                                                set_selected_index.set(Some(idx));
                                                            }
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
