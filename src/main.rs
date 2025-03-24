use leptos::prelude::*;
use TheTempleProject::app::App;

fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(|| 
        view! { 
            <App/>
            <p class="text-red-500 p-4">"Hello, world!"</p> 
        }
    )
}
