use crate::components::overview_a::OverviewA;
use crate::components::overview_b::OverviewB;
use crate::components::preview::Preview;
use crate::components::title::Title;
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    // 管理选中的目录路径（初始为空，表示根目录）
    let (selected_path, set_selected_path) = signal::<Option<String>>(None);
    // 管理 OverviewB 的第一个目录（用于 Preview 初始显示）
    let (first_directory, set_first_directory) = signal::<Option<String>>(None);

    view! {
        <div class="grid grid-cols-10 gap-1 h-screen p-4">
            <div class="col-span-10">
                <Title/>
            </div>
            <div class="col-span-2 overflow-y-auto">
                <OverviewA />
            </div>
            <div class="col-span-3 overflow-y-auto">
                <OverviewB selected_path=set_selected_path first_directory=set_first_directory/>
            </div>
            <div class="col-span-5 overflow-y-auto">
                <Preview selected_path=selected_path first_directory=first_directory/>
            </div>
        </div>
    }
}
