use leptos::prelude::*;

#[component]
pub fn Preview() -> impl IntoView {
    view! {
        // <div class="text-2xl">
        <div>
            <img src="http://43.131.27.176:8080/resource/3.jpg" class="box-border size-32" alt="http://43.131.27.176:8080/resource/resource/3.jpg"/>
            // <img src="http://localhost:8080/resource/3.jpg" class="box-border size-32" alt="http://43.131.27.176:8080/resource/resource/3.jpg"/>
            <p>"this is Preview"</p>
            <p>"this is Preview"</p>
            <p>"this is Preview"</p>
            <p>"this is Preview"</p>
        </div>
    }
}
