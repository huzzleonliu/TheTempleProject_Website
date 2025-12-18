use leptos::callback::UnsyncCallback;
use leptos::prelude::*;

use crate::{NodeKind, UiNode};
use crate::utils::button_style_builder;
use crate::utils::ButtonIconMatcher;

/// Overview 栏：展示“当前位置的父级层级”列表，帮助用户在层级间快速回退。
#[component]
pub fn OverviewColumn(
    nodes: Memo<Vec<UiNode>>,
    #[prop(into)] current_path: Signal<Option<String>>,
    #[prop(into)] on_select: UnsyncCallback<Option<String>>,
) -> impl IntoView {
    view! {
        <ul class="h-full overflow-y-auto flex flex-col gap-1 py-1">
            <For
                each=move || nodes.get().into_iter()
                key=|node| node.id.clone()
                children=move |node: UiNode| {
                    let label = node.label.clone();
                    let node_clone = node.clone();
                    let is_selected = Memo::new({
                        let current_path = current_path.clone();
                        let node = node.clone();
                        move |_| {
                            let current = current_path.get().unwrap_or_default();
                            node.directory_path.as_deref() == Some(current.as_str())
                        }
                    });

                    view! {
                        <li class="w-full min-w-0 flex flex-row gap-1">
                            <button
                                class=move || {
                                    format!("flex w-full gap-1 items-center {} ", button_style_builder(&node, is_selected.get()))
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
