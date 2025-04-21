use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_net::http::Request;

#[component]
pub fn ButtonGet() -> impl IntoView {

    let print = move |_|{
        spawn_local(async move {
            // match
            Request::get("http://localhost.:8080/test")
                .send()
                .await
                .unwrap();
            // {
            //     Ok(response) => {
            //         let text = response.text().await.unwrap_or_else(|_| "Failed to get text".to_string());
            //         set_text.set(text);
            //     },
            //     Err(err) => {
            //         console::log_1(&format!("Error: {:?}", err).into());
            //     }
            // }
        });

    };
    view! {
        <div class="flex flex-col items-center justify-center h-screen">
            <button on:click={print} class="bg-blue-500 text-white font-bold py-2 px-4 rounded hover:bg-blue-700">
                on_click 
            </button>
        </div>
    }
}
