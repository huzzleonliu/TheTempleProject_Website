use std::cell::RefCell;
use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::utils::api::{get_child_directories, get_node_assets, get_root_directories};
use crate::utils::types::{
    parent_path, split_levels, AssetNode, AssetsCache, DirectoryNode, NodeKind, NodesCache, UiNode,
    ROOT_PATH,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetailMode {
    /// 当前 present 目录下资源：资源逐条 viewer 渲染（但资源点击不响应），目录仍作为 entry 可 enter
    Resources,
    /// 子目录 listing：所有内容一律渲染成 entry（资源点击不响应，目录可 enter）
    Entry,
}

#[derive(Clone)]
pub struct HomeLogic {
    pub current_path: RwSignal<Option<String>>,
    pub current_selected_index: RwSignal<Option<usize>>,
    pub present_nodes: Memo<Vec<UiNode>>,
    pub overview_nodes: Memo<Vec<UiNode>>,
    pub detail_nodes: Memo<Vec<UiNode>>,
    pub detail_render_mode: Memo<DetailMode>,
}

#[derive(Clone, Copy)]
struct Stores {
    path_cache: RwSignal<NodesCache>,
    assets_cache: RwSignal<AssetsCache>,
}

thread_local! {
    static STORES: RefCell<Option<Stores>> = RefCell::new(None);
}

fn set_stores(stores: Stores) {
    STORES.with(|cell| {
        *cell.borrow_mut() = Some(stores);
    });
}

fn with_stores<T>(f: impl FnOnce(Stores) -> T) -> T {
    STORES.with(|cell| {
        let stores = cell
            .borrow()
            .expect("HomeLogic stores not initialized (call HomeLogic::new first)");
        f(stores)
    })
}

impl HomeLogic {
    pub fn new() -> Self {
        let stores = Stores {
            path_cache: RwSignal::new(HashMap::new()),
            assets_cache: RwSignal::new(HashMap::new()),
        };
        set_stores(stores);

        let current_path = RwSignal::new(None::<String>);
        let current_selected_index = RwSignal::new(None::<usize>);

        // Present: Overview + 当前目录 children+assets
        let present_nodes = Memo::new({
            let current_path = current_path.clone();
            move |_| {
                let key = current_path.get().unwrap_or_else(|| ROOT_PATH.to_string());
                let (directories, assets) = with_stores(|stores| {
                    let dirs = stores
                        .path_cache
                        .with(|map| map.get(&key).cloned())
                        .unwrap_or_default();
                    let assets = if key.is_empty() {
                        Vec::<AssetNode>::new()
                    } else {
                        stores
                            .assets_cache
                            .with(|map| map.get(&key).cloned())
                            .unwrap_or_default()
                    };
                    (dirs, assets)
                });

                let mut nodes = build_ui_nodes(&directories, &assets);
                let overview_node = UiNode {
                    id: format!("overview:{}", key),
                    order: 0,
                    label: "Overview".to_string(),
                    kind: NodeKind::Overview,
                    directory_path: if key.is_empty() { None } else { Some(key.clone()) },
                    raw_path: None,
                    has_children: false,
                };
                let mut combined = Vec::with_capacity(nodes.len() + 1);
                combined.push(overview_node);
                combined.append(&mut nodes);
                combined
            }
        });

        // Overview: 父级层级（目录列表）
        let overview_nodes = Memo::new({
            let current_path = current_path.clone();
            move |_| {
                let parent = match current_path.get() {
                    Some(path) => parent_path(&path).unwrap_or_else(|| ROOT_PATH.to_string()),
                    None => ROOT_PATH.to_string(),
                };
                let directories = with_stores(|stores| {
                    stores
                        .path_cache
                        .with(|map| map.get(&parent).cloned())
                        .unwrap_or_default()
                });
                if current_path.get().is_none() {
                    vec![UiNode {
                        id: ROOT_PATH.to_string(),  
                        order: 0,
                        label: "/".to_string(),
                        kind: NodeKind::Directory,
                        directory_path: Some(ROOT_PATH.to_string()),
                        raw_path: Some("/".to_string()),
                        has_children: true,
                    }]
                } else {
                    build_ui_nodes(&directories, &[] as &[AssetNode])
                }
            }
        });

        // 当前 focus 的目录路径（若 focus 是 Directory）
        let focused_dir_path = Memo::new({
            let present_nodes = present_nodes.clone();
            let current_selected_index = current_selected_index.clone();
            move |_| {
                let nodes = present_nodes.get();
                let idx = current_selected_index.get().unwrap_or(0);
                nodes.get(idx)
                    .and_then(|n| {
                        if matches!(n.kind, NodeKind::Directory) {
                            n.directory_path.clone()
                        } else {
                            None
                        }
                    })
                    .filter(|p| !p.is_empty())
            }
        });

        // 目录 listing 的异步派生（不暴露 loading/error；未 ready 时 detail_nodes 返回空）
        let dir_entries_res: LocalResource<Result<Vec<UiNode>, String>> = {
            let focused_dir_path = focused_dir_path.clone();
            LocalResource::new(move || {
                let maybe_path = focused_dir_path.get();
                async move {
                    let Some(path) = maybe_path else { return Ok(Vec::new()); };
                    ensure_children(&path).await?;
                    ensure_assets(&path).await?;
                    let (directories, assets) = with_stores(|stores| {
                        let dirs = stores
                            .path_cache
                            .with(|map| map.get(&path).cloned())
                            .unwrap_or_default();
                        let assets = stores
                            .assets_cache
                            .with(|map| map.get(&path).cloned())
                            .unwrap_or_default();
                        (dirs, assets)
                    });
                    Ok(build_ui_nodes(&directories, &assets))
                }
            })
        };

        // Detail nodes: 纯 UiNode 列表（Overview/Directory/Resource）
        let detail_nodes = Memo::new({
            let present_nodes = present_nodes.clone();
            let current_selected_index = current_selected_index.clone();
            let dir_entries_res = dir_entries_res;
            move |_| {
                let nodes = present_nodes.get();
                if nodes.is_empty() {
                    return Vec::<UiNode>::new();
                }
                let idx = current_selected_index.get().unwrap_or(0).min(nodes.len() - 1);
                let Some(focus) = nodes.get(idx).cloned() else { return Vec::new(); };
                match focus.kind {
                    NodeKind::Overview => nodes.into_iter().skip(1).collect(),
                    NodeKind::Directory => match dir_entries_res.get() {
                        Some(Ok(list)) => list,
                        _ => Vec::new(),
                    },
                    _ => vec![focus],
                }
            }
        });

        // DetailRenderMode: 最简规则——focus 是 Directory => Entry，否则 Resources
        let detail_render_mode = Memo::new({
            let present_nodes = present_nodes.clone();
            let current_selected_index = current_selected_index.clone();
            move |_| {
                let nodes = present_nodes.get();
                if nodes.is_empty() {
                    return DetailMode::Entry;
                }
                let idx = current_selected_index.get().unwrap_or(0).min(nodes.len() - 1);
                let Some(focus) = nodes.get(idx) else { return DetailMode::Entry; };
                if matches!(focus.kind, NodeKind::Directory) {
                    DetailMode::Entry
                } else {
                    DetailMode::Resources
                }
            }
        });

        // 初始加载：确保 root children
        {
            let current_path = current_path.clone();
            let current_selected_index = current_selected_index.clone();
            spawn_local(async move {
                let _ = ensure_children(ROOT_PATH).await;
                current_path.set(None);
                current_selected_index.set(Some(0));
            });
        }

        HomeLogic {
            current_path,
            current_selected_index,
            present_nodes,
            overview_nodes,
            detail_nodes,
            detail_render_mode,
        }
    }
}

// --------- navigation helpers (pure-ish) ---------

pub fn navigate_to(logic: &HomeLogic, target: Option<String>, preferred_index: Option<usize>) {
    let logic = logic.clone();
    spawn_local(async move {
        if let Err(_e) = ensure_path_and_ancestors(target.as_deref()).await {
            return;
        }
        if let Some(ref path) = target {
            let _ = ensure_assets(path).await;
        }
        logic.current_path.set(target.clone());

        let nodes = logic.present_nodes.get_untracked();
        if nodes.is_empty() {
            logic.current_selected_index.set(None);
            return;
        }
        let idx = preferred_index
            .and_then(|i| if i < nodes.len() { Some(i) } else { None })
            .or(Some(0));
        logic.current_selected_index.set(idx);
    });
}

/// 用于 Overview 点击：进入 `child_path` 的父目录，并把 present 光标定位到 `child_path` 对应的目录项。
///
/// 关键点：present 列表第 0 项固定是 Overview，所以这里会做 `pos + 1`。
pub fn navigate_to_parent_and_focus_child(logic: &HomeLogic, child_path: String) {
    let logic = logic.clone();
    spawn_local(async move {
        let parent = parent_path(&child_path).unwrap_or_else(|| ROOT_PATH.to_string());
        let _ = ensure_children(&parent).await;
        let _ = ensure_assets(&parent).await;

        let (dirs, assets) = with_stores(|stores| {
            let dirs = stores
                .path_cache
                .with(|m| m.get(&parent).cloned())
                .unwrap_or_default();
            let assets = if parent.is_empty() {
                Vec::new()
            } else {
                stores
                    .assets_cache
                    .with(|m| m.get(&parent).cloned())
                    .unwrap_or_default()
            };
            (dirs, assets)
        });
        let ui_nodes = build_ui_nodes(&dirs, &assets);
        let pos = ui_nodes
            .iter()
            .position(|n| n.directory_path.as_deref() == Some(child_path.as_str()))
            .unwrap_or(0);

        let target_layer = if parent.is_empty() { None } else { Some(parent) };
        navigate_to(&logic, target_layer, Some(pos + 1));
    });
}

async fn ensure_children(path: &str) -> Result<(), String> {
    let already = with_stores(|stores| stores.path_cache.with(|map| map.contains_key(path)));
    if already {
        return Ok(());
    }
    let data = if path.is_empty() {
        get_root_directories().await
    } else {
        get_child_directories(path).await
    }?;
    with_stores(|stores| {
        stores.path_cache.update(|map| {
            map.insert(path.to_string(), data);
        });
    });
    Ok(())
}

async fn ensure_assets(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Ok(());
    }
    let already = with_stores(|stores| stores.assets_cache.with(|map| map.contains_key(path)));
    if already {
        return Ok(());
    }
    let data = get_node_assets(path).await?;
    with_stores(|stores| {
        stores.assets_cache.update(|map| {
            map.insert(path.to_string(), data);
        });
    });
    Ok(())
}

