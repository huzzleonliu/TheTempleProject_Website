use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use gloo_net::http::Request;
use leptos::callback::UnsyncCallback;
use leptos::prelude::*;
use leptos::task::spawn_local;
use pulldown_cmark::{html, Options, Parser};
use serde_json;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::api::{get_child_directories, get_node_assets, get_root_directories};
use crate::components::keyboard_handlers;
use crate::types::{
    parent_path, split_levels, AssetNode, AssetsCache, DetailItem, DirectoryNode, NodeKind,
    NodesCache, UiNode, ROOT_PATH,
};

/// 封装 Home 页面所需的所有信号、派生数据与操作方法。
#[derive(Clone)]
pub struct HomeLogic {
    pub selected_index: RwSignal<Option<usize>>,
    pub detail_items: RwSignal<Vec<DetailItem>>,
    pub detail_loading: RwSignal<bool>,
    pub detail_error: RwSignal<Option<String>>,
    pub detail_path: RwSignal<Option<String>>,

    pub present_nodes: Memo<Vec<UiNode>>,
    pub overview_nodes: Memo<Vec<UiNode>>,
    pub overview_highlight: Memo<Option<String>>,

    pub present_select_callback: UnsyncCallback<usize>,
    pub present_enter_callback: UnsyncCallback<usize>,
    pub overview_select_callback: UnsyncCallback<Option<String>>,
    pub mobile_navigate_callback: UnsyncCallback<Option<String>>,

    pub detail_scroll_ref: NodeRef<leptos::html::Div>,
    pub present_scroll_ref: NodeRef<leptos::html::Div>,

    pub current_path: RwSignal<Option<String>>,
    pub keyboard_enabled: RwSignal<bool>,
}

