use super::HomeLogic;
use crate::components::body::Detail;
use crate::components::header::MobileHeader;
use leptos::callback::Callback;
use leptos::prelude::*;

#[component]
pub fn MobileNavigator(logic: HomeLogic) -> impl IntoView {
    let (pending_path, set_pending_path) = signal::<Option<String>>(None);
    let navigate = logic.mobile_navigate_callback.clone();

    Effect::new({
        let pending_path = pending_path.clone();
        let set_pending_path = set_pending_path.clone();
        move |_| {
            if let Some(target) = pending_path.get() {
                if target.is_empty() {
                    navigate.run(None);
                } else {
                    navigate.run(Some(target.clone()));
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
                    <Detail
                        logic=logic.clone()
                        on_node_click=Callback::new({
                            let set_pending_path = set_pending_path.clone();
                            move |value| set_pending_path.set(value)
                        })
                    />
                </div>

            </div>
        </div>
    }
}
