use gloo_net::http::Request;
use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn TextGet() -> impl IntoView {
    // 创建响应式信号存储请求结果
    let (response, set_response) = signal("Click to fetch".to_string());

    Effect::new(move |_| {
        spawn_local(async move {
            // 发送请求前显示加载状态
            set_response.set("Loading...".into());

            match Request::get("/api/text").send().await {
                Ok(resp) => {
                    // 读取响应文本
                    match resp.text().await {
                        Ok(text) => set_response.set(text),
                        Err(e) => set_response.set(format!("Read error: {e}")),
                    }
                }
                Err(e) => set_response.set(format!("Request failed: {e}")),
            }
        });
    });

    view! {
        <div class="flex flex-col items-center justify-center h-screen">
            <p>
                {response}
                "Click the button to fetch text from the server."
            </p>
        </div>
    }
}
