use leptos::prelude::*;

#[component]
pub fn OverviewA() -> impl IntoView {
    view! {
        <ul class="text-2xl text-gray-500">
            <li class="hover:text-white hover:bg-gray-800">"Once And Once Again"</li>
            <li class="hover:text-white hover:bg-gray-800">"The Temple Project"</li>
            <li class="hover:text-white hover:bg-gray-800">"With What Can I Hold You Back"</li>
            <li class="hover:text-white hover:bg-gray-800">"Way of Scholar"</li>
            <li>"-"</li>
            <li class="hover:text-white hover:bg-gray-800">"About the Aritst"</li>
        </ul>
    }
}
