use crate::pages::home::Home;
use crate::utils::lang::{init_lang, persist_lang};
use leptos::prelude::*;
use leptos_router::components::*;
use leptos_router::path;

#[component]
pub fn app() -> impl IntoView {
    let initial_lang = init_lang();
    let lang_signal = RwSignal::new(initial_lang);

    provide_context(lang_signal);

    Effect::new({
        let lang_signal = lang_signal.clone();
        move |_| {
            let current = lang_signal.get();
            persist_lang(current);
        }
    });

    view! {

        <Router>
         <Routes fallback=||"not found">
         <Route path=path!("/") view=Home/>
         </Routes>
        </Router>
    }
}
