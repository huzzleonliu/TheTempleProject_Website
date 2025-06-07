use leptos::prelude::*;

#[component]
pub fn Preview() -> impl IntoView {
    view! {
        // <div class="text-2xl">
        <div>
            <img src="http://tp_resource:80/resource/3.jpg" class="box-border size-32" alt="http://tp_resource.:80/3.jpg"/>
            <img src="http://43.131.27.176:8084/resource/3.jpg" class="box-border size-32" alt="http://43.131.27.176:8084/resource/3.jpg"/>
            <img src="http://localhost:8084/resource/3.jpg" class="box-border size-32" alt="http://localhost:8084/resource/3.jpg"/>
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
