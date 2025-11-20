use leptos::callback::UnsyncCallback;
use leptos::ev::MouseEvent;
use leptos::prelude::*;
use wasm_bindgen::JsValue;

use crate::{NodeKind, UiNode};

fn log_overview_b_state(nodes: &[UiNode], selected: Option<usize>) {
    let serialized = serde_json::to_string(nodes).unwrap_or_else(|_| "[]".to_string());
    web_sys::console::log_3(
        &JsValue::from_str("[OverviewB]"),
        &JsValue::from_str(&format!("selected={selected:?}")),
        &JsValue::from_str(&serialized),
    );
}

/// OverviewB 组件：展示“当前层级”的所有节点，并提供选中 / 进入的交互。
#[component]
pub fn OverviewB(
    nodes: Memo<Vec<UiNode>>,
    #[prop(optional)] scroll_container_ref: NodeRef<leptos::html::Div>,
    selected_index: ReadSignal<Option<usize>>,
    #[prop(into)] on_select: UnsyncCallback<usize>,
    #[prop(into)] on_enter: UnsyncCallback<usize>,
) -> impl IntoView {
    let container_ref = scroll_container_ref;

    {
        let nodes = nodes.clone();
        let selected_index = selected_index.clone();
        Effect::new(move |_| {
            log_overview_b_state(&nodes.get(), selected_index.get());
        });
    }

    view! {
        <div class="h-full overflow-y-auto pr-1" node_ref=container_ref.clone()>
            <ul class="text-2xl text-gray-500 outline-none space-y-1">
                <For
                    each=move || nodes.get().into_iter().enumerate()
                    key=|(idx, node)| format!("{}:{}", idx, node.id)
                    children=move |(idx, node): (usize, UiNode)| {
                        let is_selected = Memo::new({
                            let selected_index = selected_index.clone();
                            move |_| selected_index.get() == Some(idx)
                        });
                        let label = node.label.clone();
                        let detail = node
                            .raw_path
                            .clone()
                            .or_else(|| node.directory_path.clone())
                            .unwrap_or_default();
                        let is_directory = matches!(node.kind, NodeKind::Directory);
                        let node_clone = node.clone();
                        let idx_attr = idx.to_string();

                        view! {
                            <li class="w-full min-w-0" data-index=idx_attr.clone()>
                                <button
                                    class=move || {
                                        let base = "w-full h-full text-left truncate text-2xl px-2 py-2 rounded";
                                        if is_selected.get() {
                                            format!("{base} text-white bg-gray-800")
                                        } else {
                                            format!("{base} text-gray-400 hover:text-white hover:bg-gray-800 focus-within:bg-gray-700")
                                        }
                                    }
                                    on:click=move |_event: MouseEvent| {
                                        on_select.run(idx);
                                    }
                                    on:dblclick=move |_event: MouseEvent| {
                                        if is_directory {
                                            on_enter.run(idx);
                                        }
                                    }
                                >
                                    {label}
                                    {move || {
                                        if is_directory && node_clone.has_children {
                                            view! { <span class="ml-2 text-xs text-gray-500">"[+]"</span> }.into_view()
                                        } else {
                                            view! { <span class="ml-2 text-xs text-gray-500">""</span> }.into_view()
                                        }
                                    }}
                                    <div class="text-xs text-gray-600 break-all">{detail.clone()}</div>
                                </button>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
    }
}
