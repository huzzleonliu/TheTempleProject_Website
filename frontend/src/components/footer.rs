use crate::utils::lang::Lang;
use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    let lang = use_context::<RwSignal<Lang>>()
        .expect("Lang context should be provided in App");

    let hint_text = move || match lang.get() {
        Lang::Zh => "使用h/l进行前进和后退，使用j/k在PRESENT框中进行上下移动，使用 Shift+J/K 在DETAIL框中上下滚动",
        Lang::En => "Use h/l to navigate forward and backward, use j/k to move up and down in the PRESENT box, use Shift+J/K to scroll up and down in the DETAIL box",
    };

    view! {
        <div>
        <p class="text-sm text-gray-500">
            {hint_text}
        </p>
        </div>
    }
}
