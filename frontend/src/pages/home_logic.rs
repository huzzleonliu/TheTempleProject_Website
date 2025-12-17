use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use leptos::callback::{Callback, UnsyncCallback};
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

#[path = "home_logic/detail.rs"]
mod detail;

#[derive(Clone, Debug, PartialEq, Eq)]
enum DetailKey {
    None,
    Overview,
    Directory(String),
    Markdown(String),
    Image(String),
    Video(String),
    Pdf(String),
    Other(String),
}

/// Detail 的 ViewModel：这是 UI 直接消费的数据结构（VM = ViewModel）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DetailView {
    Empty,
    Overview {
        entries: Vec<UiNode>,
        markdown_html: HashMap<String, String>,
    },
    DirectoryEntries {
        entries: Vec<UiNode>,
    },
    MarkdownDoc {
        path: String,
        html: String,
    },
    Image {
        path: String,
        url: String,
    },
    Video {
        path: String,
        url: String,
    },
    Pdf {
        path: String,
        url: String,
    },
    Other {
        path: String,
        url: String,
    },
}

/// Detail 的 ViewModel：这是 UI 直接消费的数据结构（VM = ViewModel）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetailVm {
    pub loading: bool,
    pub error: Option<String>,
    pub view: DetailView,
}

/// 封装 Home 页面所需的所有信号、派生数据与操作方法。
#[derive(Clone)]
pub struct HomeLogic {
    pub selected_index: RwSignal<Option<usize>>,
    pub detail_vm: Memo<DetailVm>,

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

    pub detail_click_callback: Callback<UiNode>,
}

