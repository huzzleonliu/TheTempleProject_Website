use super::HomeLogic;
use crate::components::body::{DetailPanel, OverviewColumn, PresentColumn};
use crate::components::footer::Footer;
use crate::components::header::Header;
use crate::utils::types::DetailItem;
use leptos::prelude::*;
use std::sync::Arc;

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
        detail_vm,
        selected_index,
        detail_click_callback,
        ..
    } = logic;

    let (desktop_detail_items, set_desktop_detail_items) = signal(Vec::<DetailItem>::new());
    let (desktop_loading, set_desktop_loading) = signal(false);
    let (desktop_error, set_desktop_error) = signal(None::<String>);

    Effect::new(move |_| {
        let vm = detail_vm.get();
        set_desktop_loading.set(vm.loading);
        set_desktop_error.set(vm.error);
        set_desktop_detail_items.set(vm.items);
    });

    let on_detail_click: Arc<dyn Fn(DetailItem) + Send + Sync> = {
        let cb = detail_click_callback.clone();
        Arc::new(move |item: DetailItem| cb.run(item))
    };

    view! {
        <div class="flex flex-col h-screen">
            <div class="px-4 pt-4 pb-12 flex-shrink-0">
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
                        loading=desktop_loading
                        error=desktop_error
                        scroll_container_ref=detail_scroll_ref
                        on_node_click=Some(on_detail_click.clone())
                    />
                </div>
            </div>
            <div class="px-4 pb-4 flex-shrink-0">
                <Footer/>
            </div>
        </div>
    }
}
