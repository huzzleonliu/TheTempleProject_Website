use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;

use leptos::callback::UnsyncCallback;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_json;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

use crate::api::{get_child_directories, get_node_assets, get_root_directories};
use crate::components::keyboard_handlers;
use crate::types::{
    parent_path, split_levels, AssetNode, AssetsCache, DirectoryNode, NodeKind, NodesCache, UiNode,
    ROOT_PATH,
};

/// 封装 Home 页面所需的所有信号、派生数据与操作方法。
pub struct HomeLogic {
    pub selected_index: RwSignal<Option<usize>>,
    pub preview_nodes: RwSignal<Vec<DirectoryNode>>,
    pub preview_loading: RwSignal<bool>,
    pub preview_error: RwSignal<Option<String>>,

    pub current_nodes: Memo<Vec<UiNode>>,
    pub overview_a_nodes: Memo<Vec<UiNode>>,
    pub overview_a_highlight: Memo<Option<String>>,

    pub select_index_callback: UnsyncCallback<usize>,
    pub enter_index_callback: UnsyncCallback<usize>,
    pub overview_a_select_callback: UnsyncCallback<Option<String>>,

    pub preview_scroll_ref: NodeRef<leptos::html::Div>,
    pub overview_b_scroll_ref: NodeRef<leptos::html::Div>,
}

impl HomeLogic {
    pub fn new() -> Self {
        let path_cache: RwSignal<NodesCache> = RwSignal::new(HashMap::new());
        let assets_cache: RwSignal<AssetsCache> = RwSignal::new(HashMap::new());
        let current_path = RwSignal::new(None::<String>);
        let selected_index = RwSignal::new(None::<usize>);
        let preview_path = RwSignal::new(None::<String>);
        let preview_nodes = RwSignal::new(Vec::<DirectoryNode>::new());
        let preview_loading = RwSignal::new(false);
        let preview_error = RwSignal::new(None::<String>);
        let preview_scroll_ref = NodeRef::<leptos::html::Div>::new();
        let overview_b_scroll_ref = NodeRef::<leptos::html::Div>::new();

        let current_nodes = Memo::new({
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

                let snapshot = build_ui_nodes(&directories, &assets);
                log_nodes("current_nodes", &key, &snapshot);
                snapshot
            }
        });

