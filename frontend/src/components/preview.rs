use leptos::prelude::*;

#[component]
pub fn Preview() -> impl IntoView {
    view! {
        // <div class="text-2xl">
        <div>
        // <img src="https://the-temple-project-1321767328.cos.eu-frankfurt.myqcloud.com/2.jpg" class="box-border size-32" alt="not found"/>
        // <img src="https://the-temple-project-1321767328.cos.eu-frankfurt.myqcloud.com/3.jpg" class="box-border size-32" alt="not found"/>
            <img src="/E2.jpg" class="box-border size-32" alt="can not get"/>
            <img src="/E1.jpeg" class="box-border size-32" alt="not found"/>
            <img src="/images/E12.jpeg" class="box-border size-32" alt="not found"/>
            <p>"this is Preview"</p>
            <p>"this is Preview"</p>
            <p>"this is Preview"</p>
            <p>"this is Preview"</p>
        </div>
    }
}
