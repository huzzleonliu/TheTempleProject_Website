use leptos::ev::MouseEvent;
use leptos::prelude::*;
use leptos::callback::{Callable, UnsyncCallback};

use crate::DirectoryNode;

/// OverviewB 组件：展示“当前层级”的所有节点，并提供选中 / 进入的交互。
#[component]
pub fn OverviewB(
    nodes: Memo<Vec<DirectoryNode>>,
    selected_index: ReadSignal<Option<usize>>,
    #[prop(into)]
    on_select: UnsyncCallback<usize>,
    #[prop(into)]
    on_enter: UnsyncCallback<usize>,
) -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500 outline-none">
            <For
                each=move || nodes.get().into_iter().enumerate()
                key=|(idx, node)| format!("{}:{}", idx, node.path)
                children=move |(idx, node): (usize, DirectoryNode)| {
                    // 使用 Memo 保持“当前行是否被选中”的状态，便于在键盘导航时高亮
                    let is_selected = Memo::new({
                        let selected_index = selected_index.clone();
                        move |_| selected_index.get() == Some(idx)
                    });
                    let has_subnodes = node.has_subnodes;
                    let path = node.path.clone();
                    let label = node.raw_filename.clone();

                    view! {
                        <li class="w-full min-w-0">
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
                                    // 单击仅改变选中项，不改变当前层级
                                    on_select.run(idx);
                                }
                                on:dblclick=move |_event: MouseEvent| {
                                    if has_subnodes {
                                        // 双击进入下一层级
                                        on_enter.run(idx);
                                    }
                                }
                            >
                                {label}
                                {move || {
                                    if has_subnodes {
                                        view! { <span class="ml-2 text-xs text-gray-500">"[+]"</span> }.into_view()
                                    } else {
                                        view! { <span class="ml-2 text-xs text-gray-500">""</span> }.into_view()
                                    }
                                }}
                                <div class="text-xs text-gray-600 break-all">{path.clone()}</div>
                            </button>
                        </li>
                    }
                }
            />
        </ul>
    }
}
