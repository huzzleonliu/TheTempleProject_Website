use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::console;
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
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
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
        let current_dirs: Vec<String> = directories
            .iter()
            .map(|d| d.path.clone())
            .collect();
        set_overview_a_directories.set(current_dirs);
        
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
                    let dir_paths: Vec<String> = children.iter()
                        .map(|d| d.path.clone())
                        .collect();
                    console::log_2(&"[鼠标点击] 加载子节点成功，数量:".into(), &children.len().into());
                    set_overview_b_directories.set(dir_paths);
                    
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