        let overview_a_nodes = Memo::new({
            let path_cache = path_cache.clone();
            let current_path = current_path.clone();
            move |_| {
                let cache = path_cache.get();
                match current_path.get() {
                    Some(path) => {
                        let parent = parent_path(&path).unwrap_or_else(|| ROOT_PATH.to_string());
                        let directories = cache.get(&parent).cloned().unwrap_or_default();
                        let snapshot = build_ui_nodes(&directories, &[] as &[AssetNode]);
                        log_nodes("overview_a_nodes", &parent, &snapshot);
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

        let overview_a_highlight = Memo::new({
            let current_path = current_path.clone();
            move |_| Some(current_path.get().unwrap_or_else(|| ROOT_PATH.to_string()))
        });

        // occupy placeholder for select_index closure, defined later
        let select_index_inner = Rc::new({
            let selected_index = selected_index.clone();
            let current_nodes = current_nodes.clone();
            let overview_b_scroll_ref = overview_b_scroll_ref.clone();
            move |idx: usize| {
                let len = current_nodes.get_untracked().len();
                if len == 0 {
                    selected_index.set(None);
                    scroll_selected_into_view(&overview_b_scroll_ref, None);
                } else if idx < len {
                    selected_index.set(Some(idx));
                    scroll_selected_into_view(&overview_b_scroll_ref, Some(idx));
                }
            }
        });

        // 主导航函数
        let navigate_to = Rc::new({
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            let current_path = current_path.clone();
            let selected_index = selected_index.clone();
            let preview_path = preview_path.clone();
            let current_nodes = current_nodes.clone();
            let overview_b_scroll_ref = overview_b_scroll_ref.clone();
            move |target: Option<String>, preferred_index: Option<usize>| {
                let path_cache = path_cache.clone();
                let assets_cache = assets_cache.clone();
                let current_path = current_path.clone();
                let selected_index = selected_index.clone();
                let preview_path = preview_path.clone();
                let current_nodes = current_nodes.clone();
                let overview_b_scroll_ref = overview_b_scroll_ref.clone();
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

                    let nodes = current_nodes.get_untracked();
                    if nodes.is_empty() {
                        selected_index.set(None);
                        preview_path.set(None);
                        scroll_selected_into_view(&overview_b_scroll_ref, None);
                        return;
                    }

                    let normalized_idx = preferred_index
                        .and_then(|idx| if idx < nodes.len() { Some(idx) } else { None })
                        .or_else(|| {
                            nodes
                                .iter()
                                .position(|node| matches!(node.kind, NodeKind::Directory))
                        })
                        .or_else(|| Some(0));

                    selected_index.set(normalized_idx);
                    scroll_selected_into_view(&overview_b_scroll_ref, normalized_idx);

                    let preview = normalized_idx
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
                    preview_path.set(preview);
                    log_nodes("navigate_to.nodes", cache_key.as_str(), &nodes);
                });
            }
        });

        let move_selection = Rc::new({
            let selected_index = selected_index.clone();
            let current_nodes = current_nodes.clone();
            let overview_b_scroll_ref = overview_b_scroll_ref.clone();
            move |delta: i32| {
                let len = current_nodes.get_untracked().len() as i32;
                if len == 0 {
                    selected_index.set(None);
                    scroll_selected_into_view(&overview_b_scroll_ref, None);
                    return;
                }

                let current = selected_index.get_untracked().unwrap_or(0) as i32;
                let next = (current + delta).clamp(0, len - 1);
                if current != next {
                    selected_index.set(Some(next as usize));
                    scroll_selected_into_view(&overview_b_scroll_ref, Some(next as usize));
                }
            }
        });

        let enter_selection = Rc::new({
            let current_nodes = current_nodes.clone();
            let selected_index = selected_index.clone();
            let navigate_to = navigate_to.clone();
            move || {
                if let Some(idx) = selected_index.get_untracked() {
                    if let Some(node) = current_nodes.get_untracked().get(idx) {
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
                            navigate_to(target, Some(idx));
                        }
                        Some(_) => navigate_to(None, None),
                        None => {}
                    }
                });
            }
        });

        // Watch selected index -> preview path
        {
            let current_nodes = current_nodes.clone();
            let selected_index_signal = selected_index.clone();
            let preview_path_signal = preview_path.clone();
            let overview_b_scroll_ref = overview_b_scroll_ref.clone();
            Effect::new(move |_| {
                let nodes = current_nodes.get();
                let len = nodes.len();
                let current_idx = selected_index_signal.get();

                let first_directory = nodes
                    .iter()
                    .enumerate()
                    .find(|(_, node)| matches!(node.kind, NodeKind::Directory))
                    .map(|(idx, _)| idx);

                let normalized_idx = if len == 0 {
                    None
                } else {
                    match current_idx {
                        Some(idx) if idx < len => Some(idx),
                        _ => first_directory.or_else(|| Some(0)),
                    }
                };

                if current_idx != normalized_idx {
                    selected_index_signal.set(normalized_idx);
                }

                scroll_selected_into_view(&overview_b_scroll_ref, normalized_idx);

                let preview = normalized_idx
                    .and_then(|idx| nodes.get(idx))
                    .and_then(|node| {
                        if matches!(node.kind, NodeKind::Directory) && node.directory_path.is_some()
                        {
                            node.directory_path.clone()
                        } else {
                            None
                        }
                    });
                preview_path_signal.set(preview);
            });
        }

        // Preview panel data loader
        {
            let preview_path_signal = preview_path.clone();
            let path_cache = path_cache.clone();
            let preview_nodes = preview_nodes.clone();
            let preview_loading = preview_loading.clone();
            let preview_error = preview_error.clone();
            Effect::new(move |_| {
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

        // 初始载入
        {
            let initialized = Rc::new(Cell::new(false));
            let path_cache = path_cache.clone();
            let current_path = current_path.clone();
            let selected_index = selected_index.clone();
            let preview_path = preview_path.clone();
            let current_nodes = current_nodes.clone();
            let overview_b_scroll_ref = overview_b_scroll_ref.clone();
            Effect::new(move |_| {
                if initialized.get() {
                    return;
                }
                initialized.set(true);

                let path_cache = path_cache.clone();
                let current_path = current_path.clone();
                let selected_index = selected_index.clone();
                let preview_path = preview_path.clone();
                let current_nodes = current_nodes.clone();

                spawn_local(async move {
                    if let Err(e) = ensure_children(ROOT_PATH, path_cache.clone()).await {
                        web_sys::console::log_2(
                            &"[初始化] 根节点加载失败".into(),
                            &JsValue::from_str(&e),
                        );
                        return;
                    }

                    current_path.set(None);
                    let nodes = current_nodes.get_untracked();
                    let default_idx = nodes
                        .iter()
                        .enumerate()
                        .find(|(_, node)| matches!(node.kind, NodeKind::Directory))
                        .map(|(idx, _)| idx)
                        .or_else(|| if nodes.is_empty() { None } else { Some(0) });

                    selected_index.set(default_idx);

                    let preview = default_idx.and_then(|idx| nodes.get(idx)).and_then(|node| {
                        if matches!(node.kind, NodeKind::Directory) && node.directory_path.is_some()
                        {
                            node.directory_path.clone()
                        } else {
                            None
                        }
                    });
                    preview_path.set(preview);
                    scroll_selected_into_view(&overview_b_scroll_ref, default_idx);
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
            let preview_scroll_ref_clone = preview_scroll_ref.clone();
            Effect::new(move |_| {
                if listener_added_ref.get() {
                    return;
                }
                listener_added_ref.set(true);

                let move_selection = move_selection_cb.clone();
                let enter_selection = enter_selection_cb.clone();
                let go_back = go_back_cb.clone();
                let preview_scroll_ref = preview_scroll_ref_clone;
                let overview_scroll_ref = overview_b_scroll_ref.clone();

                let handle_global_keydown =
                    Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
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
                            preview_scroll_ref.clone(),
                            overview_scroll_ref.clone(),
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

        let select_index_callback = {
            let select_index = select_index_inner.clone();
            UnsyncCallback::new(move |idx: usize| select_index(idx))
        };

        let enter_index_callback = {
            let current_nodes = current_nodes.clone();
            let navigate_to = navigate_to.clone();
            UnsyncCallback::new(move |idx: usize| {
                if let Some(node) = current_nodes.get_untracked().get(idx) {
                    if matches!(node.kind, NodeKind::Directory) && node.directory_path.is_some() {
                        navigate_to(node.directory_path.clone(), None);
                    }
                }
            })
        };

        let overview_a_select_callback = {
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
                                    &"[OverviewA] 加载父级失败".into(),
                                    &JsValue::from_str(&e),
                                );
                                return;
                            }
                            if !parent.is_empty() {
                                if let Err(e) = ensure_assets(&parent, assets_cache.clone()).await {
                                    web_sys::console::log_2(
                                        &"[OverviewA] 加载资源失败".into(),
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
                            navigate_to(target_layer, Some(idx));
                        }
                    }
                });
            })
        };

        HomeLogic {
            selected_index,
            preview_nodes,
            preview_loading,
            preview_error,
            current_nodes,
            overview_a_nodes,
            overview_a_highlight,
            select_index_callback,
            enter_index_callback,
            overview_a_select_callback,
            preview_scroll_ref,
            overview_b_scroll_ref,
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

fn classify_asset_kind(filename: &str) -> NodeKind {
    let ext = filename.rsplit('.').next().map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("md") | Some("markdown") => NodeKind::Markdown,
        Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("bmp") | Some("svg")
        | Some("webp") | Some("ico") => NodeKind::Image,
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