async fn ensure_path_and_ancestors(path: Option<&str>) -> Result<(), String> {
    ensure_children(ROOT_PATH).await?;
    if let Some(path) = path {
        for level in split_levels(path) {
            if let Some(parent) = parent_path(&level) {
                ensure_children(&parent).await?;
            }
            ensure_children(&level).await?;
        }
    }
    Ok(())
}

fn build_ui_nodes(directories: &[DirectoryNode], assets: &[AssetNode]) -> Vec<UiNode> {
    let mut nodes: Vec<UiNode> = directories
        .iter()
        .map(|dir| UiNode {
            id: dir.path.clone(),
            order: 0,
            label: dir.raw_filename.clone(),
            kind: NodeKind::Directory,
            directory_path: Some(dir.path.clone()),
            raw_path: Some(dir.path.clone()),
            has_children: dir.has_subnodes,
        })
        .collect();

    nodes.extend(assets.iter().map(|asset| {
        // 如果文件名中包含 _ 且第一个部分是数字，则认为文件已重命名，取第二个部分作为文件名
        // 这是一个临时的改动，因为目前的资源文件名都没有完整地被重命名，期望未来文件名都以“序号_原始文件名.ext”的形式命名
        let mut label = asset.raw_filename.clone();
        let mut order = 1;
        if asset.raw_filename.contains("_") && asset.raw_filename.split("_").nth(0).is_some_and(|s| s.parse::<usize>().is_ok()) {
            label = asset.raw_filename.split("_").nth(1).unwrap().to_string();
            order = asset.raw_filename.split("_").nth(0).unwrap().parse::<usize>().unwrap();
        }
        UiNode {
        id: asset.file_path.clone(),
        label: label,
        order: order,
        kind: classify_asset_kind(&asset.raw_filename),
        directory_path: None,
        raw_path: Some(asset.raw_path.clone()),
        has_children: false,
    }}));
    // 按照序号排序，如果序号相同，则按照文件名排序
    nodes.sort_by_key(|node| (node.order, node.label.to_ascii_lowercase()));
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