impl HomeLogic {
    pub fn new() -> Self {
        let path_cache: RwSignal<NodesCache> = RwSignal::new(HashMap::new());
        let assets_cache: RwSignal<AssetsCache> = RwSignal::new(HashMap::new());
        let current_path = RwSignal::new(None::<String>);
        let selected_index = RwSignal::new(None::<usize>);
        let detail_path = RwSignal::new(None::<String>);
        let detail_items = RwSignal::new(Vec::<DetailItem>::new());
        let detail_loading = RwSignal::new(false);
        let detail_error = RwSignal::new(None::<String>);
        let detail_scroll_ref = NodeRef::<leptos::html::Div>::new();
        let present_scroll_ref = NodeRef::<leptos::html::Div>::new();
        let keyboard_enabled = RwSignal::new(true);

        let present_nodes = Memo::new({
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            let current_path = current_path.clone();
            move |_| {
                let key = current_path.get().unwrap_or_else(|| ROOT_PATH.to_string());
                let directories = path_cache
                    .with(|map| map.get(&key).cloned())
                    .unwrap_or_default();
                let assets = if key.is_empty() {
                    Vec::new()
                } else {
                    assets_cache
                        .with(|map| map.get(&key).cloned())
                        .unwrap_or_default()
                };

                let mut nodes = build_ui_nodes(&directories, &assets);
                let overview_label = "Overview".to_string();
                let overview_node = UiNode {
                    id: format!("overview:{}", key),
                    label: overview_label,
                    kind: NodeKind::Overview,
                    directory_path: if key.is_empty() {
                        None
                    } else {
                        Some(key.clone())
                    },
                    raw_path: None,
                    has_children: false,
                };
                let mut combined = Vec::with_capacity(nodes.len() + 1);
                combined.push(overview_node);
                combined.append(&mut nodes);
                log_nodes("present_nodes", &key, &combined);
                combined
            }
        });

        let overview_nodes = Memo::new({
            let path_cache = path_cache.clone();
            let current_path = current_path.clone();
            move |_| {
                let cache = path_cache.get();
                match current_path.get() {
                    Some(path) => {
                        let parent = parent_path(&path).unwrap_or_else(|| ROOT_PATH.to_string());
                        let directories = cache.get(&parent).cloned().unwrap_or_default();
                        let snapshot = build_ui_nodes(&directories, &[] as &[AssetNode]);
                        log_nodes("overview_nodes", &parent, &snapshot);
                        snapshot
                    }
                    None => vec![UiNode {
                        id: ROOT_PATH.to_string(),
                        label: "/".to_string(),
                        kind: NodeKind::Directory,
                        directory_path: Some(ROOT_PATH.to_string()),
                        raw_path: Some("/".to_string()),
                        has_children: true,
                    }],
                }
            }
        });

        let overview_highlight = Memo::new({
            let current_path = current_path.clone();
            move |_| Some(current_path.get().unwrap_or_else(|| ROOT_PATH.to_string()))
        });

        // occupy placeholder for select_index closure, defined later
        let select_index_inner = Rc::new({
            let selected_index = selected_index.clone();
            let present_nodes = present_nodes.clone();
            let present_scroll_ref = present_scroll_ref.clone();
            move |idx: usize| {
                let len = present_nodes.get_untracked().len();
                if len == 0 {
                    selected_index.set(None);
                    scroll_selected_into_view(&present_scroll_ref, None);
                } else if idx < len {
                    selected_index.set(Some(idx));
                    scroll_selected_into_view(&present_scroll_ref, Some(idx));
                }
            }
        });

        // 主导航函数
        let navigate_to = Rc::new({
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            let current_path = current_path.clone();
            let selected_index = selected_index.clone();
            let detail_path = detail_path.clone();
            let present_nodes = present_nodes.clone();
            let present_scroll_ref = present_scroll_ref.clone();
            move |target: Option<String>, preferred_index: Option<usize>| {
                let path_cache = path_cache.clone();
                let assets_cache = assets_cache.clone();
                let current_path = current_path.clone();
                let selected_index = selected_index.clone();
                let detail_path = detail_path.clone();
                let present_nodes = present_nodes.clone();
                let present_scroll_ref = present_scroll_ref.clone();
                spawn_local(async move {
                    log_target("[导航] 请求", target.as_deref());
                    if let Err(e) =
                        ensure_path_and_ancestors(target.as_ref(), path_cache.clone()).await
                    {
                        web_sys::console::log_2(&"[导航] 加载失败".into(), &JsValue::from_str(&e));
                        return;
                    }

                    let cache_key = target
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| ROOT_PATH.to_string());

                    if let Some(ref path) = target {
                        if let Err(e) = ensure_assets(path, assets_cache.clone()).await {
                            web_sys::console::log_2(
                                &"[导航] 资源加载失败".into(),
                                &JsValue::from_str(&e),
                            );
                        }
                    }

                    current_path.set(target.clone());

                    let nodes = present_nodes.get_untracked();
                    if nodes.is_empty() {
                        selected_index.set(None);
                        detail_path.set(None);
                        scroll_selected_into_view(&present_scroll_ref, None);
                        return;
                    }

                    let normalized_idx = preferred_index
                        .and_then(|idx| if idx < nodes.len() { Some(idx) } else { None })
                        .or(Some(0));

                    selected_index.set(normalized_idx);
                    scroll_selected_into_view(&present_scroll_ref, normalized_idx);

                    let detail_target =
                        normalized_idx
                            .and_then(|idx| nodes.get(idx))
                            .and_then(|node| {
                                if matches!(node.kind, NodeKind::Directory)
                                    && node.directory_path.is_some()
                                {
                                    node.directory_path.clone()
                                } else {
                                    None
                                }
                            });
                    detail_path.set(detail_target);
                    log_nodes("navigate_to.nodes", cache_key.as_str(), &nodes);
                });
            }
        });

        let move_selection = Rc::new({
            let selected_index = selected_index.clone();
            let present_nodes = present_nodes.clone();
            let present_scroll_ref = present_scroll_ref.clone();
            move |delta: i32| {
                let len = present_nodes.get_untracked().len() as i32;
                if len == 0 {
                    selected_index.set(None);
                    scroll_selected_into_view(&present_scroll_ref, None);
                    return;
                }

                let current = selected_index.get_untracked().unwrap_or(0) as i32;
                let next = (current + delta).clamp(0, len - 1);
                if current != next {
                    selected_index.set(Some(next as usize));
                    scroll_selected_into_view(&present_scroll_ref, Some(next as usize));
                }
            }
        });

