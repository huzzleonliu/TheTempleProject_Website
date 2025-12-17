use gloo_net::http::Request;
use pulldown_cmark::{html, Options, Parser};

use crate::utils::types::{NodeKind, UiNode};

pub(super) fn build_overview_entries_from_nodes(nodes: &[UiNode]) -> Vec<UiNode> {
    let mut dir_nodes = Vec::new();
    let mut asset_nodes = Vec::new();

    for node in nodes
        .iter()
        .filter(|node| !matches!(node.kind, NodeKind::Overview))
    {
        if matches!(node.kind, NodeKind::Directory) {
            dir_nodes.push(node.clone());
        } else {
            asset_nodes.push(node.clone());
        }
    }

    dir_nodes.sort_by_key(|n| n.label.to_ascii_lowercase());
    asset_nodes.sort_by_key(|n| n.label.to_ascii_lowercase());

    dir_nodes.extend(asset_nodes);
    dir_nodes
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


