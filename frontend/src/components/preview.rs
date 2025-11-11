use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::DirectoryNode;
use crate::api::get_child_directories;

// 类型已移动到 crate::types

#[component]
pub fn Preview(
    preview_path: ReadSignal<Option<String>>,
    scroll_container_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    let (directories, set_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // 当 preview_path 改变时，获取子目录
    create_effect(move |_| {
        if let Some(path) = preview_path.get() {
            let path_clone = path.clone();
            spawn_local(async move {
                set_loading.set(true);
                set_error.set(None);

                match get_child_directories(&path_clone).await {
                    Ok(children) => {
                        set_directories.set(children);
                        set_loading.set(false);
                    }
                    Err(e) => {
                        set_error.set(Some(e));
                        set_loading.set(false);
                    }
                }
            });
        } else {
            set_directories.set(Vec::new());
        }
    });

    view! {
        <div 
            node_ref=scroll_container_ref
            class="h-full overflow-y-auto overflow-x-hidden"
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
                                        when=move || !directories.get().is_empty()
                                        fallback=move || {
                                            view! { <div><p>"该目录没有子节点"</p></div> }
                                        }
                                    >
                                        <div>
                                            <For
                                                each=move || directories.get()
                                                key=|dir| dir.path.clone()
                                                children=move |dir: DirectoryNode| {
                                                    // 显示名称：使用 raw_filename（未清洗的目录名）
                                                    let display_name = dir.raw_filename.clone();
                                                    view! {
                                                        <div class="text-2xl text-gray-500 hover:text-white hover:bg-gray-800">
                                                            {display_name}
                                                        </div>
                                                    }
                                                }
                                            />
                                            // <div style="height: 50vh;"></div>
                                        </div>
                                    </Show>
                                }
                            }
                        >
                            <div><p class="text-red-500">{move || error.get().unwrap_or_else(|| "未知错误".to_string())}</p></div>
                        </Show>
                    }
                }
            >
                <div><p>"加载中..."</p></div>
            </Show>
        </div>
    }
}
