use leptos::prelude::*;
use web_sys::console;

use crate::Item;
use crate::components::item::ItemComponent;

/// OverviewB 组件：显示当前层级的目录列表
/// 
/// # 功能
/// - 显示当前层级的目录列表
/// - 支持鼠标点击导航
/// - 支持键盘导航（j/k/l/h 键）
/// - 支持 Shift+J/K 滚动 Preview
/// 
/// # 参数
/// - `items`: Item 列表（在 Home 中创建）
/// - `selected_path`: 当前选中的路径（用于高亮显示）
/// - `selected_index`: 当前选中的索引（用于键盘导航）
/// - `preview_scroll_ref`: Preview 滚动容器的引用（用于 Shift+J/K 滚动）
#[component]
pub fn OverviewB(
    items: ReadSignal<Vec<Item>>,
    selected_path: ReadSignal<Option<String>>,
    selected_index: ReadSignal<Option<usize>>,
    preview_scroll_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    // 当 items 改变时，如果索引未设置或超出范围，则重置为 0
    create_effect(move |_| {
        let items_list = items.get();
        if !items_list.is_empty() {
            if let Some(current_idx) = selected_index.get() {
                if current_idx >= items_list.len() {
                    console::log_2(&"[OverviewB] 索引超出范围，重置为 0。当前索引:".into(), &current_idx.into());
                    console::log_2(&"[OverviewB] 列表长度:".into(), &items_list.len().into());
                    // 通过第一个 item 的信号来更新索引
                    if let Some(first_item) = items_list.first() {
                        first_item.set_selected_index.set(Some(0));
                    }
                }
            } else {
                console::log_1(&"[OverviewB] 索引未设置，重置为 0".into());
                if let Some(first_item) = items_list.first() {
                    first_item.set_selected_index.set(Some(0));
                }
            }
        }
    });

    // 当选中索引改变时，更新 Preview 显示的内容
    create_effect(move |_| {
        let items_list = items.get();
        if let Some(index) = selected_index.get() {
            if index < items_list.len() {
                let selected_item = &items_list[index];
                console::log_2(&"[OverviewB] 选中索引改变，更新 Preview:".into(), &selected_item.path.clone().into());
                
                // 设置选中的路径（用于高亮显示）
                selected_item.set_selected_path.set(Some(selected_item.path.clone()));
                
                // 只有当节点有子节点时才设置 Preview
                if selected_item.has_subnodes {
                    selected_item.set_preview_path.set(Some(selected_item.path.clone()));
                } else {
                    selected_item.set_preview_path.set(None);
                }
            }
        }
    });

    view! {
        <ul 
            class="text-2xl text-gray-500 outline-none"
        >
            <For
                each=move || items.get()
                key=|item| item.path.clone()
                children=move |item: Item| {
                    let path_clone = item.path.clone();
                    
                    // 使用 Memo 计算是否选中
                    let is_selected = Memo::new(move |_| {
                        if let Some(index) = selected_index.get() {
                            let items_list = items.get();
                            if let Some(item_idx) = items_list.iter().position(|it| it.path == path_clone) {
                                index == item_idx
                            } else {
                                false
                            }
                        } else {
                            selected_path.get().as_ref() == Some(&path_clone)
                        }
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
