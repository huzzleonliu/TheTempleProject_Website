use super::HomeLogic;
use crate::components::body::{DetailPanel, OverviewColumn, PresentColumn};
use crate::components::footer::Footer;
use crate::components::header::Header;
use leptos::prelude::*;

#[component]
pub fn DesktopLayout(logic: HomeLogic) -> impl IntoView {
    view! {
        <div class="flex flex-col h-screen">
            <div class="px-4 pt-4 pb-12 flex-shrink-0">
                <Header/>
            </div>
            <div class="grid grid-cols-10 grid-rows-1 flex-1 min-h-0 overflow-hidden items-start">
                <div class="col-span-2 overflow-y-auto px-4 pt-0">
                    <OverviewColumn
                        nodes=logic.derived.overview_nodes
                        current_path=logic.state.current_path.read_only()
                        on_select=logic.actions.overview_select
                    />
                </div>
                <div class="col-span-3 h-full min-h-0 px-4 pt-0">
                    <PresentColumn
                        nodes=logic.derived.present_nodes
                        scroll_container_ref=logic.refs.present_scroll
                        selected_index=logic.state.selected_index.read_only()
                        on_select=logic.actions.present_select
                        on_enter=logic.actions.present_enter
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 px-4 pt-0">
                    <DetailPanel
                        nodes=logic.derived.detail_nodes
                        is_present_dir_detail=logic.derived.is_present_dir_detail
                        scroll_container_ref=logic.refs.detail_scroll
                        on_node_click=Some(logic.actions.detail_enter.clone())
                    />
                </div>
            </div>
            <div class="px-4 pb-4 flex-shrink-0">
                <Footer/>
            </div>
        </div>
    }
}
