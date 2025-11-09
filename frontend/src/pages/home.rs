use crate::components::overview_a::OverviewA;
use crate::components::overview_b::OverviewB;
use crate::components::preview::Preview;
use crate::components::title::Title;
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    // OverviewA 显示的目录列表（父级节点）
    let (overview_a_directories, set_overview_a_directories) = signal::<Vec<String>>(Vec::new());
    // OverviewA 中高亮的路径（当前节点的父级）
    let (overview_a_selected_path, set_overview_a_selected_path) = signal::<Option<String>>(None);
    // OverviewB 显示的目录列表（当前节点）
    let (overview_b_directories, set_overview_b_directories) = signal::<Vec<String>>(Vec::new());
    // Preview 显示的路径（当前选中节点的子节点）
    let (preview_path, set_preview_path) = signal::<Option<String>>(None);
    // OverviewB 中当前选中的路径（用于高亮显示）
    let (selected_path, set_selected_path) = signal::<Option<String>>(None);
    // OverviewB 中当前选中的索引（用于键盘导航）
    let (selected_index, set_selected_index) = signal::<Option<usize>>(None);

    // Preview 滚动容器的 NodeRef（用于从 OverviewB 控制滚动）
    let preview_scroll_ref = NodeRef::<leptos::html::Div>::new();

    view! {
        <div class="flex flex-col h-screen overflow-hidden">
            <div class="px-4 pt-4 pb-0 flex-shrink-0">
                <Title/>
            </div>
            <div class="grid grid-cols-10 grid-rows-1 flex-1 min-h-0 overflow-hidden">
                <div class="col-span-2 h-full min-h-0 overflow-y-auto px-4 pt-0">
                    <OverviewA
                        overview_a_directories=overview_a_directories
                        overview_a_selected_path=overview_a_selected_path
                        set_selected_path=set_selected_path
                        set_overview_b_directories=set_overview_b_directories
                        set_overview_a_directories=set_overview_a_directories
                        set_preview_path=set_preview_path
                    />
                </div>
                <div class="col-span-3 h-full min-h-0 overflow-y-auto px-4 pt-0">
                    <OverviewB
                        overview_b_directories=overview_b_directories
                        set_overview_b_directories=set_overview_b_directories
                        set_overview_a_directories=set_overview_a_directories
                        selected_path=selected_path
                        set_selected_path=set_selected_path
                        set_preview_path=set_preview_path
                        selected_index=selected_index
                        set_selected_index=set_selected_index
                        overview_a_directories=overview_a_directories
                        overview_a_selected_path=overview_a_selected_path
                        set_overview_a_selected_path=set_overview_a_selected_path
                        preview_scroll_ref=preview_scroll_ref
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 overflow-y-auto px-4 pt-0">
                    <Preview
                        preview_path=preview_path
                        scroll_container_ref=preview_scroll_ref
                    />
                </div>
            </div>
        </div>
    }
}
