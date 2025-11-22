use super::HomeLogic;
use crate::components::body::{DetailPanel, OverviewColumn, PresentColumn};
use crate::components::footer::Footer;
use crate::components::header::Header;
use crate::utils::types::DetailItem;
use leptos::prelude::*;

#[component]
pub fn DesktopLayout(logic: HomeLogic) -> impl IntoView {
    let HomeLogic {
        present_nodes,
        overview_nodes,
        overview_highlight,
        present_select_callback,
        present_enter_callback,
        overview_select_callback,
        detail_scroll_ref,
        present_scroll_ref,
        detail_items,
        detail_loading,
        detail_error,
        detail_path,
        selected_index,
        ..
    } = logic;

    let raw_detail_items = detail_items.read_only();
    let detail_path_signal = detail_path.clone();
    let (desktop_detail_items, set_desktop_detail_items) = signal(Vec::<DetailItem>::new());

    Effect::new(move |_| {
        let items = raw_detail_items.get();
        let forced = if detail_path_signal.get().is_some() {
            items
                .into_iter()
                .map(|mut item| {
                    item.display_as_entry = true;
                    item
                })
                .collect()
        } else {
            items
        };
        set_desktop_detail_items.set(forced);
    });

    view! {
        <div class="flex flex-col h-screen">
            <div class="px-4 pt-4 pb-0 flex-shrink-0">
                <Header/>
            </div>
            <div class="grid grid-cols-10 grid-rows-1 flex-1 min-h-0 overflow-hidden items-start">
                <div class="col-span-2 overflow-y-auto px-4 pt-0">
                    <OverviewColumn
                        nodes=overview_nodes
                        highlighted_path=overview_highlight
                        on_select=overview_select_callback
                    />
                </div>
                <div class="col-span-3 h-full min-h-0 px-4 pt-0">
                    <PresentColumn
                        nodes=present_nodes
                        scroll_container_ref=present_scroll_ref
                        selected_index=selected_index.read_only()
                        on_select=present_select_callback
                        on_enter=present_enter_callback
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 px-4 pt-0">
                    <DetailPanel
                        items=desktop_detail_items
                        loading=detail_loading.read_only()
                        error=detail_error.read_only()
                        scroll_container_ref=detail_scroll_ref
                    />
                </div>
            </div>
            <div class="px-4 pb-4 flex-shrink-0">
                <Footer/>
            </div>
        </div>
    }
}
