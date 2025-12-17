use crate::utils::lang::Lang;
use crate::utils::types::parent_path;
use leptos::prelude::*;
use leptos_icons::Icon;

const ROOT_LABEL: &str = "/";

// ---------------- Desktop Header ----------------
#[component]
pub fn Header() -> impl IntoView {
    let lang = use_context::<RwSignal<Lang>>()
        .expect("Lang context should be provided in App");

    let title_text = move || match lang.get() {
        Lang::Zh => "神庙计划",
        Lang::En => "The Temple Project",
    };

    let toggle_label = move || match lang.get() {
        Lang::Zh => "中文",
        Lang::En => "EN",
    };

    view! {
        <header class="flex flex-col gap-2">
            <div class="flex items-center justify-between gap-4">
                <h1 class="text-4xl font-semibold">
                    {title_text}
                </h1>
                <button
                    class="inline-flex items-center gap-1 px-3 py-1 border border-gray-700 text-md text-gray-300 hover:bg-gray-900"
                    on:click=move |_| {
                        lang.update(|l| *l = l.toggle());
                    }
                >
                    <Icon icon=icondata::LuGlobe />
                    <span>{toggle_label}</span>
                </button>
            </div>
        </header>
    }
}

// ---------------- Mobile Header ----------------
#[component]
pub fn MobileHeader(
    current_path: RwSignal<Option<String>>,
    set_pending_path: WriteSignal<Option<String>>,
) -> impl IntoView {
    let lang = use_context::<RwSignal<Lang>>()
        .expect("Lang context should be provided in App");

    let segments = Memo::new(move |_| format_segments(current_path.get()));

    let back_text = move || match lang.get() {
        Lang::Zh => "返回",
        Lang::En => "Back",
    };

    let toggle_label = move || match lang.get() {
        Lang::Zh => "中文",
        Lang::En => "EN",
    };

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
                {back_text}
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
            <button
                class="ml-auto px-2 py-1 rounded-full border border-gray-700 text-sm text-gray-300 hover:bg-gray-800 inline-flex items-center gap-1"
                on:click=move |_| {
                    lang.update(|l| *l = l.toggle());
                }
            >
                <Icon icon=icondata::LuGlobe />
                <span>{toggle_label}</span>
            </button>
        </div>
    }
}

fn format_segments(path: Option<String>) -> Vec<String> {
    match path {
        Some(p) if !p.is_empty() => p.split('.').map(truncate_segment).collect(),
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