        let enter_selection = Rc::new({
            let present_nodes = present_nodes.clone();
            let selected_index = selected_index.clone();
            let navigate_to = navigate_to.clone();
            move || {
                if let Some(idx) = selected_index.get_untracked() {
                    if let Some(node) = present_nodes.get_untracked().get(idx) {
                        if matches!(node.kind, NodeKind::Directory) && node.directory_path.is_some()
                        {
                            navigate_to(node.directory_path.clone(), None);
                        }
                    }
                }
            }
        });

        let go_back = Rc::new({
            let current_path = current_path.clone();
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            let navigate_to = navigate_to.clone();
            move || {
                let current = current_path.get_untracked();
                let path_cache = path_cache.clone();
                let assets_cache = assets_cache.clone();
                let navigate_to = navigate_to.clone();
                spawn_local(async move {
                    match current {
                        Some(path) if !path.is_empty() => {
                            let parent =
                                parent_path(&path).unwrap_or_else(|| ROOT_PATH.to_string());
                            if let Err(e) = ensure_children(&parent, path_cache.clone()).await {
                                web_sys::console::log_2(
                                    &"[返回上级] 加载父级失败".into(),
                                    &JsValue::from_str(&e),
                                );
                                return;
                            }
                            if !parent.is_empty() {
                                if let Err(e) = ensure_assets(&parent, assets_cache.clone()).await {
                                    web_sys::console::log_2(
                                        &"[返回上级] 加载资源失败".into(),
                                        &JsValue::from_str(&e),
                                    );
                                }
                            }

                            let directories = path_cache
                                .with(|map| map.get(&parent).cloned())
                                .unwrap_or_default();
                            let assets = if parent.is_empty() {
                                Vec::new()
                            } else {
                                assets_cache
                                    .with(|map| map.get(&parent).cloned())
                                    .unwrap_or_default()
                            };
                            let ui_nodes = build_ui_nodes(&directories, &assets);
                            let idx = ui_nodes
                                .iter()
                                .position(|node| {
                                    node.directory_path.as_deref() == Some(path.as_str())
                                })
                                .unwrap_or(0);
                            let target = if parent.is_empty() {
                                None
                            } else {
                                Some(parent)
                            };
                            navigate_to(target, Some(idx + 1));
                        }
                        Some(_) => navigate_to(None, None),
                        None => {}
                    }
                });
            }
        });

