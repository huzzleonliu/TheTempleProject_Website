pub mod logic;

use crate::components::mobile::MobileNavigator;
use crate::components::overview_a::OverviewA;
use crate::components::overview_b::OverviewB;
use crate::components::preview::Preview;
use crate::components::title::Title;
use leptos::prelude::*;
pub use logic::HomeLogic;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;

const MOBILE_BREAKPOINT_PX: f64 = 900.0;

/// 页面入口：聚焦数据如何在三栏间流动，操作细节位于 `logic` 模块。
#[component]
pub fn Home() -> impl IntoView {
    let logic = HomeLogic::new();
    let desktop_logic = logic.clone();
    let mobile_logic = logic.clone();
    let is_mobile = use_is_mobile_flag(MOBILE_BREAKPOINT_PX);

    view! {
        <Show
            when=move || is_mobile.get()
            fallback=move || view! { <DesktopHome logic=desktop_logic.clone() /> }
        >
            <MobileNavigator logic=mobile_logic.clone() />
        </Show>
    }
}

#[component]
fn DesktopHome(logic: HomeLogic) -> impl IntoView {
    let HomeLogic {
        current_nodes,
        overview_a_nodes,
        overview_a_highlight,
        select_index_callback,
        enter_index_callback,
        overview_a_select_callback,
        preview_scroll_ref,
        overview_b_scroll_ref,
        preview_items,
        preview_loading,
        preview_error,
        selected_index,
        ..
    } = logic;

    view! {
        <div class="flex flex-col h-screen">
            <div class="px-4 pt-4 pb-0 flex-shrink-0">
                <Title/>
            </div>
            <div class="grid grid-cols-10 grid-rows-1 flex-1 min-h-0 overflow-hidden items-start">
                <div class="col-span-2 overflow-y-auto px-4 pt-0">
                    <OverviewA
                        nodes=overview_a_nodes
                        highlighted_path=overview_a_highlight
                        on_select=overview_a_select_callback
                    />
                </div>
                <div class="col-span-3 h-full min-h-0 px-4 pt-0">
                    <OverviewB
                        nodes=current_nodes
                        scroll_container_ref=overview_b_scroll_ref
                        selected_index=selected_index.read_only()
                        on_select=select_index_callback
                        on_enter=enter_index_callback
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 px-4 pt-0">
                    <Preview
                        items=preview_items.read_only()
                        loading=preview_loading.read_only()
                        error=preview_error.read_only()
                        scroll_container_ref=preview_scroll_ref
                    />
                </div>
            </div>
        </div>
    }
}

fn use_is_mobile_flag(threshold: f64) -> ReadSignal<bool> {
    let initial = is_mobile_viewport(threshold);
    let (is_mobile, set_is_mobile) = create_signal(initial);

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
