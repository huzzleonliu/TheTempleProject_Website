use crate::{components::detail_panel::DetailPanel, pages::home::HomeLogic, types::DetailItem};
use leptos::prelude::*;
use std::sync::Arc;

#[component]
pub fn DetailPane(logic: HomeLogic, on_node_click: Callback<Option<String>>) -> impl IntoView {
    let detail_items = logic.detail_items.read_only();
    let detail_loading = logic.detail_loading.read_only();
    let detail_error = logic.detail_error.read_only();
    let detail_scroll_ref = logic.detail_scroll_ref.clone();
    let pane_key = Memo::new({
        let current_path = logic.current_path.clone();
        move |_| current_path.get().unwrap_or_else(|| "root".to_string())
    });

    let detail_callback: Arc<dyn Fn(DetailItem) + Send + Sync> = {
        let handler = on_node_click.clone();
        Arc::new(move |item: DetailItem| {
            handler.run(item.directory_path.clone());
        })
    };

    view! {
        <div class=move || format!("absolute inset-0 flex flex-col gap-4 p-4 pane-{}", pane_key.get())>
            <div class="border border-gray-800 rounded-xl overflow-hidden flex-1">
                <DetailPanel
                    items=detail_items
                    loading=detail_loading
                    error=detail_error
                    scroll_container_ref=detail_scroll_ref
                    on_node_click=Some(detail_callback.clone())
                />
            </div>
        </div>
    }
}
