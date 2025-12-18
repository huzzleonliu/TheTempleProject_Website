use super::HomeLogic;
use crate::components::body::{DetailPanel, OverviewColumn, PresentColumn};
use crate::components::footer::Footer;
use crate::components::header::Header;
use leptos::prelude::*;
use std::rc::Rc;
use crate::pages::home::logic::{navigate_to};
use crate::utils::keyboard::install_keyboard_listener;
use crate::utils::mouse::scroll_selected_into_view;

#[component]
pub fn DesktopLayout(logic: HomeLogic) -> impl IntoView {
    // --------- refs (scroll containers) ---------
    let present_scroll = NodeRef::<leptos::html::Div>::new();
    let detail_scroll = NodeRef::<leptos::html::Div>::new();

    // --------- actions (UI events) ---------
    let on_overview_enter = {
        let logic = logic.clone();
        UnsyncCallback::new(move |idx: usize| overview_enter(&logic, idx))
    };
    let on_present_select = {
        let logic = logic.clone();
        UnsyncCallback::new(move |idx: usize| present_select(&logic, idx))
    };
    let on_present_enter = {
        let logic = logic.clone();
        UnsyncCallback::new(move |idx: usize| present_enter(&logic, idx))
    };
    let on_detail_enter = {
        let logic = logic.clone();
        UnsyncCallback::new(move |idx: usize| detail_enter(&logic, idx))
    };

    // --------- keyboard wiring (desktop only) ---------
    {
        let present_scroll = present_scroll.clone();
        let detail_scroll = detail_scroll.clone();
        let move_selection = {
            let logic = logic.clone();
            let present_scroll = present_scroll.clone();
            Rc::new(move |delta: i32| {
                let nodes = logic.present_nodes.get_untracked();
                if nodes.is_empty() {
                    logic.current_selected_index.set(None);
                    scroll_selected_into_view(&present_scroll, None);
                    return;
                }
                let len = nodes.len() as i32;
                let cur = logic.current_selected_index.get_untracked().unwrap_or(0) as i32;
                let next = (cur + delta).clamp(0, len - 1) as usize;
                logic.current_selected_index.set(Some(next));
                scroll_selected_into_view(&present_scroll, Some(next));
            })
        };
        let enter_selection = {
            let logic = logic.clone();
            Rc::new(move || {
                if let Some(idx) = logic.current_selected_index.get_untracked() {
                    present_enter(&logic, idx);
                }
            })
        };
        let go_back = {
            let logic = logic.clone();
            Rc::new(move || {
                let current = logic.current_path.get_untracked();
                if let Some(path) = current {
                    if path.is_empty() {
                        return;
                    }
                    let parent = crate::utils::types::parent_path(&path)
                        .unwrap_or_else(|| crate::utils::types::ROOT_PATH.to_string());
                    let target_layer = if parent.is_empty() { None } else { Some(parent) };
                    navigate_to(&logic, target_layer, Some(0));
                }
            })
        };
        install_keyboard_listener(
            move_selection,
            enter_selection,
            go_back,
            detail_scroll,
            present_scroll,
        );
    }

    view! {
        <div class="flex flex-col h-screen">
            <div class="px-4 pt-4 pb-12 flex-shrink-0">
                <Header/>
            </div>
            <div class="grid grid-cols-10 grid-rows-1 flex-1 min-h-0 overflow-hidden items-start">
                <div class="col-span-2 overflow-y-auto px-4 pt-0">
                    <OverviewColumn
                        nodes=logic.overview_nodes
                        current_path=logic.current_path.read_only()
                        on_enter=on_overview_enter
                    />
                </div>
                <div class="col-span-3 h-full min-h-0 px-4 pt-0">
                    <PresentColumn
                        nodes=logic.present_nodes
                        selected_index=logic.current_selected_index.read_only()
                        on_select=on_present_select
                        on_enter=on_present_enter
                        scroll_container_ref=present_scroll
                    />
                </div>
                <div class="col-span-5 h-full min-h-0 px-4 pt-0">
                    <DetailPanel
                        nodes=logic.detail_nodes
                        detail_render_mode=logic.detail_render_mode
                        on_enter=on_detail_enter
                        scroll_container_ref=detail_scroll
                    />
                </div>
            </div>
            <div class="px-4 pb-4 flex-shrink-0">
                <Footer/>
            </div>
        </div>
    }
}

// ---------------- actions (UI events) ----------------
// NOTE: 按你的要求，这四个函数暂时放在 `home_desktop_layout.rs` 中，并且放在 `DesktopLayout` 外部。

pub fn present_select(logic: &HomeLogic, idx: usize) {
    logic.current_selected_index.set(Some(idx));
}

pub fn present_enter(logic: &HomeLogic, idx: usize) {
    let nodes = logic.present_nodes.get_untracked();
    let Some(node) = nodes.get(idx) else { return; };
    if !matches!(node.kind, crate::NodeKind::Directory) {
        return;
    }
    crate::pages::home::logic::navigate_to(logic, node.directory_path.clone(), None);
}

pub fn overview_enter(logic: &HomeLogic, idx: usize) {
    let nodes = logic.overview_nodes.get_untracked();
    let Some(node) = nodes.get(idx) else { return; };
    let Some(path) = node.directory_path.clone() else { return; };
    if path.is_empty() {
        crate::pages::home::logic::navigate_to(logic, None, None);
        return;
    }
    // 进入父层，并把 present 光标定位到当前点击的目录项（pos + 1，跳过 Overview）
    crate::pages::home::logic::navigate_to_parent_and_focus_child(logic, path);
}

pub fn detail_enter(logic: &HomeLogic, idx: usize) {
    let nodes = logic.detail_nodes.get_untracked();
    let Some(node) = nodes.get(idx) else { return; };

    // 资源不响应；只有目录可 enter（进入目录更新 current_path）
    if !matches!(node.kind, crate::NodeKind::Directory) {
        return;
    }
    crate::pages::home::logic::navigate_to(logic, node.directory_path.clone(), None);
}
