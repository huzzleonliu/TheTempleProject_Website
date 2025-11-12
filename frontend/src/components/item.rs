use leptos::prelude::*;
use crate::{ItemContext, ItemType};
use crate::components::mouse_handlers;

/// Item 组件：统一的列表项组件
/// 
/// 根据 item_type 进行模式匹配，执行不同的操作
/// 
/// # 参数
/// - `context`: Item 上下文，包含 Item 数据和所有相关的信号
/// - `is_selected`: 是否选中的 Memo
#[component]
pub fn ItemComponent(
    context: ItemContext,
    is_selected: Memo<bool>,
) -> impl IntoView {
    let path = context.item.path.clone();
    let has_subnodes = context.item.has_subnodes;
    let display_name = context.item.raw_filename.clone();
    let item_type = context.item.item_type.clone();
    
    // 提取导航信号和数据信号
    let nav = context.nav.clone();
    let data = context.data.clone();
    let is_overview_a = context.is_overview_a;
    
    // 根据 item_type 决定样式
    let item_class = match item_type {
        ItemType::Node => "text-2xl",
        // ItemType::File => "text-xl pl-4",
    };
    
    // 根据 item_type 和 is_overview_a 进行模式匹配，决定点击行为
    let handle_click = move |_| {
        match item_type {
            ItemType::Node => {
                if is_overview_a {
                    // OverviewA 中的处理：调用 handle_overview_a_click
                    mouse_handlers::handle_overview_a_click(
                        path.clone(),
                        nav.set_selected_path,
                        nav.set_overview_b_directories,
                        nav.set_overview_a_directories,
                        nav.set_preview_path,
                        nav.set_selected_index,
                    );
                } else {
                    // OverviewB 中的处理：调用 handle_node_click
                    mouse_handlers::handle_node_click(
                        path.clone(),
                        has_subnodes,
                        data.directories.get(),
                        nav.set_overview_a_directories,
                        nav.set_overview_a_selected_path,
                        nav.set_overview_b_directories,
                        nav.set_preview_path,
                        nav.set_selected_path,
                        nav.set_selected_index,
                    );
                }
            }
            // 未来可以添加其他类型的处理：
            // ItemType::File => {
            //     // File 类型的处理逻辑
            //     // 例如：打开文件预览、下载等
            // }
        }
    };
    
    view! {
        <li class="w-full min-w-0">
            <button
                class=move || {
                    let base_class = format!("w-full h-full text-left truncate {}", item_class);
                    if is_selected.get() {
                        format!("{} text-white bg-gray-800", base_class)
                    } else {
                        format!("{} hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400", base_class)
                    }
                }
                on:click=handle_click
            >
                {display_name}
            </button>
        </li>
    }
}

