use leptos::prelude::*;

use crate::DirectoryNode;

#[component]
pub fn Preview(
    nodes: ReadSignal<Vec<DirectoryNode>>,
    loading: ReadSignal<bool>,
    error: ReadSignal<Option<String>>,
    scroll_container_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
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
                                        when=move || !nodes.get().is_empty()
                                        fallback=move || view! { <div class="text-gray-500">"该目录没有子节点"</div> }
                                    >
                                        <ul class="space-y-1">
                                            <For
                                                each=move || nodes.get()
                                                key=|dir| dir.path.clone()
                                                children=move |dir: DirectoryNode| {
                                                    view! {
                                                        <li class="text-2xl text-gray-400 hover:text-white hover:bg-gray-800 px-2 py-1 rounded transition-colors">
                                                            <div>{dir.raw_filename.clone()}</div>
                                                            <div class="text-xs text-gray-600 break-all">{dir.path.clone()}</div>
                                                        </li>
                                                    }
                                                }
                                            />
                                        </ul>
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
