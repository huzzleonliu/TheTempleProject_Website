use crate::{NodeKind, UiNode};
use leptos_icons::Icon;
use leptos::prelude::*;

pub fn button_style_builder(node: &UiNode, is_selected: bool) -> String {
    let base = "h-full text-left truncate text-base px-2";
    let type_class = match &node.kind {
        NodeKind::Directory => {
            if is_selected { "text-black font-bold bg-green-500 hover:text-black hover:bg-green-500 focus-within:bg-green-700" } 
            else { "text-green-500 hover:text-black hover:bg-gray-400 focus-within:bg-gray-500" }
        }
        NodeKind::Markdown => {
            if is_selected { "text-black font-bold bg-sky-500 hover:text-black hover:bg-sky-500 focus-within:bg-sky-700" } 
            else { "text-sky-500 hover:text-black hover:bg-gray-400 focus-within:bg-gray-500" }
        }
        NodeKind::Image => {
            if is_selected { "text-black font-bold bg-pink-500 hover:text-black hover:bg-pink-500 focus-within:bg-pink-700" } 
            else { "text-pink-500 hover:text-black hover:bg-gray-400 focus-within:bg-gray-500" }
        }
        NodeKind::Video => {
            if is_selected { "text-black font-bold bg-violet-500 hover:text-black hover:bg-violet-500 focus-within:bg-violet-700" } 
            else { "text-violet-500 hover:text-black hover:bg-gray-400 focus-within:bg-gray-500" }
        }
        NodeKind::Pdf => {
            if is_selected { "text-black font-bold bg-amber-500 hover:text-black hover:bg-amber-500 focus-within:bg-amber-700" } 
            else { "text-amber-500 hover:text-black hover:bg-gray-400 focus-within:bg-gray-500" }
        }
        NodeKind::Other => {
            if is_selected { "text-black font-bold bg-gray-500 hover:text-black hover:bg-gray-500 focus-within:bg-gray-700" } 
            else { "text-gray-500 hover:text-black hover:bg-gray-400 focus-within:bg-gray-500" }
        }
        NodeKind::Overview => {
            if is_selected { "text-black font-bold bg-gray-500 hover:text-black hover:bg-gray-500 focus-within:bg-gray-700" } 
            else { "text-gray-500 hover:text-black hover:bg-gray-400 focus-within:bg-gray-500" }
        }
    };
    format!("{base} {type_class} hover:text-black hover:bg-gray-400 focus-within:bg-gray-500")
}

#[component]
pub fn ButtonIconMatcher(kind: NodeKind) -> impl IntoView {
    match kind {
        NodeKind::Directory => view! { <Icon icon=icondata::LuFolder /> },
        NodeKind::Markdown => view! { <Icon icon=icondata::LuFileText /> },
        NodeKind::Image => view! { <Icon icon=icondata::LuImage /> },
        NodeKind::Video => view! { <Icon icon=icondata::LuVideo /> },
        NodeKind::Pdf => view! { <Icon icon=icondata::LuFileType /> },
        NodeKind::Other => view! { <Icon icon=icondata::LuFile /> },
        NodeKind::Overview => view! { <Icon icon=icondata::LuFiles /> },
    }
}