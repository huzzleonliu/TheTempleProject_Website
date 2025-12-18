use crate::pages::home::logic::DetailMode;
use crate::{NodeKind, UiNode};
use gloo_net::http::Request;
use leptos::callback::UnsyncCallback;
use leptos::prelude::*;
use pulldown_cmark::{html, Options, Parser};

// NOTE: `For` 的 children 闭包要求 `Send`，因此这里的回调类型需要 `Send + Sync`。

// ---------------- Shared Detail Panel ----------------
#[component]
pub fn DetailPanel(
    #[prop(into)] nodes: Signal<Vec<UiNode>>,
    #[prop(into)] detail_render_mode: Signal<DetailMode>,
    #[prop(into)] on_enter: UnsyncCallback<usize>,
    scroll_container_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    view! {
        <div
            node_ref=scroll_container_ref
            class="h-full overflow-y-auto overflow-x-hidden px-2"
        >
            {move || {
                let snapshot = nodes.get();
                if snapshot.is_empty() {
                    return view! { <div class="text-gray-500">"暂无内容"</div> }.into_any();
                }

                let mode = detail_render_mode.get();
                view! {
                    <div class="space-y-3">
                        <For
                            each=move || nodes.get().into_iter().enumerate()
                            key=|(idx, node)| format!("{}:{}", idx, node.id)
                            children=move |(idx, node): (usize, UiNode)| {
                                render_node(idx, node, mode, on_enter.clone())
                            }
                        />
                    </div>
                }
                .into_any()
            }}
        </div>
    }
}

fn render_node(
    idx: usize,
    node: UiNode,
    mode: DetailMode,
    on_enter: UnsyncCallback<usize>,
) -> AnyView {
    // 核心规则（你要的）：
    // - 如果 detail 对应当前 present 目录：资源逐条 viewer 渲染（目录仍是条目）
    // - 如果 detail 对应子目录 listing：资源一律条目渲染（哪怕只有一条）
    let is_resource = !matches!(node.kind, NodeKind::Directory | NodeKind::Overview);
    let render_resource_as_viewer = matches!(mode, DetailMode::Resources) && is_resource;
    if !render_resource_as_viewer {
        return render_entry(idx, node, on_enter);
    }

    // Viewer 模式：单节点时按类型展示（Markdown 有 loading/error；其余按资源类型展示）
    // NOTE: Directory/Overview 理论上不会作为单节点 viewer 出现，但这里仍按 entry 渲染兜底。
    match node.kind {
        NodeKind::Markdown => {
            let Some(path) = node.raw_path.clone() else {
                return view! { <div class="text-red-500 py-4">"无法定位 Markdown 文件"</div> }.into_any();
            };
            let label = node.label.clone();
            view! { <MarkdownViewer label=label path=path /> }.into_any()
        }
        NodeKind::Image => {
            let path = node.raw_path.clone().unwrap_or_default();
            let src = asset_to_url(&path);
            view! {
                <div class="space-y-1">
                    <img src=src class="max-w-full shadow" alt=node.label.clone()/>
                    <div class="text-xs text-gray-400 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
        NodeKind::Video => {
            let path = node.raw_path.clone().unwrap_or_default();
            let src = asset_to_url(&path);
            view! {
                <div class="space-y-1">
                    <video src=src.clone() controls class="w-full shadow">
                        <track kind="captions"/>
                    </video>
                    <div class="text-xs text-gray-400 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
        NodeKind::Pdf => {
            let path = node.raw_path.clone().unwrap_or_default();
            let src = asset_to_url(&path);
            let iframe_src = src.clone();
            let label = node.label.clone();
            view! {
                <div class="space-y-2">
                    <div class="font-semibold text-lg text-gray-100">{label.clone()}</div>
                    <object data=src type="application/pdf" class="w-full h-[75vh] border border-gray-700 bg-gray-900">
                        <iframe src=iframe_src class="w-full h-full rounded" title=label.clone()></iframe>
                    </object>
                    <div class="text-xs text-gray-500 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
        NodeKind::Other => {
            let path = node.raw_path.clone().unwrap_or_default();
            view! {
                <div class="bg-gray-900 text-gray-200 px-3 py-2 rounded">
                    <div class="font-medium text-base">{node.label.clone()}</div>
                    <div class="text-xs text-gray-500 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
        NodeKind::Directory | NodeKind::Overview => render_entry(idx, node, on_enter),
    }
}

#[component]
fn MarkdownViewer(label: String, path: String) -> impl IntoView {
    let md_res: LocalResource<Result<String, String>> = LocalResource::new({
        let path = path.clone();
        move || {
            let path = path.clone();
            async move {
                let raw = fetch_text_asset(&path).await?;
                Ok(render_markdown(&raw))
            }
        }
    });

    view! {
        {move || match md_res.get() {
            None => view! { <div class="text-gray-500 py-4">"加载中..."</div> }.into_any(),
            Some(Err(e)) => view! { <div class="text-red-500 py-4">{e}</div> }.into_any(),
            Some(Ok(html)) => view! {
                <div class="bg-gray-800 text-gray-100 px-3 py-3 space-y-2">
                    <div class="font-semibold text-lg">{label.clone()}</div>
                    <div class="prose prose-invert max-w-none text-sm leading-6" inner_html=html></div>
                </div>
            }.into_any(),
        }}
    }
}

fn render_entry(idx: usize, node: UiNode, on_enter: UnsyncCallback<usize>) -> AnyView {
    let kind = node.kind.clone();
    let label = node.label.clone();
    let directory_path = node.directory_path.clone();
    let raw_path = node.raw_path.clone();
    let has_children = node.has_children;

    let detail = directory_path.clone().or(raw_path.clone()).unwrap_or_default();

    let badge = match kind {
        NodeKind::Directory => {
            if has_children {
                "[目录 • +]"
            } else {
                "[目录]"
            }
        }
        NodeKind::Markdown => "[Markdown]",
        NodeKind::Image => "[图片]",
        NodeKind::Video => "[视频]",
        NodeKind::Pdf => "[PDF]",
        NodeKind::Other => "[文件]",
        NodeKind::Overview => "[Overview]",
    };

    let inner = move || {
        view! {
            <div class="w-full text-left truncate text-2xl px-2 py-2 text-gray-400 bg-gray-900/40 border border-gray-800 ">
                <div class="flex items-center gap-2">
                    <span class="text-xs text-gray-500">{badge}</span>
                    <span class="text-gray-100">{label.clone()}</span>
                </div>
                <div class="text-xs text-gray-600 break-all mt-1">{detail.clone()}</div>
            </div>
        }
    };

    if matches!(kind, NodeKind::Directory) {
        return view! {
            <div class="w-full min-w-0">
                <button class="w-full text-left" on:click=move |_| {
                    on_enter.run(idx);
                }>
                    {inner()}
                </button>
            </div>
        }
        .into_any();
    }

    view! {
        <div class="w-full min-w-0">
            {inner()}
        </div>
    }
    .into_any()
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

fn render_markdown(raw: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_HEADING_ATTRIBUTES);
    let parser = Parser::new_ext(raw, options);

    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
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
