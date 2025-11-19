use crate::{NodeKind, PreviewItem};
use leptos::prelude::*;
use wasm_bindgen::JsValue;

#[component]
pub fn Preview(
    items: ReadSignal<Vec<PreviewItem>>,
    loading: ReadSignal<bool>,
    error: ReadSignal<Option<String>>,
    scroll_container_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    {
        let items = items.clone();
        let loading = loading.clone();
        let error = error.clone();
        Effect::new(move |_| {
            let snapshot = items.get();
            let serialized =
                serde_json::to_string(&snapshot).unwrap_or_else(|_| "[]".to_string());
            let loading_state = loading.get();
            let error_state = error.get();
            web_sys::console::log_4(
                &JsValue::from_str("[Preview]"),
                &JsValue::from_str(&format!("loading={loading_state}")),
                &JsValue::from_str(&format!("error={error_state:?}")),
                &JsValue::from_str(&serialized),
            );
        });
    }

    view! {
        <div
            node_ref=scroll_container_ref
            class="h-full overflow-y-auto overflow-x-hidden px-2"
        >
            <Show
                when=move || loading.get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || error.get().is_some()
                            fallback=move || {
                                view! {
                                    <Show
                                        when=move || !items.get().is_empty()
                                        fallback=move || view! { <div class="text-gray-500">"暂无内容"</div> }
                                    >
                                        <div class="space-y-3">
                        <For
                            each=move || items.get()
                                key=|item| format!(
                                    "{}:{}:{}",
                                    item.id,
                                    item.content.is_some() as u8,
                                    item.display_as_entry as u8
                                )
                                                children=move |item: PreviewItem| {
                                                    render_preview_item(item)
                                                }
                                            />
                                        </div>
                                    </Show>
                                }
                            }
                        >
                            <div class="text-red-500">{move || error.get().unwrap_or_else(|| "未知错误".to_string())}</div>
                        </Show>
                    }
                }
            >
                <div class="text-gray-500 py-4">"加载中..."</div>
            </Show>
        </div>
    }
}

fn render_preview_item(item: PreviewItem) -> AnyView {
    let PreviewItem {
        id: _,
        label,
        kind,
        directory_path,
        raw_path,
        has_children,
        content,
        display_as_entry,
    } = item;

    if display_as_entry {
        return render_listing_entry(kind, label, directory_path, raw_path, has_children);
    }

    match kind {
        NodeKind::Directory => {
            let detail = directory_path.unwrap_or_default();
            let indicator = if has_children { "[+]" } else { "" };
            view! {
                <div class="w-full min-w-0">
                    <button class="w-full text-left truncate text-2xl px-2 py-2 rounded text-gray-400 hover:text-white hover:bg-gray-800 focus-within:bg-gray-700 transition-colors">
                        <span>{label.clone()}</span>
                        <span class="ml-2 text-xs text-gray-500">{indicator}</span>
                        <div class="text-xs text-gray-600 break-all mt-1">{detail}</div>
                    </button>
                </div>
            }
            .into_any()
        }
        NodeKind::Markdown => {
            let rendered =
                content.unwrap_or_else(|| "<p class=\"text-sm\">无法加载 Markdown 内容</p>".into());
            view! {
                <div class="bg-gray-800 text-gray-100 px-3 py-3 rounded space-y-2">
                    <div class="font-semibold text-lg">{label.clone()}</div>
                    <div class="prose prose-invert max-w-none text-sm leading-6" inner_html=rendered></div>
                </div>
            }
            .into_any()
        }
        NodeKind::Image => {
            let path = raw_path.unwrap_or_default();
            let src = asset_to_url(&path);
            view! {
                <div class="space-y-1">
                    <img src=src class="max-w-full rounded shadow" alt=label.clone()/>
                    <div class="text-xs text-gray-400 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
        NodeKind::Video => {
            let path = raw_path.unwrap_or_default();
            let src = asset_to_url(&path);
            view! {
                <div class="space-y-1">
                    <video src=src.clone() controls class="w-full rounded shadow">
                        <track kind="captions"/>
                    </video>
                    <div class="text-xs text-gray-400 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
        NodeKind::Overview => view! { <div class="text-gray-500">"当前概览"</div> }.into_any(),
        NodeKind::Other => {
            let path = raw_path.unwrap_or_default();
            view! {
                <div class="bg-gray-900 text-gray-200 px-3 py-2 rounded">
                    <div class="font-medium text-base">{label.clone()}</div>
                    <div class="text-xs text-gray-500 break-all">{path}</div>
                </div>
            }
            .into_any()
        }
    }
}

fn render_listing_entry(
    kind: NodeKind,
    label: String,
    directory_path: Option<String>,
    raw_path: Option<String>,
    has_children: bool,
) -> AnyView {
    let detail = directory_path
        .clone()
        .or(raw_path.clone())
        .unwrap_or_default();

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
        NodeKind::Other => "[文件]",
        NodeKind::Overview => "[Overview]",
    };

    view! {
        <div class="w-full min-w-0">
            <div class="w-full text-left truncate text-2xl px-2 py-2 rounded text-gray-400 bg-gray-900/40 border border-gray-800">
                <div class="flex items-center gap-2">
                    <span class="text-xs text-gray-500">{badge}</span>
                    <span class="text-gray-100">{label.clone()}</span>
                </div>
                <div class="text-xs text-gray-600 break-all mt-1">{detail}</div>
            </div>
        </div>
    }
    .into_any()
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
