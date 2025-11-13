use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::api::{get_child_directories, get_root_directories};
use crate::components::keyboard_handlers;
use crate::components::overview_a::OverviewA;
use crate::components::overview_b::OverviewB;
use crate::components::preview::Preview;
use crate::components::title::Title;
use crate::types::{parent_path, split_levels, DirectoryNode, NodesCache, ROOT_PATH};
use leptos::prelude::*;
use leptos::callback::UnsyncCallback;
use leptos::task::spawn_local;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

#[component]
pub fn Home() -> impl IntoView {
    let path_cache = RwSignal::new(HashMap::new());
    let current_path = RwSignal::new(None::<String>);
    let selected_index = RwSignal::new(None::<usize>);
    let pending_selected_index = RwSignal::new(None::<usize>);
    let preview_path = RwSignal::new(None::<String>);
    let preview_nodes = RwSignal::new(Vec::<DirectoryNode>::new());
    let preview_loading = RwSignal::new(false);
    let preview_error = RwSignal::new(None::<String>);

    let preview_scroll_ref = NodeRef::<leptos::html::Div>::new();

    // 当前层级下的节点列表（用于 OverviewB）
    let current_children = create_memo({
        let path_cache = path_cache.clone();
        let current_path = current_path.clone();
        move |_| {
            let cache = path_cache.get();
            let key = current_path
                .get()
                .unwrap_or_else(|| ROOT_PATH.to_string());
            cache.get(&key).cloned().unwrap_or_default()
        }
    });

    // OverviewA：展示当前节点上一级目录的节点列表
    let overview_a_nodes = create_memo({
        let path_cache = path_cache.clone();
        let current_path = current_path.clone();
        move |_| {
            let cache = path_cache.get();
            match current_path.get() {
                Some(path) => {
                    let parent = parent_path(&path).unwrap_or_else(|| ROOT_PATH.to_string());
                    cache.get(&parent).cloned().unwrap_or_default()
                }
                None => vec![DirectoryNode {
                    path: ROOT_PATH.to_string(),
                    raw_filename: "/".to_string(),
                    has_subnodes: true,
                }],
            }
        }
    });

    let overview_a_highlight = create_memo({
        let current_path = current_path.clone();
        move |_| Some(current_path.get().unwrap_or_else(|| ROOT_PATH.to_string()))
    });

    // 辅助函数：进入指定路径
    let navigate_to = Rc::new({
        let path_cache = path_cache.clone();
        let current_path = current_path.clone();
        let selected_index = selected_index.clone();
        let pending_selected_index = pending_selected_index.clone();
        let preview_path = preview_path.clone();
        move |target: Option<String>| {
            let path_cache = path_cache.clone();
            let current_path = current_path.clone();
            let selected_index = selected_index.clone();
            let pending_selected_index = pending_selected_index.clone();
            let preview_path = preview_path.clone();
            spawn_local(async move {
                if let Err(e) = ensure_path_and_ancestors(target.as_ref(), path_cache.clone()).await {
                    web_sys::console::log_2(&"[导航] 加载失败: ".into(), &e.into());
                    return;
                }

                let cache_key = target
                    .as_ref()
                    .cloned()
                    .unwrap_or_else(|| ROOT_PATH.to_string());
                let children = path_cache
                    .with(|map| map.get(&cache_key).cloned())
                    .unwrap_or_default();

                current_path.set(target.clone());

                let preferred_index = pending_selected_index.get_untracked();
                pending_selected_index.set(None);

                if children.is_empty() {
                    selected_index.set(None);
                    preview_path.set(None);
                } else {
                    let idx = preferred_index.unwrap_or(0).min(children.len() - 1);
                    selected_index.set(Some(idx));
                    let preview = children
                        .get(idx)
                        .and_then(|node| if node.has_subnodes { Some(node.path.clone()) } else { None });
                    preview_path.set(preview);
                }
            });
        }
    });

    // 选中某个 index（用于点击或键盘）
    let select_index = Rc::new({
        let selected_index = selected_index.clone();
        let current_children = current_children.clone();
        move |idx: usize| {
            let len = current_children.get_untracked().len();
            if len == 0 {
                selected_index.set(None);
            } else if idx < len {
                selected_index.set(Some(idx));
            }
        }
    });

    // 键盘移动选择
    let move_selection = Rc::new({
        let selected_index = selected_index.clone();
        let current_children = current_children.clone();
        move |delta: i32| {
            let len = current_children.get_untracked().len() as i32;
            if len == 0 {
                selected_index.set(None);
                return;
            }

            let current = selected_index.get_untracked().unwrap_or(0) as i32;
            let next = (current + delta).clamp(0, len - 1);
            if current != next {
                selected_index.set(Some(next as usize));
            }
        }
    });

    // 进入当前选中节点
    let enter_selection = Rc::new({
        let current_children = current_children.clone();
        let selected_index = selected_index.clone();
        let navigate_to = navigate_to.clone();
        move || {
            if let Some(idx) = selected_index.get_untracked() {
                if let Some(node) = current_children.get_untracked().get(idx) {
                    if node.has_subnodes {
                        navigate_to(Some(node.path.clone()));
                    }
                }
            }
        }
    });

    // 返回上一级
    let go_back = Rc::new({
        let current_path = current_path.clone();
        let navigate_to = navigate_to.clone();
        move || {
            let current = current_path.get_untracked();
            match current {
                Some(ref path) if !path.is_empty() => {
                    let parent = parent_path(path)
                        .and_then(|p| if p.is_empty() { None } else { Some(p) });
                    navigate_to(parent);
                }
                Some(_) => navigate_to(None),
                None => {}
            }
        }
    });

    // 选中索引变化时更新 preview_path（默认取第一个子节点）
    {
        let current_children = current_children.clone();
        let selected_index_signal = selected_index.clone();
        let preview_path_signal = preview_path.clone();
        create_effect(move |_| {
            let children = current_children.get();
            let len = children.len();
            let current_idx = selected_index_signal.get();

            let normalized_idx = if len == 0 {
                None
            } else {
                match current_idx {
                    Some(idx) if idx < len => Some(idx),
                    _ => Some(0),
                }
            };

            if current_idx != normalized_idx {
                selected_index_signal.set(normalized_idx);
            }

            let preview = normalized_idx
                .and_then(|idx| children.get(idx))
                .and_then(|node| if node.has_subnodes { Some(node.path.clone()) } else { None });
            preview_path_signal.set(preview);
        });
    }

    // 预览区域数据加载
    {
        let preview_path_signal = preview_path.clone();
        let path_cache = path_cache.clone();
        let preview_nodes = preview_nodes.clone();
        let preview_loading = preview_loading.clone();
        let preview_error = preview_error.clone();
        create_effect(move |_| {
            if let Some(path) = preview_path_signal.get() {
                preview_loading.set(true);
                preview_error.set(None);
                let path_cache = path_cache.clone();
                let preview_nodes = preview_nodes.clone();
                let preview_loading = preview_loading.clone();
                let preview_error = preview_error.clone();
                spawn_local(async move {
                    if let Err(e) = ensure_children(&path, path_cache.clone()).await {
                        preview_error.set(Some(e));
                        preview_nodes.set(Vec::new());
                    } else {
                        let nodes = path_cache
                            .with(|map| map.get(&path).cloned())
                            .unwrap_or_default();
                        preview_error.set(None);
                        preview_nodes.set(nodes);
                    }
                    preview_loading.set(false);
                });
            } else {
                preview_nodes.set(Vec::new());
                preview_error.set(None);
                preview_loading.set(false);
            }
        });
    }

    // 初始化加载根节点
    {
        let initialized = Rc::new(Cell::new(false));
        let path_cache = path_cache.clone();
        let current_path = current_path.clone();
        let selected_index = selected_index.clone();
        let preview_path = preview_path.clone();
        create_effect(move |_| {
            if initialized.get() {
                return;
            }
            initialized.set(true);

            let path_cache = path_cache.clone();
            let current_path = current_path.clone();
            let selected_index = selected_index.clone();
            let preview_path = preview_path.clone();

            spawn_local(async move {
                if let Err(e) = ensure_children(ROOT_PATH, path_cache.clone()).await {
                    web_sys::console::log_2(&"初始化失败: ".into(), &e.into());
                    return;
                }

                let children = path_cache
                    .with(|map| map.get(ROOT_PATH).cloned())
                    .unwrap_or_default();

                current_path.set(None);
                if children.is_empty() {
                    selected_index.set(None);
                    preview_path.set(None);
                } else {
                    selected_index.set(Some(0));
                    let preview = if children[0].has_subnodes {
                        Some(children[0].path.clone())
                    } else {
                        None
                    };
                    preview_path.set(preview);
                }
            });
        });
    }

    // OverviewA 点击（切换到某个父级节点）
    let overview_a_select_callback: UnsyncCallback<Option<String>> = {
        let navigate_to = navigate_to.clone();
        let path_cache = path_cache.clone();
        let pending_selected_index = pending_selected_index.clone();
        UnsyncCallback::new(move |target: Option<String>| {
            let navigate_to = navigate_to.clone();
            let path_cache = path_cache.clone();
            let pending_selected_index = pending_selected_index.clone();

            spawn_local(async move {
                let (dest_parent, highlight_path) = match target {
                    None => {
                        pending_selected_index.set(None);
                        (None, None)
                    }
                    Some(ref path) if path.is_empty() => {
                        pending_selected_index.set(None);
                        (None, None)
                    }
                    Some(path) => {
                        let parent = parent_path(&path).unwrap_or_else(|| ROOT_PATH.to_string());
                        (Some(parent), Some(path))
                    }
                };

                if let Some(ref parent) = dest_parent {
                    if let Err(e) = ensure_children(parent, path_cache.clone()).await {
                        web_sys::console::log_2(&"[OverviewA] 加载失败: ".into(), &e.into());
                        return;
                    }

                    if let Some(ref highlight_path) = highlight_path {
                        let siblings = path_cache
                            .with(|map| map.get(parent).cloned())
                            .unwrap_or_default();

                        if let Some(idx) = siblings
                            .iter()
                            .position(|node| node.path == *highlight_path)
                        {
                            pending_selected_index.set(Some(idx));
                        }
                    }
                }

                navigate_to(dest_parent.clone());

                if dest_parent.is_none() {
                    // select_index(None.unwrap_or_default()); // Removed as per edit hint
                }
            });
        })
    };

    // OverviewB 点击选择
    let select_index_callback: UnsyncCallback<usize> = {
        let select_index = select_index.clone();
        UnsyncCallback::new(move |idx: usize| select_index(idx))
    };

    // OverviewB 双击进入
    let enter_index_callback: UnsyncCallback<usize> = {
        let current_children = current_children.clone();
        let navigate_to = navigate_to.clone();
        UnsyncCallback::new(move |idx: usize| {
            if let Some(node) = current_children.get_untracked().get(idx) {
                if node.has_subnodes {
                    navigate_to(Some(node.path.clone()));
                }
            }
        })
    };

    // 键盘事件
    {
        let listener_added = Rc::new(Cell::new(false));
        let listener_added_ref = listener_added.clone();
        let move_selection = move_selection.clone();
        let enter_selection = enter_selection.clone();
        let go_back = go_back.clone();
        create_effect(move |_| {
            if listener_added_ref.get() {
                return;
            }
            listener_added_ref.set(true);

            let move_selection = move_selection.clone();
            let enter_selection = enter_selection.clone();
            let go_back = go_back.clone();

            let handle_global_keydown = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                if let Some(active_element) = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                {
                    let tag_name = active_element.tag_name();
                    if matches!(tag_name.as_str(), "INPUT" | "TEXTAREA")
                        || active_element.has_attribute("contenteditable")
                    {
                        return;
                    }
                }

                keyboard_handlers::handle_keyboard_navigation(
                    &event,
                    move_selection.clone(),
                    enter_selection.clone(),
                    go_back.clone(),
                    preview_scroll_ref,
                );
            }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

            if let Some(window) = web_sys::window() {
                let _ = window.add_event_listener_with_callback(
                    "keydown",
                    handle_global_keydown.as_ref().unchecked_ref(),
                );
                handle_global_keydown.forget();
            }
        });
    }

    view! {
        <div class="flex flex-col h-screen">
            <div class="px-4 pt-4 pb-0 flex-shrink-0">
                <Title/>
            </div>
            <div class="grid grid-cols-10 grid-rows-1 flex-1 min-h-0 overflow-hidden items-start">
                <div class="col-span-2 overflow-y-auto px-4 pt-0">
                    <OverviewA
                        nodes=overview_a_nodes
                        highlighted_path=overview_a_highlight
                        on_select=overview_a_select_callback
                    />
                </div>
                <div class="col-span-3 overflow-y-auto px-4 pt-0">
                    <OverviewB
                        nodes=current_children
                        selected_index=selected_index.read_only()
                        on_select=select_index_callback
                        on_enter=enter_index_callback
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 px-4 pt-0">
                    <Preview
                        nodes=preview_nodes.read_only()
                        loading=preview_loading.read_only()
                        error=preview_error.read_only()
                        scroll_container_ref=preview_scroll_ref
                    />
                </div>
            </div>
        </div>
    }
}

async fn ensure_children(path: &str, cache: RwSignal<NodesCache>) -> Result<(), String> {
    if cache.with(|map| map.contains_key(path)) {
        return Ok(());
    }

    let data = if path.is_empty() {
        get_root_directories().await
    } else {
        get_child_directories(path).await
    }?;

    cache.update(|map| {
        map.insert(path.to_string(), data);
    });

    Ok(())
}

async fn ensure_path_and_ancestors(path: Option<&String>, cache: RwSignal<NodesCache>) -> Result<(), String> {
    ensure_children(ROOT_PATH, cache.clone()).await?;

    if let Some(path) = path {
        for level in split_levels(path) {
            if let Some(parent) = parent_path(&level) {
                ensure_children(&parent, cache.clone()).await?;
            }
            ensure_children(&level, cache.clone()).await?;
        }
    }

    Ok(())
}
