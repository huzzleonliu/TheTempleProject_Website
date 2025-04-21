use leptos::prelude::*;
use leptos_router::components::*;
use TheTempleProject::app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| 
        view! { 
            <Router base="/static">
                  <App/>
            </Router>
        }
    )
}