        // Watch selected index -> detail path
        {
            let present_nodes = present_nodes.clone();
            let selected_index_signal = selected_index.clone();
            let detail_path_signal = detail_path.clone();
            let detail_items_signal = detail_items.clone();
            let detail_loading_signal = detail_loading.clone();
            let detail_error_signal = detail_error.clone();
            let present_scroll_ref = present_scroll_ref.clone();
            Effect::new(move |_| {
                let nodes = present_nodes.get();
                let len = nodes.len();
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

                scroll_selected_into_view(&present_scroll_ref, normalized_idx);

                match normalized_idx.and_then(|idx| nodes.get(idx)) {
                    None => {
                        detail_loading_signal.set(false);
                        detail_error_signal.set(None);
                        detail_items_signal.set(Vec::new());
                        detail_path_signal.set(None);
                    }
                    Some(node) => match node.kind {
                        NodeKind::Overview => {
                            detail_error_signal.set(None);
                            let overview_items = build_detail_items_from_nodes(&nodes[1..]);
                            let markdown_indices: Vec<(usize, String)> = overview_items
                                .iter()
                                .enumerate()
                                .filter_map(|(idx, item)| {
                                    if matches!(item.kind, NodeKind::Markdown) {
                                        item.raw_path.clone().map(|path| (idx, path))
                                    } else {
                                        None
                                    }
                                })
                                .collect();

                            if markdown_indices.is_empty() {
                                detail_loading_signal.set(false);
                                detail_items_signal.set(overview_items);
                            } else {
                                detail_loading_signal.set(true);
                                let shared_items = Rc::new(RefCell::new(overview_items));
                                let pending = Rc::new(Cell::new(markdown_indices.len()));

                                detail_items_signal.set(shared_items.borrow().clone());

                                for (idx, path) in markdown_indices {
                                    let shared_items = shared_items.clone();
                                    let pending = pending.clone();
                                    let detail_items_signal = detail_items_signal.clone();
                                    let detail_loading_signal = detail_loading_signal.clone();
                                    let detail_error_signal = detail_error_signal.clone();
                                    spawn_local(async move {
                                        match fetch_text_asset(&path).await {
                                            Ok(markdown) => {
                                                let mut items = shared_items.borrow_mut();
                                                if let Some(item) = items.get_mut(idx) {
                                                    item.content = Some(render_markdown(&markdown));
                                                }
                                                detail_items_signal.set(items.clone());
                                            }
                                            Err(err) => {
                                                detail_error_signal.set(Some(err));
                                            }
                                        }
                                        let remaining = pending.get() - 1;
                                        pending.set(remaining);
                                        if remaining == 0 {
                                            detail_loading_signal.set(false);
                                        }
                                    });
                                }
                            }

                            detail_path_signal.set(None);
                        }
                        NodeKind::Directory => {
                            if let Some(path) = node.directory_path.clone() {
                                detail_loading_signal.set(true);
                                detail_error_signal.set(None);
                                detail_items_signal.set(Vec::new());
                                detail_path_signal.set(Some(path));
                            } else {
                                detail_loading_signal.set(false);
                                detail_error_signal.set(None);
                                detail_items_signal.set(Vec::new());
                                detail_path_signal.set(None);
                            }
                        }
                        NodeKind::Markdown => {
                            detail_loading_signal.set(true);
                            detail_error_signal.set(None);
                            detail_path_signal.set(None);
                            detail_items_signal.set(Vec::new());

                            if let Some(path) = node.raw_path.clone() {
                                let detail_loading_signal = detail_loading_signal.clone();
                                let detail_error_signal = detail_error_signal.clone();
                                let detail_items_signal = detail_items_signal.clone();
                                let item = detail_item_from_ui_node(node);
                                spawn_local(async move {
                                    match fetch_text_asset(&path).await {
                                        Ok(content) => {
                                            let mut rendered = item;
                                            rendered.content = Some(render_markdown(&content));
                                            detail_items_signal.set(vec![rendered]);
                                            detail_loading_signal.set(false);
                                            detail_error_signal.set(None);
                                        }
                                        Err(err) => {
                                            detail_items_signal.set(Vec::new());
                                            detail_loading_signal.set(false);
                                            detail_error_signal.set(Some(err));
                                        }
                                    }
                                });
                            } else {
                                detail_loading_signal.set(false);
                                detail_error_signal.set(Some("无法定位 Markdown 文件".into()));
                            }
                        }
                        NodeKind::Video | NodeKind::Image | NodeKind::Pdf | NodeKind::Other => {
                            detail_loading_signal.set(false);
                            detail_error_signal.set(None);
                            detail_path_signal.set(None);
                            detail_items_signal.set(vec![detail_item_from_ui_node(node)]);
                        }
                    },
                }
            });
        }

        // Detail panel data loader
        {
            let detail_path_signal = detail_path.clone();
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            let detail_items = detail_items.clone();
            let detail_loading = detail_loading.clone();
            let detail_error = detail_error.clone();
            Effect::new(move |_| {
                if let Some(path) = detail_path_signal.get() {
                    detail_loading.set(true);
                    detail_error.set(None);
                    let path_cache = path_cache.clone();
                    let assets_cache = assets_cache.clone();
                    let detail_items = detail_items.clone();
                    let detail_loading = detail_loading.clone();
                    let detail_error = detail_error.clone();
                    spawn_local(async move {
                        let dirs_result = ensure_children(&path, path_cache.clone()).await;
                        let assets_result = ensure_assets(&path, assets_cache.clone()).await;
                        if let Err(e) = dirs_result {
                            detail_error.set(Some(e));
                            detail_items.set(Vec::new());
                        } else if let Err(e) = assets_result {
                            detail_error.set(Some(e));
                            detail_items.set(Vec::new());
                        } else {
                            let directories = path_cache
                                .with(|map| map.get(&path).cloned())
                                .unwrap_or_default();
                            let assets = assets_cache
                                .with(|map| map.get(&path).cloned())
                                .unwrap_or_default();
                            detail_error.set(None);
                            detail_items.set(build_detail_items_for_path(&directories, &assets));
                        }
                        detail_loading.set(false);
                    });
                } else {
                    detail_loading.set(false);
                }
            });
        }

