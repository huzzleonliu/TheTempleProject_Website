mod logic;

use crate::components::overview_a::OverviewA;
use crate::components::overview_b::OverviewB;
use crate::components::preview::Preview;
use crate::components::title::Title;
use leptos::prelude::*;
use logic::HomeLogic;

/// 页面入口：聚焦数据如何在三栏间流动，操作细节位于 `logic` 模块。
#[component]
pub fn Home() -> impl IntoView {
    let HomeLogic {
        current_nodes,
        overview_a_nodes,
        overview_a_highlight,
        select_index_callback,
        enter_index_callback,
        overview_a_select_callback,
        preview_scroll_ref,
        preview_nodes,
        preview_loading,
        preview_error,
        selected_index,
        ..
    } = HomeLogic::new();

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
                <div class="col-span-3 overflow-y-auto px-4 pt-0">
                    <OverviewB
                        nodes=current_nodes
                        selected_index=selected_index.read_only()
                        on_select=select_index_callback
                        on_enter=enter_index_callback
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 px-4 pt-0">
                    <Preview
                        nodes=preview_nodes.read_only()
                        loading=preview_loading.read_only()
                        error=preview_error.read_only()
                        scroll_container_ref=preview_scroll_ref
                    />
                </div>
            </div>
        </div>
    }
}
