use std::cell::Cell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

use leptos::callback::UnsyncCallback;
use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

use crate::utils::api::{get_child_directories, get_node_assets, get_root_directories};
use crate::utils::keyboard;
use crate::utils::types::{
    parent_path, split_levels, AssetNode, AssetsCache, DirectoryNode, NodeKind, NodesCache, UiNode,
    ROOT_PATH,
};

/// Home 的核心状态：只有这两个是真正“状态”，其余都应由它们派生。
#[derive(Clone)]
pub struct HomeState {
    pub current_path: RwSignal<Option<String>>,
    pub selected_index: RwSignal<Option<usize>>,
}

/// Home 的派生数据：仅由 HomeState（以及内部 cache/resource）推导出来，UI 只读消费。
#[derive(Clone)]
pub struct HomeDerived {
    pub present_nodes: Memo<Vec<UiNode>>,
    pub overview_nodes: Memo<Vec<UiNode>>,
    pub detail_nodes: Memo<Vec<UiNode>>,
    /// detail 是否对应当前 present 目录（此时 detail 中的资源逐条 viewer 渲染）
    pub is_present_dir_detail: Memo<bool>,
}

/// Home 的交互入口：UI 事件只调用这些回调，不直接读写内部实现细节。
#[derive(Clone)]
pub struct HomeActions {
    pub present_select: UnsyncCallback<usize>,
    pub present_enter: UnsyncCallback<usize>,
    pub overview_select: UnsyncCallback<Option<String>>,
    pub mobile_navigate: UnsyncCallback<Option<String>>,

    /// Detail 的“进入/导航”回调（语义上就是 enter；旧名 detail_click_callback）
    pub detail_enter: Arc<dyn Fn(UiNode) + Send + Sync>,
}

/// Home 的 DOM refs：仅用于滚动/定位，不参与业务状态。
#[derive(Clone)]
pub struct HomeRefs {
    pub detail_scroll: NodeRef<leptos::html::Div>,
    pub present_scroll: NodeRef<leptos::html::Div>,
}

/// Home 的 UI/行为开关。
#[derive(Clone)]
pub struct HomeFlags {
    pub keyboard_enabled: RwSignal<bool>,
}

/// 封装 Home 页面所需的所有信号、派生数据与操作方法（分组后更清晰）。
#[derive(Clone)]
pub struct HomeLogic {
    pub state: HomeState,
    pub derived: HomeDerived,
    pub actions: HomeActions,
    pub refs: HomeRefs,
    pub flags: HomeFlags,
}

