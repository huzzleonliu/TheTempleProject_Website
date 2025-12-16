use crate::utils::lang::Lang;
use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    let lang = use_context::<RwSignal<Lang>>()
        .expect("Lang context should be provided in App");

    let hint_text = move || match lang.get() {
        Lang::Zh => "使用 hjkl 进行导航，使用 Shift+J/K 进行翻页",
        Lang::En => "Use hjkl to navigate, use Shift+J/K to scroll detail",
    };

    view! {
        <div>
        <p class="text-sm text-gray-500">
            {hint_text}
        </p>
        </div>
    }
}
