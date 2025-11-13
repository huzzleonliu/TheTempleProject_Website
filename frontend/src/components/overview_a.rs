use leptos::prelude::*;
use leptos::callback::{Callable, UnsyncCallback};

use crate::DirectoryNode;

/// OverviewA 组件：显示面包屑/父级节点列表
#[component]
pub fn OverviewA(
    nodes: Memo<Vec<DirectoryNode>>,
    highlighted_path: Memo<Option<String>>,
    #[prop(into)]
    on_select: UnsyncCallback<Option<String>>,
) -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500 flex flex-col gap-1">
            <For
                each=move || nodes.get().into_iter()
                key=|node| node.path.clone()
                children=move |node: DirectoryNode| {
                    let path = node.path.clone();
                    let label = node.raw_filename.clone();
                    let highlight_signal = highlighted_path.clone();
                    let class_path = path.clone();
                    let click_path = path.clone();

                    view! {
                        <li class="w-full min-w-0">
                            <button
                                class=move || {
                                    let base = "w-full h-full text-left truncate text-2xl px-2 py-1 rounded";
                                    let is_selected = highlight_signal
                                        .get()
                                        .as_ref()
                                        .map(|selected| selected == &class_path)
                                        .unwrap_or(false);

                                    if is_selected {
                                        format!("{base} text-white bg-gray-800")
                                    } else {
                                        format!("{base} text-gray-400 hover:text-white hover:bg-gray-800 focus-within:bg-gray-700")
                                    }
                                }
                                on:click=move |_| {
                                    if click_path.is_empty() {
                                        on_select.run(None);
                                    } else {
                                        on_select.run(Some(click_path.clone()));
                                    }
                                }
                            >
                                {label}
                                <div class="text-xs text-gray-600 break-all">{path.clone()}</div>
                            </button>
                        </li>
                    }
                }
            />
        </ul>
    }
}
