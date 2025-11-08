use crate::components::overview_a::OverviewA;
use crate::components::overview_b::OverviewB;
use crate::components::preview::Preview;
use crate::components::title::Title;
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    // OverviewA 显示的目录列表（上一级节点）
    let (overview_a_directories, set_overview_a_directories) = signal::<Vec<String>>(Vec::new());
    // OverviewB 显示的目录列表（当前节点的兄弟节点）
    let (overview_b_directories, set_overview_b_directories) = signal::<Vec<String>>(Vec::new());
    // Preview 显示的路径（被点击的节点）
    let (preview_path, set_preview_path) = signal::<Option<String>>(None);
    // 当前选中的路径（用于高亮显示）
    let (selected_path, set_selected_path) = signal::<Option<String>>(None);
    // OverviewB 中当前选中的索引（用于键盘导航）
    let (selected_index, set_selected_index) = signal::<Option<usize>>(None);

    view! {
        <div class="grid grid-cols-10 gap-1 h-screen p-4">
            <div class="col-span-10">
                <Title/>
            </div>
            <div class="col-span-2 overflow-y-auto">
                <OverviewA 
                    overview_a_directories=overview_a_directories
                    set_selected_path=set_selected_path
                    set_overview_b_directories=set_overview_b_directories
                    set_overview_a_directories=set_overview_a_directories
                    set_preview_path=set_preview_path
                />
            </div>
            <div class="col-span-3 overflow-y-auto">
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
                />
            </div>
            <div class="col-span-5 overflow-y-auto">
                <Preview 
                    preview_path=preview_path
                />
            </div>
        </div>
    }
}
