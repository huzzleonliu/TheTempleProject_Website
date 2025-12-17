use crate::pages::home::HomeLogic;
use crate::{DetailView, NodeKind, UiNode};
use leptos::callback::Callback;
use leptos::prelude::*;
use std::sync::Arc;

// NOTE: `For` 的 children 闭包要求 `Send`，因此这里的回调类型需要 `Send + Sync`。

// ---------------- Shared Detail Panel ----------------
#[component]
pub fn DetailPanel(
    view: ReadSignal<DetailView>,
    loading: ReadSignal<bool>,
    error: ReadSignal<Option<String>>,
    scroll_container_ref: NodeRef<leptos::html::Div>,
    #[prop(optional_no_strip)] on_node_click: Option<Arc<dyn Fn(UiNode) + Send + Sync>>,
) -> impl IntoView {
    enum RenderState {
        Loading,
        Error(String),
        Empty,
        Content,
    }

    view! {
        <div
            node_ref=scroll_container_ref
            class="h-full overflow-y-auto overflow-x-hidden px-2"
        >
            {move || {
                let state = if loading.get() {
                    RenderState::Loading
                } else if let Some(err_msg) = error.get() {
                    RenderState::Error(err_msg)
                } else {
                    match view.get() {
                        DetailView::Empty => RenderState::Empty,
                        DetailView::DirectoryEntries { ref entries } if entries.is_empty() => RenderState::Empty,
                        DetailView::Overview { ref entries, .. } if entries.is_empty() => RenderState::Empty,
                        _ => RenderState::Content,
                    }
                };

                let rendered: AnyView = match state {
                    RenderState::Loading => {
                        let message = "加载中...".to_string();
                        view! { <div class="text-gray-500 py-4">{message}</div> }.into_any()
                    }
                    RenderState::Error(err_msg) => {
                        view! { <div class="text-red-500">{err_msg}</div> }.into_any()
                    }
                    RenderState::Empty => {
                        let message = "暂无内容".to_string();
                        view! { <div class="text-gray-500">{message}</div> }.into_any()
                    }
                    RenderState::Content => {
                        let on_node_click = on_node_click.clone();
                        match view.get() {
                            DetailView::DirectoryEntries { entries } => {
                                view! {
                                    <div class="space-y-3">
                                        <For
                                            each=move || entries.clone()
                                            key=|node| node.id.clone()
                                            children=move |node: UiNode| {
                                                render_listing_entry(node, on_node_click.clone())
                                            }
                                        />
                                    </div>
                                }
                                .into_any()
                            }
                            DetailView::Overview { entries, markdown_html } => {
                                view! {
                                    <div class="space-y-3">
                                        <For
                                            each=move || entries.clone()
                                            key=|node| node.id.clone()
                                            children=move |node: UiNode| {
                                                render_overview_node(node, markdown_html.clone(), on_node_click.clone())
                                            }
                                        />
                                    </div>
                                }
                                .into_any()
                            }
                            DetailView::MarkdownDoc { path, html } => {
                                let label = path.clone();
                                view! {
                                    <div class="bg-gray-800 text-gray-100 px-3 py-3 space-y-2">
                                        <div class="font-semibold text-lg">{label}</div>
                                        <div class="prose prose-invert max-w-none text-sm leading-6" inner_html=html></div>
                                    </div>
                                }
                                .into_any()
                            }
                            DetailView::Image { path, url } => {
                                view! {
                                    <div class="space-y-1">
                                        <img src=url class="max-w-full shadow" alt=path.clone()/>
                                        <div class="text-xs text-gray-400 break-all">{path}</div>
                                    </div>
                                }
                                .into_any()
                            }
                            DetailView::Video { path, url } => {
                                view! {
                                    <div class="space-y-1">
                                        <video src=url.clone() controls class="w-full shadow">
                                            <track kind="captions"/>
                                        </video>
                                        <div class="text-xs text-gray-400 break-all">{path}</div>
                                    </div>
                                }
                                .into_any()
                            }
                            DetailView::Pdf { path, url } => {
                                let iframe_src = url.clone();
                                view! {
                                    <div class="space-y-2">
                                        <div class="font-semibold text-lg text-gray-100">{path.clone()}</div>
                                        <object data=url type="application/pdf" class="w-full h-[75vh] border border-gray-700 bg-gray-900">
                                            <iframe src=iframe_src class="w-full h-full rounded" title=path.clone()></iframe>
                                        </object>
                                        <div class="text-xs text-gray-500 break-all">{path}</div>
                                    </div>
                                }
                                .into_any()
                            }
                            DetailView::Other { path, url: _ } => {
                                view! {
                                    <div class="bg-gray-900 text-gray-200 px-3 py-2 rounded">
                                        <div class="font-medium text-base">{path.clone()}</div>
                                        <div class="text-xs text-gray-500 break-all">{path}</div>
                                    </div>
                                }
                                .into_any()
                            }
                            DetailView::Empty => view! { <div class="text-gray-500">"暂无内容"</div> }.into_any(),
                        }
                    }
                };

                rendered
            }}
        </div>
    }
}