        // 初始载入
        {
            let initialized = Rc::new(Cell::new(false));
            let path_cache = path_cache.clone();
            let current_path = current_path.clone();
            let selected_index = selected_index.clone();
            let detail_path = detail_path.clone();
            let present_nodes = present_nodes.clone();
            let present_scroll_ref = present_scroll_ref.clone();
            Effect::new(move |_| {
                if initialized.get() {
                    return;
                }
                initialized.set(true);

                let path_cache = path_cache.clone();
                let current_path = current_path.clone();
                let selected_index = selected_index.clone();
                let detail_path = detail_path.clone();
                let present_nodes = present_nodes.clone();

                spawn_local(async move {
                    if let Err(e) = ensure_children(ROOT_PATH, path_cache.clone()).await {
                        web_sys::console::log_2(
                            &"[初始化] 根节点加载失败".into(),
                            &JsValue::from_str(&e),
                        );
                        return;
                    }

                    current_path.set(None);
                    let nodes = present_nodes.get_untracked();
                    let default_idx = if nodes.is_empty() { None } else { Some(0) };

                    selected_index.set(default_idx);

                    let detail_target =
                        default_idx.and_then(|idx| nodes.get(idx)).and_then(|node| {
                            if matches!(node.kind, NodeKind::Directory)
                                && node.directory_path.is_some()
                            {
                                node.directory_path.clone()
                            } else {
                                None
                            }
                        });
                    detail_path.set(detail_target);
                    scroll_selected_into_view(&present_scroll_ref, default_idx);
                });
            });
        }

        // 注册键盘事件
        {
            let listener_added = Rc::new(Cell::new(false));
            let listener_added_ref = listener_added.clone();
            let move_selection_cb = move_selection.clone();
            let enter_selection_cb = enter_selection.clone();
            let go_back_cb = go_back.clone();
            let detail_scroll_ref_clone = detail_scroll_ref.clone();
            let keyboard_enabled_signal = keyboard_enabled.clone();
            Effect::new(move |_| {
                if listener_added_ref.get() {
                    return;
                }
                listener_added_ref.set(true);

                let move_selection = move_selection_cb.clone();
                let enter_selection = enter_selection_cb.clone();
                let go_back = go_back_cb.clone();
                let detail_scroll_ref = detail_scroll_ref_clone;
                let present_scroll_ref = present_scroll_ref.clone();

                let keyboard_enabled_inner = keyboard_enabled_signal.clone();
                let handle_global_keydown =
                    Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
                        if !keyboard_enabled_inner.get_untracked() {
                            return;
                        }
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
                            detail_scroll_ref.clone(),
                            present_scroll_ref.clone(),
                        );
                    })
                        as Box<dyn FnMut(web_sys::KeyboardEvent)>);

                if let Some(window) = web_sys::window() {
                    if let Err(err) = window.add_event_listener_with_callback(
                        "keydown",
                        handle_global_keydown.as_ref().unchecked_ref(),
                    ) {
                        web_sys::console::log_2(
                            &"[事件] 注册键盘监听失败".into(),
                            &JsValue::from(err),
                        );
                    }
                }
                handle_global_keydown.forget();
            });
        }

        // 调试：current_path 变化日志
        {
            let current_path = current_path.clone();
            Effect::new(move |_| {
                let path = current_path.get();
                log_target("[路径] 当前路径", path.as_deref());
            });
        }

        let present_select_callback = {
            let select_index = select_index_inner.clone();
            UnsyncCallback::new(move |idx: usize| select_index(idx))
        };

        let present_enter_callback = {
            let present_nodes = present_nodes.clone();
            let navigate_to = navigate_to.clone();
            UnsyncCallback::new(move |idx: usize| {
                if let Some(node) = present_nodes.get_untracked().get(idx) {
                    if matches!(node.kind, NodeKind::Directory) && node.directory_path.is_some() {
                        navigate_to(node.directory_path.clone(), None);
                    }
                }
            })
        };

        let overview_select_callback = {
            let navigate_to = navigate_to.clone();
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            UnsyncCallback::new(move |target: Option<String>| {
                let navigate_to = navigate_to.clone();
                let path_cache = path_cache.clone();
                let assets_cache = assets_cache.clone();

                spawn_local(async move {
                    match target {
                        None => navigate_to(None, None),
                        Some(path) if path.is_empty() => navigate_to(None, None),
                        Some(path) => {
                            let parent =
                                parent_path(&path).unwrap_or_else(|| ROOT_PATH.to_string());
                            if let Err(e) = ensure_children(&parent, path_cache.clone()).await {
                                web_sys::console::log_2(
                                    &"[OverviewColumn] 加载父级失败".into(),
                                    &JsValue::from_str(&e),
                                );
                                return;
                            }
                            if !parent.is_empty() {
                                if let Err(e) = ensure_assets(&parent, assets_cache.clone()).await {
                                    web_sys::console::log_2(
                                        &"[OverviewColumn] 加载资源失败".into(),
                                        &JsValue::from_str(&e),
                                    );
                                }
                            }

                            let directories = path_cache
                                .with(|map| map.get(&parent).cloned())
                                .unwrap_or_default();
                            let assets = if parent.is_empty() {
                                Vec::new()
                            } else {
                                assets_cache
                                    .with(|map| map.get(&parent).cloned())
                                    .unwrap_or_default()
                            };
                            let ui_nodes = build_ui_nodes(&directories, &assets);
                            let idx = ui_nodes
                                .iter()
                                .position(|node| {
                                    node.directory_path.as_deref() == Some(path.as_str())
                                })
                                .unwrap_or(0);
                            let target_layer = if parent.is_empty() {
                                None
                            } else {
                                Some(parent)
                            };
                            navigate_to(target_layer, Some(idx + 1));
                        }
                    }
                });
            })
        };

        let mobile_navigate_callback = {
            let navigate_to = navigate_to.clone();
            UnsyncCallback::new(move |target: Option<String>| {
                navigate_to(target.clone(), None);
            })
        };

        HomeLogic {
            selected_index,
            detail_items,
            detail_loading,
            detail_error,
            detail_path,
            present_nodes,
            overview_nodes,
            overview_highlight,
            present_select_callback,
            present_enter_callback,
            overview_select_callback,
            mobile_navigate_callback,
            detail_scroll_ref,
            present_scroll_ref,
            current_path,
            keyboard_enabled,
        }
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

