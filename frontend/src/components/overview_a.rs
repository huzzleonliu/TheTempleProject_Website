use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use crate::{DirectoryNode, ItemContext, NavigationSignals, DataSignals};
use crate::api::{get_child_directories, get_root_directories};
use crate::components::item::ItemComponent;

// 类型已移动到 crate::types

/// OverviewA 组件：显示父级节点列表
/// 
/// # 功能
/// - 显示父级节点列表（当前节点的祖先节点）
/// - 支持高亮显示当前节点的父级
/// - 支持鼠标点击导航
#[component]
pub fn OverviewA(
    overview_a_directories: ReadSignal<Vec<String>>,
    overview_a_selected_path: ReadSignal<Option<String>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_selected_index: WriteSignal<Option<usize>>,
) -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500">
            <li class="w-full min-w-0">
                <button
                    class="w-full h-full text-left hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400 truncate"
                    on:click=move |_| {
                        // 点击 "/" 时，加载一级目录到 OverviewB，但不移动内容
                        spawn_local(async move {
                            if let Ok(directories) = get_root_directories().await {
                                let dir_paths: Vec<String> = directories.iter()
                                    .map(|d| d.path.clone())
                                    .collect();
                                set_overview_b_directories.set(dir_paths);
                                
                                // 设置第一个目录用于 Preview
                                if let Some(first_dir) = directories.first() {
                                    set_preview_path.set(Some(first_dir.path.clone()));
                                }
                                
                                // OverviewA 保持为空（只有 "/"）
                                set_overview_a_directories.set(Vec::new());
                                set_selected_path.set(None);
                            }
                        });
                    }
                >
                    "/"
                </button>
            </li>
            <For
                each=move || overview_a_directories.get()
                key=|path| path.clone()
                children=move |path: String| {
                    let path_clone = path.clone();
                    
                    // 创建导航信号
                    let nav = NavigationSignals {
                        set_overview_a_directories,
                        set_overview_a_selected_path,
                        set_overview_b_directories,
                        set_preview_path,
                        set_selected_path,
                        set_selected_index,
                    };
                    
                    // OverviewA 中不需要 directories，传入空列表
                    let (empty_directories_read, _empty_directories_write) = signal::<Vec<DirectoryNode>>(Vec::new());
                    let data = DataSignals {
                        directories: empty_directories_read,
                    };
                    
                    // 创建 ItemContext
                    let context = ItemContext::from_path(
                        path,
                        nav,
                        data,
                    );
                    
                    // 使用 Memo 计算是否选中（避免闭包类型问题）
                    let is_selected = Memo::new(move |_| {
                        overview_a_selected_path.get().as_ref() == Some(&path_clone)
                    });
                    
                    view! {
                        <ItemComponent
                            context=context
                            is_selected=is_selected
                        />
                    }
                }
            />
        </ul>
    }
}
