use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DirectoryNode {
    path: String,
    has_layout: bool,
    has_visual_assets: bool,
    has_text: i32,
    has_images: i32,
    has_subnodes: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DirectoriesResponse {
    directories: Vec<DirectoryNode>,
}

#[component]
pub fn Preview(
    selected_path: ReadSignal<Option<String>>,
    first_directory: ReadSignal<Option<String>>,
) -> impl IntoView {
    let (directories, set_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // 当 selected_path 改变时，获取子目录
    create_effect(move |_| {
        // 优先使用选中的路径，如果没有则使用第一个目录
        let path_to_use = selected_path.get().or_else(|| first_directory.get());
        
        if let Some(path) = path_to_use {
            let path_clone = path.clone();
            spawn_local(async move {
                set_loading.set(true);
                set_error.set(None);

                // URL 编码路径
                let encoded_path = urlencoding::encode(&path_clone);
                let url = format!("/api/directories/children/{}", encoded_path);

                match Request::get(&url).send().await {
                    Ok(resp) => {
                        match resp.json::<DirectoriesResponse>().await {
                            Ok(data) => {
                                set_directories.set(data.directories);
                                set_loading.set(false);
                            }
                            Err(e) => {
                                set_error.set(Some(format!("解析错误: {e}")));
                                set_loading.set(false);
                            }
                        }
                    }
                    Err(e) => {
                        set_error.set(Some(format!("请求失败: {e}")));
                        set_loading.set(false);
                    }
                }
            });
        } else {
            set_directories.set(Vec::new());
        }
    });

    view! {
        <div>
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
                                            view! { <div><p>"请选择一个目录"</p></div> }
                                        }
                                    >
                                        <div>
                                            <For
                                                each=move || directories.get()
                                                key=|dir| dir.path.clone()
                                                children=move |dir: DirectoryNode| {
                                                    let display_name = dir.path.split('.').last().unwrap_or(&dir.path).to_string();
                                                    view! {
                                                        <div class="text-2xl text-gray-500 hover:text-white hover:bg-gray-800">
                                                            <span>"· "</span>
                                                            <span>{display_name}</span>
                                                        </div>
                                                    }
                                                }
                                            />
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