async fn ensure_assets(path: &str, cache: RwSignal<AssetsCache>) -> Result<(), String> {
    if path.is_empty() || cache.with(|map| map.contains_key(path)) {
        return Ok(());
    }

    let data = get_node_assets(path).await?;

    cache.update(|map| {
        map.insert(path.to_string(), data);
    });

    Ok(())
}

async fn ensure_path_and_ancestors(
    path: Option<&String>,
    cache: RwSignal<NodesCache>,
) -> Result<(), String> {
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

fn build_ui_nodes(directories: &[DirectoryNode], assets: &[AssetNode]) -> Vec<UiNode> {
    let mut nodes: Vec<UiNode> = directories
        .iter()
        .map(|dir| UiNode {
            id: dir.path.clone(),
            label: dir.raw_filename.clone(),
            kind: NodeKind::Directory,
            directory_path: Some(dir.path.clone()),
            raw_path: Some(dir.path.clone()),
            has_children: dir.has_subnodes,
        })
        .collect();

    nodes.extend(assets.iter().map(|asset| UiNode {
        id: asset.file_path.clone(),
        label: asset.raw_filename.clone(),
        kind: classify_asset_kind(&asset.raw_filename),
        directory_path: None,
        raw_path: Some(asset.raw_path.clone()),
        has_children: false,
    }));

    nodes.sort_by_key(|node| node.label.to_ascii_lowercase());
    nodes
}

fn build_detail_items_for_path(
    directories: &[DirectoryNode],
    assets: &[AssetNode],
) -> Vec<DetailItem> {
    let mut dir_items: Vec<DetailItem> = directories
        .iter()
        .map(|dir| DetailItem {
            id: dir.path.clone(),
            label: dir.raw_filename.clone(),
            kind: NodeKind::Directory,
            directory_path: Some(dir.path.clone()),
            raw_path: None,
            has_children: dir.has_subnodes,
            content: None,
            display_as_entry: true,
        })
        .collect();
    dir_items.sort_by_key(|item| item.label.to_ascii_lowercase());

    let mut asset_items: Vec<DetailItem> = assets
        .iter()
        .map(|asset| DetailItem {
            id: asset.file_path.clone(),
            label: asset.raw_filename.clone(),
            kind: classify_asset_kind(&asset.raw_filename),
            directory_path: None,
            raw_path: Some(asset.raw_path.clone()),
            has_children: false,
            content: None,
            display_as_entry: false,
        })
        .collect();
    asset_items.sort_by_key(|item| item.label.to_ascii_lowercase());

    dir_items.extend(asset_items);
    dir_items
}

fn build_detail_items_from_nodes(nodes: &[UiNode]) -> Vec<DetailItem> {
    let mut dir_items = Vec::new();
    let mut asset_items = Vec::new();

    for node in nodes
        .iter()
        .filter(|node| !matches!(node.kind, NodeKind::Overview))
    {
        let item = detail_item_from_ui_node(node);
        if matches!(item.kind, NodeKind::Directory) {
            dir_items.push(item);
        } else {
            asset_items.push(item);
        }
    }

    dir_items.sort_by_key(|item| item.label.to_ascii_lowercase());
    asset_items.sort_by_key(|item| item.label.to_ascii_lowercase());

    dir_items.extend(asset_items);
    dir_items
}

fn detail_item_from_ui_node(node: &UiNode) -> DetailItem {
    DetailItem {
        id: node.id.clone(),
        label: node.label.clone(),
        kind: node.kind.clone(),
        directory_path: node.directory_path.clone(),
        raw_path: node.raw_path.clone(),
        has_children: node.has_children,
        content: None,
        display_as_entry: matches!(node.kind, NodeKind::Directory),
    }
}

fn classify_asset_kind(filename: &str) -> NodeKind {
    let ext = filename.rsplit('.').next().map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("md") | Some("markdown") => NodeKind::Markdown,
        Some("mp4") | Some("mov") | Some("webm") | Some("m4v") | Some("ogg") => NodeKind::Video,
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp") | Some("svg")
        | Some("webp") | Some("ico") => NodeKind::Image,
        Some("pdf") => NodeKind::Pdf,
        _ => NodeKind::Other,
    }
}

