use crate::types::parent_path;
use leptos::prelude::*;

const ROOT_LABEL: &str = "/";

#[component]
pub fn MobileHeader(
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
        format!("{}…", &segment[..5])
    }
}