impl HomeLogic {
    pub fn new() -> Self {
        let path_cache: RwSignal<NodesCache> = RwSignal::new(HashMap::new());
        let assets_cache: RwSignal<AssetsCache> = RwSignal::new(HashMap::new());
        let current_path = RwSignal::new(None::<String>);
        let selected_index = RwSignal::new(None::<usize>);
        let markdown_cache: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
        let markdown_inflight: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
        let detail_error = RwSignal::new(None::<String>);
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

        // Derived DetailKey (scaffolding; will drive Resources)
        let detail_key = Memo::new({
            let present_nodes = present_nodes.clone();
            let selected_index = selected_index.clone();
            move |_| {
                let nodes = present_nodes.get();
                if nodes.is_empty() {
                    return DetailKey::None;
                }

                let idx = selected_index
                    .get()
                    .unwrap_or(0)
                    .min(nodes.len().saturating_sub(1));
                let Some(node) = nodes.get(idx) else { return DetailKey::None; };

                match node.kind {
                    NodeKind::Overview => DetailKey::Overview,
                    NodeKind::Directory => node
                        .directory_path
                        .clone()
                        .map(DetailKey::Directory)
                        .unwrap_or(DetailKey::None),
                    NodeKind::Markdown => node
                        .raw_path
                        .clone()
                        .map(DetailKey::Markdown)
                        .unwrap_or(DetailKey::None),
                    NodeKind::Image => node
                        .raw_path
                        .clone()
                        .map(DetailKey::Image)
                        .unwrap_or(DetailKey::None),
                    NodeKind::Video => node
                        .raw_path
                        .clone()
                        .map(DetailKey::Video)
                        .unwrap_or(DetailKey::None),
                    NodeKind::Pdf => node
                        .raw_path
                        .clone()
                        .map(DetailKey::Pdf)
                        .unwrap_or(DetailKey::None),
                    NodeKind::Other => node
                        .raw_path
                        .clone()
                        .map(DetailKey::Other)
                        .unwrap_or(DetailKey::None),
                }
            }
        });

        // Overview mode: list-first, then progressively fill markdown content into markdown_cache.
        let overview_markdown_paths = Memo::new({
            let detail_key = detail_key.clone();
            let present_nodes = present_nodes.clone();
            move |_| {
                if !matches!(detail_key.get(), DetailKey::Overview) {
                    return Vec::<String>::new();
                }
                present_nodes
                    .get()
                    .into_iter()
                    .skip(1) // skip Overview item
                    .filter_map(|n| {
                        if matches!(n.kind, NodeKind::Markdown) {
                            n.raw_path.clone()
                        } else {
                            None
                        }
                    })
                    .collect()
            }
        });

        // Drive markdown_cache fetching for Overview (dedupe via markdown_inflight).
        {
            let markdown_cache = markdown_cache.clone();
            let markdown_inflight = markdown_inflight.clone();
            let detail_error = detail_error.clone();
            let detail_key = detail_key.clone();
            Effect::new(move |_| {
                if !matches!(detail_key.get(), DetailKey::Overview) {
                    return;
                }

                let paths = overview_markdown_paths.get();
                for path in paths {
                    let already_cached = markdown_cache.with(|c| c.contains_key(&path));
                    let already_inflight = markdown_inflight.with(|s| s.contains(&path));
                    if already_cached || already_inflight {
                        continue;
                    }

                    markdown_inflight.update(|s| {
                        s.insert(path.clone());
                    });

                    let markdown_cache = markdown_cache.clone();
                    let markdown_inflight = markdown_inflight.clone();
                    let detail_error = detail_error.clone();
                    spawn_local(async move {
                        match detail::fetch_text_asset(&path).await {
                            Ok(markdown) => {
                                markdown_cache.update(|cache| {
                                    cache.insert(path.clone(), detail::render_markdown(&markdown));
                                });
                            }
                            Err(err) => {
                                detail_error.set(Some(err));
                            }
                        }
                        markdown_inflight.update(|s| {
                            s.remove(&path);
                        });
                    });
                }
            });
        }

        // Reflect Overview loading state based on cache completeness/inflight state.
        let overview_loading = Memo::new({
            let detail_key = detail_key.clone();
            let markdown_cache = markdown_cache.clone();
            let markdown_inflight = markdown_inflight.clone();
            move |_| {
                if !matches!(detail_key.get(), DetailKey::Overview) {
                    return false;
                }
                let paths = overview_markdown_paths.get();
                let any_missing =
                    markdown_cache.with(|cache| paths.iter().any(|p| !cache.contains_key(p)));
                let any_inflight = markdown_inflight.with(|s| !s.is_empty());
                any_missing || any_inflight
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

        // Directory entries derived async from DetailKey::Directory via LocalResource.
        let dir_entries_res: LocalResource<Result<Vec<UiNode>, String>> = {
            let detail_key = detail_key.clone();
            let path_cache = path_cache.clone();
            let assets_cache = assets_cache.clone();
            LocalResource::new(move || {
                let key = detail_key.get();
                let path_cache = path_cache.clone();
                let assets_cache = assets_cache.clone();
                async move {
                    let DetailKey::Directory(path) = key else { return Ok(Vec::new()); };
                    ensure_children(&path, path_cache.clone()).await?;
                    let _ = ensure_assets(&path, assets_cache.clone()).await;
                    let directories = path_cache
                        .with(|map| map.get(&path).cloned())
                        .unwrap_or_default();
                    let assets = assets_cache
                        .with(|map| map.get(&path).cloned())
                        .unwrap_or_default();
                    Ok(build_ui_nodes(&directories, &assets))
                }
            })
        };

        // Selected markdown rendered HTML derived async from DetailKey::Markdown.
        let md_doc_res: LocalResource<Result<(String, String), String>> = {
            let detail_key = detail_key.clone();
            LocalResource::new(move || {
                let key = detail_key.get();
                async move {
                    let DetailKey::Markdown(path) = key else {
                        return Ok((String::new(), String::new()));
                    };
                    let raw = detail::fetch_text_asset(&path).await?;
                    Ok((path, detail::render_markdown(&raw)))
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

        // Detail 点击回调（需要 Send+Sync）：只负责把点击事件写入信号，实际导航逻辑由 HomeLogic 内部 effect 执行。
        let detail_click_callback = {
            let pending = pending_detail_click.clone();
            Callback::new(move |node: UiNode| {
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

                if !matches!(item.kind, NodeKind::Directory) {
                    return;
                }
                let clicked_path = match item.directory_path.clone() {
                    Some(p) if !p.is_empty() => p,
                    _ => return,
                };

                let focused = selected_index
                    .get_untracked()
                    .and_then(|idx| present_nodes.get_untracked().get(idx).cloned());
                let focused_kind = focused.as_ref().map(|n| n.kind.clone());

                match focused_kind {
                    Some(NodeKind::Overview) => {
                        // 场景 2：光标在 Overview，点击 D；进入 D，光标停留在 Overview（idx=0）
                        navigate_to(Some(clicked_path), Some(0));
                    }
                    Some(NodeKind::Directory) => {
                        // 场景 1：光标在目录 A，点击 A 的子目录 C；进入 A，并将 C 设为选中（idx=C+1）
                        let focused_dir = focused
                            .as_ref()
                            .and_then(|n| n.directory_path.clone())
                            .filter(|p| !p.is_empty());
                        let Some(a_path) = focused_dir else { return; };

                        let path_cache = path_cache.clone();
                        let assets_cache = assets_cache.clone();
                        let navigate_to = navigate_to.clone();

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
                            let idx_in_list = ui_nodes.iter().position(|node| {
                                matches!(node.kind, NodeKind::Directory)
                                    && node.directory_path.as_deref() == Some(clicked_path.as_str())
                            });
                            let preferred = idx_in_list.map(|idx| idx + 1);
                            navigate_to(Some(a_path), preferred);
                        });
                    }
                    _ => {}
                }
            });
        }

        // Detail ViewModel: derived from current focus (DetailKey) and async resources/caches.
        let detail_vm = Memo::new({
            let detail_error = detail_error.clone();
            let detail_key = detail_key.clone();
            let present_nodes = present_nodes.clone();
            let markdown_cache = markdown_cache.clone();
            let dir_entries_res = dir_entries_res;
            let md_doc_res = md_doc_res;
            move |_| {
                let key = detail_key.get();
                let error = detail_error.get();

                match key {
                    DetailKey::None => DetailVm {
                        loading: false,
                        error: None,
                        view: DetailView::Empty,
                    },
                    DetailKey::Overview => {
                        let nodes = present_nodes.get();
                        let slice = nodes.into_iter().skip(1).collect::<Vec<_>>();
                        let entries = detail::build_overview_entries_from_nodes(&slice);
                        let markdown_html = markdown_cache.get();
                        DetailVm {
                            loading: overview_loading.get(),
                            error,
                            view: DetailView::Overview {
                                entries,
                                markdown_html,
                            },
                        }
                    }
                    DetailKey::Directory(_path) => {
                        let maybe = dir_entries_res.get();
                        let loading = maybe.is_none();
                        match maybe {
                            None => DetailVm {
                                loading: true,
                                error: None,
                                view: DetailView::DirectoryEntries { entries: Vec::new() },
                            },
                            Some(Ok(entries)) => DetailVm {
                                loading,
                                error: None,
                                view: DetailView::DirectoryEntries { entries },
                            },
                            Some(Err(e)) => DetailVm {
                                loading: false,
                                error: Some(e),
                                view: DetailView::DirectoryEntries { entries: Vec::new() },
                            },
                        }
                    }
                    DetailKey::Markdown(path) => {
                        let maybe = md_doc_res.get();
                        let loading = maybe.is_none();
                        match maybe {
                            None => DetailVm {
                                loading: true,
                                error: None,
                                view: DetailView::MarkdownDoc { path, html: String::new() },
                            },
                            Some(Ok((p, html))) if p == path => DetailVm {
                                loading,
                                error: None,
                                view: DetailView::MarkdownDoc { path, html },
                            },
                            Some(Ok((_p, _html))) => DetailVm {
                                loading: true,
                                error: None,
                                view: DetailView::MarkdownDoc { path, html: String::new() },
                            },
                            Some(Err(e)) => DetailVm {
                                loading: false,
                                error: Some(e),
                                view: DetailView::MarkdownDoc { path, html: String::new() },
                            },
                        }
                    }
                    DetailKey::Image(path) => {
                        let url = asset_to_url(&path);
                        DetailVm { loading: false, error: None, view: DetailView::Image { path, url } }
                    }
                    DetailKey::Video(path) => {
                        let url = asset_to_url(&path);
                        DetailVm { loading: false, error: None, view: DetailView::Video { path, url } }
                    }
                    DetailKey::Pdf(path) => {
                        let url = asset_to_url(&path);
                        DetailVm { loading: false, error: None, view: DetailView::Pdf { path, url } }
                    }
                    DetailKey::Other(path) => {
                        let url = asset_to_url(&path);
                        DetailVm { loading: false, error: None, view: DetailView::Other { path, url } }
                    }
                }
            }
        });

        HomeLogic {
            selected_index,
            detail_vm,
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
            detail_click_callback,
        }
    }
}

fn asset_to_url(raw_path: &str) -> String {
    let normalized = raw_path.replace('\\', "/");
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