fn scroll_selected_into_view(container_ref: &NodeRef<leptos::html::Div>, index: Option<usize>) {
    if let Some(idx) = index {
        if let Some(container) = container_ref.get() {
            if let Some(element) = container.dyn_ref::<web_sys::Element>().and_then(|el| {
                el.query_selector(&format!(r#"[data-index="{}"]"#, idx))
                    .ok()
                    .flatten()
            }) {
                element.scroll_into_view_with_bool(false);
            }
        }
    }
}

async fn fetch_text_asset(path: &str) -> Result<String, String> {
    let url = asset_to_url(path);
    Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

fn asset_to_url(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    if normalized.starts_with("http://") || normalized.starts_with("https://") {
        normalized
    } else {
        let trimmed = normalized.trim_start_matches('/');
        let origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "".to_string());
        let base = origin.trim_end_matches('/');
        format!("{}/resource/{}", base, trimmed)
    }
}

fn render_markdown(raw: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(raw, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

fn log_nodes(label: &str, path: &str, nodes: &[UiNode]) {
    if let Ok(serialized) = serde_json::to_string(nodes) {
        web_sys::console::log_3(
            &JsValue::from_str("[节点]"),
            &JsValue::from_str(label),
            &JsValue::from_str(&format!("path={path}, nodes={serialized}")),
        );
    } else {
        web_sys::console::log_3(
            &JsValue::from_str("[节点]"),
            &JsValue::from_str(label),
            &JsValue::from_str(&format!("path={path}, nodes={:?}", nodes)),
        );
    }
}

fn log_target(label: &str, target: Option<&str>) {
    web_sys::console::log_2(
        &JsValue::from_str(label),
        &JsValue::from_str(target.unwrap_or("<root>")),
    );
}
