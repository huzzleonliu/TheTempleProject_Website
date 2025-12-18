use leptos::prelude::*;
use std::cell::Cell;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

/// 处理键盘导航事件
pub fn handle_keyboard_navigation(
    event: &web_sys::KeyboardEvent,
    move_selection: Rc<dyn Fn(i32)>,
    enter_selection: Rc<dyn Fn()>,
    go_back: Rc<dyn Fn()>,
    detail_scroll_ref: NodeRef<leptos::html::Div>,
    present_scroll_ref: NodeRef<leptos::html::Div>,
) {
    let key = event.key();
    let shift_pressed = event.shift_key();

    // Shift+J / Shift+K 控制 Detail 滚动
    if shift_pressed {
        match key.as_str() {
            "J" | "j" => {
                event.prevent_default();
                event.stop_propagation();
                scroll_detail(&detail_scroll_ref, 120.0);
            }
            "K" | "k" => {
                event.prevent_default();
                event.stop_propagation();
                scroll_detail(&detail_scroll_ref, -120.0);
            }
            _ => {}
        }
        return;
    }

    // 只处理 j/k/l/h
    match key.as_str() {
        "j" | "k" | "l" | "h" => {
            event.prevent_default();
            event.stop_propagation();
        }
        _ => return,
    }

    match key.as_str() {
        "j" => {
            move_selection(1);
            scroll_present(&present_scroll_ref, 1);
        }
        "k" => {
            move_selection(-1);
            scroll_present(&present_scroll_ref, -1);
        }
        "l" => enter_selection(),
        "h" => go_back(),
        _ => {}
    }
}

/// 安装全局键盘监听（j/k/l/h + shift+j/k）并桥接到 `handle_keyboard_navigation`。
///
/// 说明：此函数内部用 `Effect` 确保同一个组件实例只安装一次监听。
pub fn install_keyboard_listener(
    move_selection: Rc<dyn Fn(i32)>,
    enter_selection: Rc<dyn Fn()>,
    go_back: Rc<dyn Fn()>,
    detail_scroll_ref: NodeRef<leptos::html::Div>,
    present_scroll_ref: NodeRef<leptos::html::Div>,
) {
    let installed = Rc::new(Cell::new(false));
    Effect::new(move |_| {
        if installed.get() {
            return;
        }
        installed.set(true);

        let move_selection = move_selection.clone();
        let enter_selection = enter_selection.clone();
        let go_back = go_back.clone();

        let handle_global_keydown = Closure::wrap(Box::new(move |event: web_sys::KeyboardEvent| {
            // 避免输入框抢占快捷键
            if let Some(active) = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.active_element())
            {
                let tag = active.tag_name();
                if matches!(tag.as_str(), "INPUT" | "TEXTAREA") || active.has_attribute("contenteditable") {
                    return;
                }
            }

            handle_keyboard_navigation(
                &event,
                move_selection.clone(),
                enter_selection.clone(),
                go_back.clone(),
                detail_scroll_ref.clone(),
                present_scroll_ref.clone(),
            );
        }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

        if let Some(window) = web_sys::window() {
            let _ = window.add_event_listener_with_callback(
                "keydown",
                handle_global_keydown.as_ref().unchecked_ref(),
            );
        }
        handle_global_keydown.forget();
    });
}

fn scroll_detail(detail_scroll_ref: &NodeRef<leptos::html::Div>, delta: f64) {
    if let Some(container) = detail_scroll_ref.get() {
        let current_scroll = container.scroll_top() as f64;
        let max_scroll = (container.scroll_height() - container.client_height()) as f64;
        let new_scroll = (current_scroll + delta).clamp(0.0, max_scroll);
        container.set_scroll_top(new_scroll as i32);
    }
}

fn scroll_present(present_ref: &NodeRef<leptos::html::Div>, direction: i32) {
    if let Some(container) = present_ref.get() {
        let current_scroll = container.scroll_top();
        let line_height = 40; // approximate line height for list items
        let delta = direction * line_height;
        let max_scroll = container.scroll_height() - container.client_height();
        let new_scroll = (current_scroll + delta).clamp(0, max_scroll);
        container.set_scroll_top(new_scroll);
    }
}
