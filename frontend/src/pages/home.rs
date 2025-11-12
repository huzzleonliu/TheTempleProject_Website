use crate::components::overview_a::OverviewA;
use crate::components::overview_b::OverviewB;
use crate::components::preview::Preview;
use crate::components::title::Title;
use crate::components::keyboard_handlers;
use crate::{DirectoryNode, Item};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::rc::Rc;
use std::cell::RefCell;

#[component]
pub fn Home() -> impl IntoView {
    // 统一数据源：都使用 DirectoryNode
    // OverviewA 显示的目录列表（父级节点）
    let (overview_a_directories, set_overview_a_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    // OverviewB 显示的目录列表（当前节点）
    let (overview_b_directories, set_overview_b_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    // OverviewB 中的目录完整信息（用于键盘事件，与 overview_b_directories 同步）
    let (directories, set_directories) = signal::<Vec<DirectoryNode>>(Vec::new());
    
    // 状态信号
    // OverviewA 中高亮的路径（当前节点的父级）
    let (overview_a_selected_path, set_overview_a_selected_path) = signal::<Option<String>>(None);
    // Preview 显示的路径（当前选中节点的子节点）
    let (preview_path, set_preview_path) = signal::<Option<String>>(None);
    // OverviewB 中当前选中的路径（用于高亮显示）
    let (selected_path, set_selected_path) = signal::<Option<String>>(None);
    // OverviewB 中当前选中的索引（用于键盘导航）
    let (selected_index, set_selected_index) = signal::<Option<usize>>(None);

    // Preview 滚动容器的 NodeRef（用于从 OverviewB 控制滚动）
    let preview_scroll_ref = NodeRef::<leptos::html::Div>::new();

    // Item 列表信号
    let (overview_a_items, set_overview_a_items) = signal::<Vec<Item>>(Vec::new());
    let (overview_b_items, set_overview_b_items) = signal::<Vec<Item>>(Vec::new());

    // 当 overview_a_directories 改变时，更新 overview_a_items
    create_effect({
        let set_overview_a_items = set_overview_a_items.clone();
        let set_overview_b_items = set_overview_b_items.clone();
        let set_overview_a_selected_path = set_overview_a_selected_path.clone();
        let set_preview_path = set_preview_path.clone();
        let set_selected_path = set_selected_path.clone();
        let set_selected_index = set_selected_index.clone();
        let overview_b_directories_read = overview_b_directories;
        
        move |_| {
            let items: Vec<Item> = overview_a_directories.get()
                .into_iter()
                .map(|node| {
                    Item::from_node(
                        node,
                        set_overview_a_items.clone(),
                        set_overview_b_items.clone(),
                        set_overview_a_selected_path.clone(),
                        set_preview_path.clone(),
                        set_selected_path.clone(),
                        set_selected_index.clone(),
                        overview_b_directories_read.clone(),
                        true, // is_overview_a
                    )
                })
                .collect();
            set_overview_a_items.set(items);
        }
    });

    // 当 overview_b_directories 改变时，更新 overview_b_items
    create_effect({
        let set_overview_a_items = set_overview_a_items.clone();
        let set_overview_b_items = set_overview_b_items.clone();
        let set_overview_a_selected_path = set_overview_a_selected_path.clone();
        let set_preview_path = set_preview_path.clone();
        let set_selected_path = set_selected_path.clone();
        let set_selected_index = set_selected_index.clone();
        let overview_b_directories_read = overview_b_directories;
        
        move |_| {
            let items: Vec<Item> = overview_b_directories.get()
                .into_iter()
                .map(|node| {
                    Item::from_node(
                        node,
                        set_overview_a_items.clone(),
                        set_overview_b_items.clone(),
                        set_overview_a_selected_path.clone(),
                        set_preview_path.clone(),
                        set_selected_path.clone(),
                        set_selected_index.clone(),
                        overview_b_directories_read.clone(),
                        false, // is_overview_a
                    )
                })
                .collect();
            set_overview_b_items.set(items);
        }
    });

    // 同步 directories：当 overview_b_directories 变化时，更新 directories（用于键盘事件）
    create_effect(move |_| {
        set_directories.set(overview_b_directories.get());
    });

    // 在 window 上添加全局键盘事件监听器
    // 这样无论焦点在哪里，都能捕获键盘事件
    // 使用 Rc<RefCell<bool>> 来确保只添加一次监听器
    let listener_added = Rc::new(RefCell::new(false));
    let listener_added_clone = listener_added.clone();
    
    create_effect(move |_| {
        // 只在组件挂载时执行一次
        if *listener_added_clone.borrow() {
            return;
        }
        *listener_added_clone.borrow_mut() = true;
        
        let handle_global_keydown = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            // 检查当前焦点是否在输入框、文本区域等元素上
            // 如果是，则不处理 hjkl 键，让用户正常输入
            if let Some(active_element) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element())
            {
                let tag_name = active_element.tag_name();
                let node_name = active_element.node_name();
                
                // 如果焦点在输入框、文本区域、可编辑元素上，不处理 hjkl 键
                let is_contenteditable = active_element.has_attribute("contenteditable");
                
                if matches!(tag_name.as_str(), "INPUT" | "TEXTAREA") 
                    || is_contenteditable
                    || matches!(node_name.as_str(), "INPUT" | "TEXTAREA") {
                    return;
                }
            }

            // 将 web_sys::KeyboardEvent 转换为 leptos::ev::KeyboardEvent
            // 创建一个包装器来处理事件
            let key = event.key();
            let shift_pressed = event.shift_key();
            
            // 只处理 hjkl 键（带或不带 Shift）
            let should_handle = match key.as_str() {
                "j" | "J" | "k" | "K" | "l" | "L" | "h" | "H" => true,
                _ => false,
            };
            
            if !should_handle {
                return;
            }
            
            // 阻止默认行为和事件冒泡
            event.prevent_default();
            event.stop_propagation();
            
            // 处理键盘导航
            keyboard_handlers::handle_keyboard_navigation(
                &event, // 传递原始事件，keyboard_handlers 会处理
                directories.get(),
                selected_index.get(),
                overview_a_directories.get().as_slice(),
                overview_a_selected_path.get(),
                set_selected_index,
                set_selected_path,
                set_overview_a_selected_path,
                set_overview_a_directories,
                set_overview_b_directories,
                set_preview_path,
                set_directories,
                preview_scroll_ref,
            );
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        // 在 window 上添加事件监听器
        if let Some(window) = web_sys::window() {
            let _ = window
                .add_event_listener_with_callback(
                    "keydown",
                    handle_global_keydown.as_ref().unchecked_ref(),
                );

            // 保存闭包的引用，防止被释放
            handle_global_keydown.forget();
        }
    });

    view! {
        <div 
            class="flex flex-col h-screen"
        >
            <div class="px-4 pt-4 pb-0 flex-shrink-0">
                <Title/>
            </div>
            <div class="grid grid-cols-10 grid-rows-1 flex-1 min-h-0 overflow-hidden items-start">
                <div class="col-span-2 overflow-y-auto px-4 pt-0">
                    <OverviewA
                        items=overview_a_items
                        selected_path=overview_a_selected_path
                        set_overview_b_directories=set_overview_b_directories
                        set_overview_a_directories=set_overview_a_directories
                        set_preview_path=set_preview_path
                        set_selected_path=set_selected_path
                        set_overview_b_items=set_overview_b_items
                    />
                </div>
                <div class="col-span-3 overflow-y-auto px-4 pt-0">
                    <OverviewB
                        items=overview_b_items
                        selected_path=selected_path
                        selected_index=selected_index
                        preview_scroll_ref=preview_scroll_ref
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 px-4 pt-0">
                    <Preview
                        preview_path=preview_path
                        scroll_container_ref=preview_scroll_ref
                    />
                </div>
            </div>
        </div>
    }
}
