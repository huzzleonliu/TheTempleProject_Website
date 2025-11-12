use leptos::prelude::*;
use crate::Item;

/// Item 组件：统一的列表项组件
/// 
/// 直接调用 Item 的 handle_click() 方法处理点击事件
/// 
/// # 参数
/// - `item`: Item 实例，包含数据和所有操作信号
/// - `is_selected`: 是否选中的 Memo
#[component]
pub fn ItemComponent(
    item: Item,
    is_selected: Memo<bool>,
) -> impl IntoView {
    let display_name = item.raw_filename.clone();
    let item_clone = item.clone();
    
    view! {
        <li class="w-full min-w-0">
            <button
                class=move || {
                    let base_class = "w-full h-full text-left truncate text-2xl";
                    if is_selected.get() {
                        format!("{} text-white bg-gray-800", base_class)
                    } else {
                        format!("{} hover:text-white hover:bg-gray-800 focus-within:bg-gray-600 focus-within:text-white active:bg-gray-400", base_class)
                    }
                }
                on:click=move |_| {
                    item_clone.handle_click();
                }
            >
                {display_name}
            </button>
        </li>
    }
}

