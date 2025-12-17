use leptos::callback::{Callable, UnsyncCallback};
use leptos::prelude::*;

use crate::{NodeKind, UiNode};
use crate::utils::button_style_builder;
use crate::utils::ButtonIconMatcher;

/// Overview 栏：展示“当前位置的父级层级”列表，帮助用户在层级间快速回退。
#[component]
pub fn OverviewColumn(
    nodes: Memo<Vec<UiNode>>,
    highlighted_path: Memo<Option<String>>,
    #[prop(into)] on_select: UnsyncCallback<Option<String>>,
) -> impl IntoView {
    view! {
        <ul class="h-full overflow-y-auto flex flex-col gap-1 py-1">
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
                        <li class="w-full min-w-0 flex flex-row gap-1">
                            <button
                                class=move || {
                                    format!("flex w-full gap-1 items-center {} ", button_style_builder(&node, false))
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
                                <ButtonIconMatcher kind=node_clone.kind.clone() />
                                <span class="min-w-0 flex-1 truncate">{label}</span>
                                // <div class="text-xs text-gray-600 break-all">{detail.clone()}</div>
                            </button>
                        </li>
                    }
                }
            />
        </ul>
    }
}
