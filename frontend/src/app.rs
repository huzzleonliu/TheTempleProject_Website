use leptos::prelude::*;
use crate::pages::home::Home;
use leptos_router::components::*;
use leptos_router::path;
// use leptos_meta::*;

#[component]
pub fn app() -> impl IntoView {
    // provide_meta_context();
    view! {
        // <main>
        //     // <Meta charset="utf-8"/>
        // </main>
        <Router>
         <Routes fallback=||"not found">
         <Route path=path!("/") view=Home/>
         </Routes>
        </Router>
    }
}
