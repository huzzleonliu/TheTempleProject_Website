use leptos::prelude::*;
use leptos::task::spawn_local;
use gloo_net::http::Request;

#[component]
pub fn ButtonGet() -> impl IntoView {
    // 创建响应式信号存储请求结果
    let (response, set_response) = create_signal("Click to fetch".to_string());

    let print = move |_| {
        spawn_local(async move {
            // 发送请求前显示加载状态
            set_response.set("Loading...".into());
            
            match Request::get("http://43.131.27.176:8081/print")
                .send()
                .await 
            {
                Ok(resp) => {
                    // 读取响应文本
                    match resp.text().await {
                        Ok(text) => set_response.set(text),
                        Err(e) => set_response.set(format!("Read error: {e}"))
                    }
                },
                Err(e) => set_response.set(format!("Request failed: {e}"))
            }
        });
    };

    view! {
        <div class="flex flex-col items-center justify-center h-screen">
            <button 
                on:click=print 
                class="bg-blue-500 text-white font-bold py-2 px-4 rounded hover:bg-blue-700"
            >
                {response}  // 动态显示响应内容
            </button>
        </div>
    }
}
