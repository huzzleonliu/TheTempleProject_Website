use crate::{
    components::preview::Preview,
    pages::home::HomeLogic,
    types::{parent_path, PreviewItem},
};
use leptos::prelude::*;
use std::sync::Arc;

const ROOT_LABEL: &str = "/";

#[component]
pub fn MobileNavigator(logic: HomeLogic) -> impl IntoView {
    let (pending_path, set_pending_path) = create_signal::<Option<String>>(None);
    Effect::new({
        let pending_path = pending_path.clone();
        let set_pending_path = set_pending_path.clone();
        let navigate_callback = logic.mobile_navigate_callback.clone();
        move |_| {
            if let Some(path) = pending_path.get() {
                if path.is_empty() {
                    navigate_callback.run(None);
                } else {
                    navigate_callback.run(Some(path.clone()));
                }
                set_pending_path.set(None);
            }
        }
    });
    view! {
        <div class="flex h-screen bg-black text-white">
            <div class="flex flex-col overflow-hidden w-full">
                <MobileHeader
                    current_path=logic.current_path.clone()
                    set_pending_path=set_pending_path.clone()
                />
                <div class="relative flex-1 overflow-hidden">
                    <PaneView logic=logic.clone() set_pending_path=set_pending_path.clone() />
                </div>
            </div>
        </div>
    }
}

#[component]
fn PaneView(logic: HomeLogic, set_pending_path: WriteSignal<Option<String>>) -> impl IntoView {
    let preview_items = logic.preview_items.read_only();
    let preview_loading = logic.preview_loading.read_only();
    let preview_error = logic.preview_error.read_only();
    let preview_scroll_ref = logic.preview_scroll_ref.clone();
    let pane_key = Memo::new({
        let current_path = logic.current_path.clone();
        move |_| current_path.get().unwrap_or_else(|| "root".to_string())
    });
    let on_directory_click = Arc::new({
        let set_pending_path = set_pending_path.clone();
        move |item: PreviewItem| {
            if let Some(path) = item.directory_path.clone() {
                set_pending_path.set(Some(path));
            }
        }
    });

    view! {
        <div class=move || format!("absolute inset-0 flex flex-col gap-4 p-4 pane-{}", pane_key.get())>
            <div class="border border-gray-800 rounded-xl overflow-hidden flex-1">
                <Preview
                    items=preview_items
                    loading=preview_loading
                    error=preview_error
                    scroll_container_ref=preview_scroll_ref
                    on_directory_click=Some(on_directory_click.clone())
                />
            </div>
        </div>
    }
}

#[component]
fn MobileHeader(
    current_path: RwSignal<Option<String>>,
    set_pending_path: WriteSignal<Option<String>>,
) -> impl IntoView {
    let segments = Memo::new(move |_| format_segments(current_path.get()));

    view! {
        <div class="h-[5vh] min-h-[48px] flex items-center gap-3 px-4 border-b border-gray-900">
            <button
                class="px-3 py-2 rounded border border-gray-700 text-sm uppercase tracking-wide hover:bg-gray-800 disabled:opacity-40 disabled:cursor-not-allowed"
                on:click=move |_| {
                    if let Some(path) = current_path.get() {
                        if let Some(parent) = parent_path(&path) {
                            set_pending_path.set(Some(parent));
                        } else {
                            set_pending_path.set(Some(String::new()));
                        }
                    }
                }
                disabled=move || current_path.get().is_none()
            >
                "Back"
            </button>
            <div class="flex-1 flex gap-2 text-sm overflow-hidden">
                <For
                    each=move || segments.get()
                    key=|segment| segment.clone()
                    children=move |segment: String| {
                        view! { <span class="truncate max-w-[5ch]">{segment}</span> }
                    }
                />
            </div>
        </div>
    }
}

fn format_segments(path: Option<String>) -> Vec<String> {
    match path {
        Some(p) if !p.is_empty() => p
            .split('.')
            .map(|segment| truncate_segment(segment))
            .collect(),
        _ => vec![ROOT_LABEL.to_string()],
    }
}

fn truncate_segment(segment: &str) -> String {
    if segment.chars().count() <= 5 {
        segment.to_string()
    } else {
        segment.chars().take(5).collect::<String>() + "..."
    }
}
