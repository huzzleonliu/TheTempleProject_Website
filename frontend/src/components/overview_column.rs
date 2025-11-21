use leptos::callback::{Callable, UnsyncCallback};
use leptos::prelude::*;

use crate::{NodeKind, UiNode};

/// Overview 栏：展示“当前位置的父级层级”列表，帮助用户在层级间快速回退。
#[component]
pub fn OverviewColumn(
    nodes: Memo<Vec<UiNode>>,
    highlighted_path: Memo<Option<String>>,
    #[prop(into)] on_select: UnsyncCallback<Option<String>>,
) -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500 flex flex-col gap-1">
            <For
                each=move || nodes.get().into_iter()
                key=|node| node.id.clone()
                children=move |node: UiNode| {
                    let node_id = node.id.clone();
                    let label = node.label.clone();
                    let detail = node
                        .raw_path
                        .clone()
                        .or_else(|| node.directory_path.clone())
                        .unwrap_or_default();
                    let highlight_signal = highlighted_path.clone();
                    let node_clone = node.clone();

                    view! {
                        <li class="w-full min-w-0">
                            <button
                                class=move || {
                                    let base = "w-full h-full text-left truncate text-2xl px-2 py-1 rounded";
                                    let is_selected = highlight_signal
                                        .get()
                                        .as_ref()
                                        .map(|selected| selected == &node_id)
                                        .unwrap_or(false);

                                    if is_selected {
                                        format!("{base} text-white bg-gray-800")
                                    } else {
                                        format!("{base} text-gray-400 hover:text-white hover:bg-gray-800 focus-within:bg-gray-700")
                                    }
                                }
                                on:click=move |_| {
                                    if matches!(node_clone.kind, NodeKind::Directory) {
                                        match node_clone.directory_path.as_deref() {
                                            Some("") | None => on_select.run(None),
                                            Some(path) => on_select.run(Some(path.to_string())),
                                        }
                                    }
                                }
                            >
                                {label}
                                <div class="text-xs text-gray-600 break-all">{detail.clone()}</div>
                            </button>
                        </li>
                    }
                }
            />
        </ul>
    }
}
