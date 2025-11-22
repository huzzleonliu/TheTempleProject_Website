#[path = "home_desktop_layout.rs"]
mod desktop_layout;
#[path = "home_logic.rs"]
pub mod logic;
#[path = "home_mobile_layout.rs"]
mod mobile_layout;

use desktop_layout::DesktopLayout;
use leptos::prelude::*;
pub use logic::HomeLogic;
use mobile_layout::MobileNavigator;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const MOBILE_BREAKPOINT_PX: f64 = 1200.0;

/// 页面入口：聚焦数据如何在三栏间流动，操作细节位于 `logic` 模块。
#[component]
pub fn Home() -> impl IntoView {
    let logic = HomeLogic::new();
    let desktop_logic = logic.clone();
    let mobile_logic = logic.clone();
    let is_mobile = use_is_mobile_flag(MOBILE_BREAKPOINT_PX);

    {
        let keyboard_enabled = logic.keyboard_enabled.clone();
        let is_mobile_flag = is_mobile.clone();
        Effect::new(move |_| {
            keyboard_enabled.set(!is_mobile_flag.get());
        });
    }

    view! {
        <Show
            when=move || is_mobile.get()
            fallback=move || view! { <DesktopLayout logic=desktop_logic.clone() /> }
        >
            <MobileNavigator logic=mobile_logic.clone() />
        </Show>
    }
}

fn use_is_mobile_flag(threshold: f64) -> ReadSignal<bool> {
    let initial = is_mobile_viewport(threshold);
    let (is_mobile, set_is_mobile) = signal(initial);

    Effect::new(move |_| {
        set_is_mobile.set(is_mobile_viewport(threshold));
    });

    Effect::new(move |_| {
        if let Some(window) = web_sys::window() {
            let set_is_mobile = set_is_mobile.clone();
            let closure = Closure::wrap(Box::new(move |_event: web_sys::UiEvent| {
                set_is_mobile.set(is_mobile_viewport(threshold));
            }) as Box<dyn FnMut(_)>);
            let _ =
                window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref());
            closure.forget();
        }
    });

    is_mobile
}

fn is_mobile_viewport(threshold: f64) -> bool {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .map(|width| width <= threshold)
        .unwrap_or(false)
}