impl HomeLogic {
    pub fn new() -> Self {
        let path_cache: RwSignal<NodesCache> = RwSignal::new(HashMap::new());
        let assets_cache: RwSignal<AssetsCache> = RwSignal::new(HashMap::new());
        let current_path = RwSignal::new(None::<String>);
        let selected_index = RwSignal::new(None::<usize>);
        let detail_scroll_ref = NodeRef::<leptos::html::Div>::new();
        let present_scroll_ref = NodeRef::<leptos::html::Div>::new();
        let keyboard_enabled = RwSignal::new(true);
        let pending_detail_click = RwSignal::new(None::<UiNode>);

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

        // detail_nodes 对应的目录上下文：用于判断“是否属于当前 present 目录”
        let detail_context_path = Memo::new({
            let present_nodes = present_nodes.clone();
            let selected_index = selected_index.clone();
            let current_path = current_path.clone();
            move |_| {
                let nodes = present_nodes.get();
                if nodes.is_empty() {
                    return current_path.get();
                }
                let idx = selected_index
                    .get()
                    .unwrap_or(0)
                    .min(nodes.len().saturating_sub(1));
                let Some(focus) = nodes.get(idx) else { return current_path.get(); };
                match focus.kind {
                    NodeKind::Directory => focus.directory_path.clone(),
                    NodeKind::Overview => current_path.get(),
                    _ => current_path.get(),
                }
            }
        });

        let is_present_dir_detail = Memo::new({
            let current_path = current_path.clone();
            let detail_context_path = detail_context_path.clone();
            move |_| {
                let current = current_path.get().unwrap_or_default();
                let ctx = detail_context_path.get().unwrap_or_default();
                current == ctx
            }
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
            let present_nodes = present_nodes.clone();
            let present_scroll_ref = present_scroll_ref.clone();
            move |target: Option<String>, preferred_index: Option<usize>| {
                let path_cache = path_cache.clone();
                let assets_cache = assets_cache.clone();
                let current_path = current_path.clone();
                let selected_index = selected_index.clone();
                let present_nodes = present_nodes.clone();
                let present_scroll_ref = present_scroll_ref.clone();
                spawn_local(async move {
                    if let Err(e) =
                        ensure_path_and_ancestors(target.as_ref(), path_cache.clone()).await
                    {
                        let _ = e;
                        return;
                    }

                    if let Some(ref path) = target {
                        let _ = ensure_assets(path, assets_cache.clone()).await;
                    }

                    current_path.set(target.clone());

                    let nodes = present_nodes.get_untracked();
                    if nodes.is_empty() {
                        selected_index.set(None);
                        scroll_selected_into_view(&present_scroll_ref, None);
                        return;
                    }

                    let normalized_idx = preferred_index
                        .and_then(|idx| if idx < nodes.len() { Some(idx) } else { None })
                        .or(Some(0));

                    selected_index.set(normalized_idx);
                    scroll_selected_into_view(&present_scroll_ref, normalized_idx);
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
                                let _ = e;
                                return;
                            }
                            if !parent.is_empty() {
                                let _ = ensure_assets(&parent, assets_cache.clone()).await;
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

        // Normalize selection index and keep present scroll in sync.
        {
            let present_nodes = present_nodes.clone();
            let selected_index_signal = selected_index.clone();
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
            });
        }

        // Directory entries derived async from the focused Directory node via LocalResource.
        let focused_directory_path = Memo::new({
            let present_nodes = present_nodes.clone();
            let selected_index = selected_index.clone();
            move |_| {
                let nodes = present_nodes.get();
                if nodes.is_empty() {
                    return None::<String>;
                }
                let idx = selected_index
                    .get()
                    .unwrap_or(0)
                    .min(nodes.len().saturating_sub(1));
                let Some(node) = nodes.get(idx) else { return None; };
                if matches!(node.kind, NodeKind::Directory) {
                    node.directory_path.clone()
                } else {
                    None
                }
            }
        });

        let dir_entries_res: LocalResource<Result<Vec<UiNode>, String>> = {
            let focused_directory_path = focused_directory_path.clone();
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            LocalResource::new(move || {
                let maybe_path = focused_directory_path.get();
                let path_cache = path_cache.clone();
                let assets_cache = assets_cache.clone();
                async move {
                    let Some(path) = maybe_path else { return Ok(Vec::new()); };
                    ensure_children(&path, path_cache.clone()).await?;
                    if !path.is_empty() {
                        let _ = ensure_assets(&path, assets_cache.clone()).await;
                    }
                    let directories = path_cache.with(|map| map.get(&path).cloned()).unwrap_or_default();
                    let assets = if path.is_empty() {
                        Vec::new()
                    } else {
                        assets_cache.with(|map| map.get(&path).cloned()).unwrap_or_default()
                    };
                    Ok(build_ui_nodes(&directories, &assets))
                }
            })
        };

        // 初始载入
        {
            let initialized = Rc::new(Cell::new(false));
            let path_cache = path_cache.clone();
            let current_path = current_path.clone();
            let selected_index = selected_index.clone();
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
                let present_nodes = present_nodes.clone();

                spawn_local(async move {
                    if let Err(e) = ensure_children(ROOT_PATH, path_cache.clone()).await {
                        let _ = e;
                        return;
                    }

                    current_path.set(None);
                    let nodes = present_nodes.get_untracked();
                    let default_idx = if nodes.is_empty() { None } else { Some(0) };

                    selected_index.set(default_idx);
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

                        keyboard::handle_keyboard_navigation(
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
                        let _ = err;
                    }
                }
                handle_global_keydown.forget();
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
                                let _ = e;
                                return;
                            }
                            if !parent.is_empty() {
                                let _ = ensure_assets(&parent, assets_cache.clone()).await;
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

        // Detail 点击回调（Send+Sync）：只负责把点击事件写入信号，实际导航逻辑由 HomeLogic 内部 effect 执行。
        let detail_enter_arc: Arc<dyn Fn(UiNode) + Send + Sync> = {
            let pending = pending_detail_click.clone();
            Arc::new(move |node: UiNode| {
                pending.set(Some(node));
            })
        };

        // 处理 detail 点击（在 HomeLogic 内部执行，可捕获非 Send 的 Rc 闭包等）
        {
            let pending = pending_detail_click.clone();
            let navigate_to = navigate_to.clone();
            let present_nodes = present_nodes.clone();
            let selected_index = selected_index.clone();
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            Effect::new(move |_| {
                let Some(item) = pending.get() else { return; };
                pending.set(None);

                let focused = selected_index
                    .get_untracked()
                    .and_then(|idx| present_nodes.get_untracked().get(idx).cloned());
                let focused_kind = focused.as_ref().map(|n| n.kind.clone());

                match focused_kind {
                    Some(NodeKind::Overview) => {
                        // 场景 2（entry-only）：光标在 Overview，点击任意条目 X；
                        // 只在当前层把 present 的选中切到 X（detail 进入 viewer/listing）。
                        let nodes = present_nodes.get_untracked();
                        if let Some(idx) = nodes.iter().position(|n| n.id == item.id) {
                            selected_index.set(Some(idx));
                        }
                    }
                    Some(NodeKind::Directory) => {
                        // 场景 1（entry-only）：光标在目录 A，点击 A 的子条目 X；
                        // 进入 A，并将 X 设为选中（idx=X+1，因为 present[0] 是 Overview）。
                        let focused_dir = focused
                            .as_ref()
                            .and_then(|n| n.directory_path.clone())
                            .filter(|p| !p.is_empty());
                        let Some(a_path) = focused_dir else { return; };

                        let path_cache = path_cache.clone();
                        let assets_cache = assets_cache.clone();
                        let navigate_to = navigate_to.clone();
                        let clicked_id = item.id.clone();

                        spawn_local(async move {
                            if let Err(e) = ensure_children(&a_path, path_cache.clone()).await {
                                let _ = e;
                                return;
                            }
                            if let Err(e) = ensure_assets(&a_path, assets_cache.clone()).await {
                                let _ = e;
                            }

                            let directories = path_cache
                                .with(|map| map.get(&a_path).cloned())
                                .unwrap_or_default();
                            let assets = assets_cache
                                .with(|map| map.get(&a_path).cloned())
                                .unwrap_or_default();

                            let ui_nodes = build_ui_nodes(&directories, &assets);
                            let idx_in_list = ui_nodes.iter().position(|node| node.id == clicked_id);
                            let preferred = idx_in_list.map(|idx| idx + 1);
                            navigate_to(Some(a_path), preferred);
                        });
                    }
                    _ => {}
                }
            });
        }

        // Detail nodes: UiNode 列表（entry-only + 单文件 viewer 也用 vec![node] 表达）
        let detail_nodes = Memo::new({
            let present_nodes = present_nodes.clone();
            let selected_index = selected_index.clone();
            let dir_entries_res = dir_entries_res;
            move |_| {
                let nodes = present_nodes.get();
                if nodes.is_empty() {
                    return Vec::<UiNode>::new();
                }
                let idx = selected_index
                    .get()
                    .unwrap_or(0)
                    .min(nodes.len().saturating_sub(1));
                let Some(focus) = nodes.get(idx).cloned() else { return Vec::new(); };

                match focus.kind {
                    NodeKind::Overview => nodes.into_iter().skip(1).collect(),
                    NodeKind::Directory => match dir_entries_res.get() {
                        Some(Ok(entries)) => entries,
                        _ => Vec::new(),
                    },
                    _ => vec![focus],
                }
            }
        });

        HomeLogic {
            state: HomeState {
                current_path: current_path.clone(),
                selected_index: selected_index.clone(),
            },
            derived: HomeDerived {
                present_nodes: present_nodes.clone(),
                overview_nodes: overview_nodes.clone(),
                detail_nodes,
                is_present_dir_detail,
            },
            actions: HomeActions {
                present_select: present_select_callback,
                present_enter: present_enter_callback,
                overview_select: overview_select_callback,
                mobile_navigate: mobile_navigate_callback,
                detail_enter: detail_enter_arc,
            },
            refs: HomeRefs {
                detail_scroll: detail_scroll_ref,
                present_scroll: present_scroll_ref,
            },
            flags: HomeFlags {
                keyboard_enabled,
            },
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

// (debug logging helpers removed)
