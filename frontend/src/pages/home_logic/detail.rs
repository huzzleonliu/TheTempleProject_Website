use gloo_net::http::Request;
use pulldown_cmark::{html, Options, Parser};

use crate::utils::types::{AssetNode, DetailItem, DirectoryNode, NodeKind, UiNode};

pub(super) fn build_detail_items_for_path(
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
            kind: super::classify_asset_kind(&asset.raw_filename),
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

pub(super) fn build_detail_items_from_nodes(nodes: &[UiNode]) -> Vec<DetailItem> {
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

pub(super) fn detail_item_from_ui_node(node: &UiNode) -> DetailItem {
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

pub(super) async fn fetch_text_asset(path: &str) -> Result<String, String> {
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

pub(super) fn render_markdown(raw: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(raw, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}


