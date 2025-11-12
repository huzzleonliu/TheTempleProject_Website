use leptos::prelude::*;
use leptos::task::spawn_local;
use crate::{DirectoryNode, Item};
use crate::api::get_root_directories;
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
    items: ReadSignal<Vec<Item>>,
    selected_path: ReadSignal<Option<String>>,
    // "/" 按钮需要的信号
    set_overview_b_directories: WriteSignal<Vec<DirectoryNode>>,
    set_overview_a_directories: WriteSignal<Vec<DirectoryNode>>,
    set_preview_path: WriteSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_overview_b_items: WriteSignal<Vec<Item>>,
) -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500">
            <li class="w-full min-w-0">
                <button
                    class="w-full h-full text-left hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400 truncate"
                    on:click=move |_| {
                        // 点击 "/" 时，加载一级目录到 OverviewB，但不移动内容
                        // 克隆必要的信号（从 items 中获取，如果存在）
                        let items_clone = items.get();
                        let first_item_opt = items_clone.first().cloned();
                        
                        spawn_local(async move {
                            if let Ok(directories) = get_root_directories().await {
                                set_overview_b_directories.set(directories.clone());
                                
                                // 创建新的 Item 列表
                                if let Some(first_item) = first_item_opt {
                                    let (dirs_signal, _) = signal::<Vec<DirectoryNode>>(directories.clone());
                                    let dirs_read = dirs_signal;
                                    let new_items: Vec<Item> = directories
                                        .iter()
                                        .map(|node| {
                                            Item::from_node(
                                                node.clone(),
                                                first_item.set_overview_a_items.clone(),
                                                first_item.set_overview_b_items.clone(),
                                                first_item.set_overview_a_selected_path.clone(),
                                                first_item.set_preview_path.clone(),
                                                first_item.set_selected_path.clone(),
                                                first_item.set_selected_index.clone(),
                                                dirs_read.clone(),
                                                false, // OverviewB
                                            )
                                        })
                                        .collect();
                                    set_overview_b_items.set(new_items);
                                }
                                
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
                each=move || items.get()
                key=|item| item.path.clone()
                children=move |item: Item| {
                    let path_clone = item.path.clone();
                    
                    // 使用 Memo 计算是否选中
                    let is_selected = Memo::new(move |_| {
                        selected_path.get().as_ref() == Some(&path_clone)
                    });
                    
                    view! {
                        <ItemComponent
                            item=item
                            is_selected=is_selected
                        />
                    }
                }
            />
        </ul>
    }
}
