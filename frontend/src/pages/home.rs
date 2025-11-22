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
use urlencoding::decode;
use urlencoding::encode;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;

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

    {
        let logic_for_url = logic.clone();
        Effect::new(move |_| {
            if let Some(initial_path) = read_path_from_url() {
                logic_for_url
                    .mobile_navigate_callback
                    .run(Some(initial_path));
            }
        });
    }

    {
        let current_path = logic.current_path.clone();
        Effect::new(move |_| {
            sync_url_with_path(current_path.get());
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

fn read_path_from_url() -> Option<String> {
    let window = web_sys::window()?;
    let location = window.location();
    let search = location.search().ok()?;
    if search.len() <= 1 {
        return None;
    }
    search[1..]
        .split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or_default();
            if key == "path" {
                decode(value).ok().map(|cow| cow.into_owned())
            } else {
                None
            }
        })
        .find(|path| !path.is_empty())
}

fn sync_url_with_path(path: Option<String>) {
    let window = match web_sys::window() {
        Some(win) => win,
        None => return,
    };

    let mut base = match window.location().origin() {
        Ok(origin) => origin,
        Err(_) => return,
    };

    match window.location().pathname() {
        Ok(pathname) => base.push_str(&pathname),
        Err(_) => base.push('/'),
    }

    let mut final_url = base;
    if let Some(path_value) = path {
        if !path_value.is_empty() {
            final_url.push_str("?path=");
            final_url.push_str(&encode(&path_value));
        }
    }

    if let Ok(history) = window.history() {
        let _ = history.replace_state_with_url(&JsValue::NULL, "", Some(&final_url));
    }
}
