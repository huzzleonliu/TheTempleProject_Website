use super::subpages::content::Content;
use super::subpages::overview::Overview;
use super::subpages::preview::Preview;
use super::subpages::title::Title;
use leptos::prelude::*;

#[component]
pub fn Home() -> impl IntoView {
    view! {
        // <div class="flex columns-3 columns-gap-4 columns-sm columns-md columns-lg">
        <div class="grid grid-cols-10 gap-1 h-screen p-4">
        <div class="col-span-10">
        <Title/>
            </div>
        <div class="col-span-2 overflow-y-auto">
            <Overview />
            </div>
        <div class="col-span-3 overflow-y-auto ">
            <Content />
            </div>
        <div class="col-span-5 overflow-y-auto ">
            <Preview />
            </div>
        </div>

    }
}
