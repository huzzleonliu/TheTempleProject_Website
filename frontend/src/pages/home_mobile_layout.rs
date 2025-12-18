use super::HomeLogic;
use crate::components::body::DetailPanel;
use crate::components::header::MobileHeader;
use leptos::prelude::*;
use crate::pages::home::logic::{navigate_to};
use super::desktop_layout::detail_enter;

#[component]
pub fn MobileNavigator(logic: HomeLogic) -> impl IntoView {
    // --------- refs (scroll containers) ---------
    let detail_scroll = NodeRef::<leptos::html::Div>::new();

    // --------- actions (UI events) ---------
    let detail_enter_cb = {
        let logic = logic.clone();
        UnsyncCallback::new(move |idx: usize| detail_enter(&logic, idx))
    };

    // --------- navigation (mobile header) ---------
    let (pending_path, set_pending_path) = signal::<Option<String>>(None);
    Effect::new({
        let logic = logic.clone();
        let pending_path = pending_path.clone();
        let set_pending_path = set_pending_path.clone();
        move |_| {
            if let Some(target) = pending_path.get() {
                if target.is_empty() {
                    navigate_to(&logic, None, None);
                } else {
                    navigate_to(&logic, Some(target.clone()), None);
                }
                set_pending_path.set(None);
            }
        }
    });

    view! {
        <div class="flex min-h-[100dvh] bg-black text-white">
            <div class="flex flex-col w-full min-h-[100dvh]">
                <MobileHeader current_path=logic.current_path.clone() set_pending_path=set_pending_path.clone() />
                <div class="relative flex-1 min-h-0 overflow-hidden">
                    <div class="absolute inset-0 flex flex-col gap-4 p-4">
                        <div class="border border-gray-800 rounded-xl overflow-hidden flex-1">
                            <DetailPanel
                                nodes=logic.detail_nodes
                                detail_render_mode=logic.detail_render_mode
                                on_enter=detail_enter_cb
                                scroll_container_ref=detail_scroll
                            />
                        </div>
                    </div>
                </div>

            </div>
        </div>
    }
}