fn render_listing_entry(
    node: UiNode,
    on_node_click: Option<Arc<dyn Fn(UiNode) + Send + Sync>>,
) -> AnyView {
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

    if let Some(callback) = on_node_click {
        let node_clone = node.clone();
        return view! {
            <div class="w-full min-w-0">
                <button class="w-full text-left" on:click=move |_| {
                    callback(node_clone.clone());
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

fn render_overview_node(
    node: UiNode,
    markdown_html: std::collections::HashMap<String, String>,
    on_node_click: Option<Arc<dyn Fn(UiNode) + Send + Sync>>,
) -> AnyView {
    if matches!(node.kind, NodeKind::Directory) {
        return render_listing_entry(node, on_node_click);
    }

    let label = node.label.clone();
    match node.kind {
        NodeKind::Markdown => {
            let path = node.raw_path.clone().unwrap_or_default();
            let rendered = markdown_html
                .get(&path)
                .cloned()
                .unwrap_or_else(|| "<p class=\"text-sm\">Markdown 渲染中...</p>".into());
            view! {
                <div class="bg-gray-800 text-gray-100 px-3 py-3 space-y-2">
                    <div class="font-semibold text-lg">{label}</div>
                    <div class="prose prose-invert max-w-none text-sm leading-6" inner_html=rendered></div>
                </div>
            }
            .into_any()
        }
        NodeKind::Image => {
            let path = node.raw_path.clone().unwrap_or_default();
            let src = asset_to_url(&path);
            view! {
                <div class="space-y-1">
                    <img src=src class="max-w-full shadow" alt=label.clone()/>
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
                    <div class="font-medium text-base">{label}</div>
                    <div class="text-xs text-gray-500 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
        NodeKind::Overview | NodeKind::Directory => {
            view! { <div class="text-gray-500">"当前概览"</div> }.into_any()
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

// ---------------- Mobile Detail Wrapper ----------------
#[component]
pub fn Detail(logic: HomeLogic, on_node_click: Callback<Option<String>>) -> impl IntoView {
    let detail_vm = logic.detail_vm;
    let (detail_view, set_detail_view) = signal(DetailView::Empty);
    let (detail_loading, set_detail_loading) = signal(false);
    let (detail_error, set_detail_error) = signal(None::<String>);
    Effect::new(move |_| {
        let vm = detail_vm.get();
        set_detail_view.set(vm.view);
        set_detail_loading.set(vm.loading);
        set_detail_error.set(vm.error);
    });
    let detail_scroll_ref = logic.detail_scroll_ref.clone();
    let pane_key = Memo::new({
        let current_path = logic.current_path.clone();
        move |_| current_path.get().unwrap_or_else(|| "root".to_string())
    });

    let detail_callback: Arc<dyn Fn(UiNode) + Send + Sync> = {
        let handler = on_node_click.clone();
        Arc::new(move |node: UiNode| {
            handler.run(node.directory_path.clone());
        })
    };

    view! {
        <div class=move || format!("absolute inset-0 flex flex-col gap-4 p-4 pane-{}", pane_key.get())>
            <div class="border border-gray-800 rounded-xl overflow-hidden flex-1">
                <DetailPanel
                    view=detail_view
                    loading=detail_loading
                    error=detail_error
                    scroll_container_ref=detail_scroll_ref
                    on_node_click=Some(detail_callback.clone())
                />
            </div>
        </div>
    }
}
