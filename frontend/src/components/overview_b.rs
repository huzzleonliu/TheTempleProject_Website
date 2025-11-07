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
pub fn OverviewB(
    selected_path: WriteSignal<Option<String>>,
    first_directory: WriteSignal<Option<String>>,
) -> impl IntoView {
    let (directories, set_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // 组件挂载时获取一级目录
    spawn_local(async move {
        set_loading.set(true);
        set_error.set(None);

        match Request::get("/api/directories/root")
            .send()
            .await
        {
            Ok(resp) => {
                match resp.json::<DirectoriesResponse>().await {
                    Ok(data) => {
                        set_directories.set(data.directories.clone());
                        // 设置第一个目录为默认选中
                        if let Some(first_dir) = data.directories.first() {
                            first_directory.set(Some(first_dir.path.clone()));
                            selected_path.set(Some(first_dir.path.clone()));
                        }
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

    view! {
        <ul class="text-2xl text-gray-500">
            <Show
                when=move || loading.get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || error.get().is_some()
                            fallback=move || {
                                view! {
                                    <For
                                        each=move || directories.get()
                                        key=|dir| dir.path.clone()
                                        children=move |dir: DirectoryNode| {
                                            let path = dir.path.clone();
                                            let display_name = path.split('.').last().unwrap_or(&path).to_string();
                                            view! {
                                                <li>
                                                    <button
                                                        class="w-full h-full text-left hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400"
                                                        on:click=move |_| {
                                                            selected_path.set(Some(path.clone()));
                                                        }
                                                    >
                                                        {display_name}
                                                    </button>
                                                </li>
                                            }
                                        }
                                    />
                                }
                            }
                        >
                            <li class="text-red-500">{move || error.get().unwrap_or_else(|| "未知错误".to_string())}</li>
                        </Show>
                    }
                }
            >
                <li>"加载中..."</li>
            </Show>
        </ul>
    }
}
