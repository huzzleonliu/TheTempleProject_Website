use leptos::prelude::*;
use leptos::task::spawn_local;
use web_sys::console;

use crate::{DirectoryNode, ItemContext, NavigationSignals, DataSignals};
use crate::api::{get_child_directories, get_root_directories};
use crate::components::item::ItemComponent;

/// API 响应数据结构
// 类型由 crate::types 提供

/// OverviewB 组件：显示当前层级的目录列表
/// 
/// # 功能
/// - 显示当前层级的目录列表
/// - 支持鼠标点击导航
/// - 支持键盘导航（j/k/l/h 键）
/// - 支持 Shift+J/K 滚动 Preview
/// - 自动聚焦以接收键盘事件
/// 
/// # 参数
/// - `overview_b_directories`: 当前层级的目录路径列表（字符串）
/// - `set_overview_b_directories`: 设置当前层级目录列表的函数
/// - `set_overview_a_directories`: 设置 OverviewA 目录列表的函数
/// - `selected_path`: 当前选中的路径（用于高亮显示）
/// - `set_selected_path`: 设置选中路径的函数
/// - `set_preview_path`: 设置 Preview 显示路径的函数
/// - `selected_index`: 当前选中的索引（用于键盘导航）
/// - `set_selected_index`: 设置选中索引的函数
/// - `overview_a_directories`: OverviewA 的目录列表（用于返回导航）
/// - `overview_a_selected_path`: OverviewA 中高亮的路径（用于返回导航）
/// - `set_overview_a_selected_path`: 设置 OverviewA 高亮路径的函数
/// - `directories`: 当前目录的完整信息（从外部传入，供全局键盘事件使用）
/// - `set_directories`: 设置目录信息的函数
/// - `preview_scroll_ref`: Preview 滚动容器的引用（用于 Shift+J/K 滚动）
#[component]
pub fn OverviewB(
    overview_b_directories: ReadSignal<Vec<String>>,
    set_overview_b_directories: WriteSignal<Vec<String>>,
    set_overview_a_directories: WriteSignal<Vec<String>>,
    selected_path: ReadSignal<Option<String>>,
    set_selected_path: WriteSignal<Option<String>>,
    set_preview_path: WriteSignal<Option<String>>,
    selected_index: ReadSignal<Option<usize>>,
    set_selected_index: WriteSignal<Option<usize>>,
    overview_a_directories: ReadSignal<Vec<String>>,
    overview_a_selected_path: ReadSignal<Option<String>>,
    set_overview_a_selected_path: WriteSignal<Option<String>>,
    directories: ReadSignal<Vec<DirectoryNode>>,
    set_directories: WriteSignal<Vec<DirectoryNode>>,
    preview_scroll_ref: NodeRef<leptos::html::Div>,
) -> impl IntoView {
    // 加载状态
    let (loading, set_loading) = signal(false);
    // 错误信息
    let (error, set_error) = signal::<Option<String>>(None);

    // 当 directories 改变时，如果索引未设置或超出范围，则重置为 0
    // 注意：这个 effect 不应该覆盖已经正确设置的索引（比如返回父级节点时）
    create_effect(move |_| {
        let dirs = directories.get();
        if !dirs.is_empty() {
            // 如果当前索引超出范围或未设置，则重置为 0
            if let Some(current_idx) = selected_index.get() {
                if current_idx >= dirs.len() {
                    console::log_2(&"[OverviewB] 索引超出范围，重置为 0。当前索引:".into(), &current_idx.into());
                    console::log_2(&"[OverviewB] 列表长度:".into(), &dirs.len().into());
                    set_selected_index.set(Some(0));
                }
            } else {
                console::log_1(&"[OverviewB] 索引未设置，重置为 0".into());
                set_selected_index.set(Some(0));
            }
        }
    });

    // 当选中索引改变时，更新 Preview 显示的内容
    // 注意：只有当 directories 不为空且与 overview_b_directories 匹配时才更新 Preview
    create_effect(move |_| {
        if let Some(index) = selected_index.get() {
            let dirs = directories.get();
            let dir_paths = overview_b_directories.get();
            
            // 检查 directories 是否为空，或者是否与 overview_b_directories 匹配
            // 如果不匹配，说明 directories 还在加载中，不应该更新 Preview
            if dirs.is_empty() {
                console::log_1(&"[OverviewB] directories 为空，不更新 Preview".into());
                return;
            }
            
            // 检查 directories 是否与 overview_b_directories 匹配
            let dirs_paths: Vec<String> = dirs.iter().map(|d| d.path.clone()).collect();
            if dirs_paths != dir_paths {
                console::log_1(&"[OverviewB] directories 与 overview_b_directories 不匹配，不更新 Preview".into());
                return;
            }
            
            if let Some(dir) = dirs.get(index) {
                // 设置选中的路径（用于高亮显示）
                set_selected_path.set(Some(dir.path.clone()));
                
                // 只有当节点有子节点时才更新 Preview
                if dir.has_subnodes {
                    console::log_2(&"[OverviewB] 选中索引改变，更新 Preview:".into(), &dir.path.clone().into());
                    set_preview_path.set(Some(dir.path.clone()));
                } else {
                    console::log_2(&"[OverviewB] 选中索引改变，节点无子节点:".into(), &dir.path.clone().into());
                    set_preview_path.set(None);
                }
            }
        }
    });

    // 键盘事件处理已移至 home.rs 的全局监听器

    // 当 overview_b_directories 改变时，从 API 加载对应的完整目录信息
    // 这个 effect 负责将路径列表转换为包含完整信息的 DirectoryNode 列表
    create_effect(move |_| {
        let dir_paths = overview_b_directories.get();
        let dir_paths_clone = dir_paths.clone();
        
        // 先清空 directories，避免旧的 directories 触发 Preview 更新
        set_directories.set(Vec::new());
        
        spawn_local(async move {
            if dir_paths_clone.is_empty() {
                // 初始加载一级目录
                console::log_1(&"[OverviewB] 初始加载一级目录".into());
                set_loading.set(true);
                set_error.set(None);

                match get_root_directories().await {
                    Ok(data) => {
                        console::log_2(&"[OverviewB] 加载根目录成功，数量:".into(), &data.len().into());
                        set_directories.set(data.clone());
                        
                        // 设置第一个有子节点的目录用于 Preview
                        if let Some(first_dir) = data.iter().find(|d| d.has_subnodes) {
                            console::log_2(&"[OverviewB] 设置 Preview 路径:".into(), &first_dir.path.clone().into());
                            set_preview_path.set(Some(first_dir.path.clone()));
                        } else {
                            console::log_1(&"[OverviewB] 没有找到有子节点的目录".into());
                            set_preview_path.set(None);
                        }
                        set_loading.set(false);
                    }
                    Err(e) => {
                        let msg = format!("{e}");
                        console::log_2(&"[OverviewB] 请求失败:".into(), &msg.clone().into());
                        set_error.set(Some(format!("请求失败: {msg}")));
                        set_loading.set(false);
                    }
                }
            } else {
                // 根据路径列表，获取父路径，然后加载兄弟节点
                if let Some(first_path) = dir_paths_clone.first() {
                    console::log_2(&"[OverviewB] 加载目录信息，第一个路径:".into(), &first_path.clone().into());
                    // 获取父路径
                    let parent_path = if first_path.contains('.') {
                        let parts: Vec<&str> = first_path.split('.').collect();
                        if parts.len() > 1 {
                            Some(parts[0..parts.len()-1].join("."))
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    
                    set_loading.set(true);
                    set_error.set(None);
                    
                    let result = if let Some(p) = parent_path {
                        get_child_directories(&p).await
                    } else {
                        get_root_directories().await
                    };
                    match result {
                        Ok(data) => {
                            console::log_2(&"[OverviewB] 加载目录信息成功，数量:".into(), &data.len().into());
                            set_directories.set(data.clone());
                            
                            // 如果 selected_index 已设置，立即根据 selected_index 更新 Preview
                            // 这样可以确保在进入新节点时，Preview 能第一时间刷新
                            if let Some(index) = selected_index.get() {
                                if let Some(dir) = data.get(index) {
                                    if dir.has_subnodes {
                                        console::log_2(&"[OverviewB] directories 加载完成，根据 selected_index 更新 Preview:".into(), &dir.path.clone().into());
                                        set_preview_path.set(Some(dir.path.clone()));
                                    } else {
                                        console::log_2(&"[OverviewB] directories 加载完成，节点无子节点:".into(), &dir.path.clone().into());
                                        set_preview_path.set(None);
                                    }
                                } else {
                                    console::log_2(&"[OverviewB] selected_index 超出范围，重置 Preview".into(), &index.into());
                                    set_preview_path.set(None);
                                }
                            } else {
                                // 如果 selected_index 未设置，设置第一个有子节点的目录用于 Preview
                                if let Some(first_dir) = data.iter().find(|d| d.has_subnodes) {
                                    console::log_2(&"[OverviewB] 设置 Preview 路径:".into(), &first_dir.path.clone().into());
                                    set_preview_path.set(Some(first_dir.path.clone()));
                                } else {
                                    console::log_1(&"[OverviewB] 没有找到有子节点的目录".into());
                                    set_preview_path.set(None);
                                }
                            }
                            set_loading.set(false);
                        }
                        Err(e) => {
                            let msg = format!("{e}");
                            console::log_2(&"[OverviewB] 请求失败:".into(), &msg.clone().into());
                            set_error.set(Some(format!("请求失败: {msg}")));
                            set_loading.set(false);
                        }
                    }
                }
            }
        });
    });

    // 自动聚焦逻辑已移除，键盘事件现在在 home.rs 中全局处理

    view! {
        <ul 
            class="text-2xl text-gray-500 outline-none"
        >
            <Show
                when=move || loading.get()
                fallback=move || {
                    view! {
                        <Show
                            when=move || error.get().is_some()
                            fallback=move || {
                                view! {
                                    <For
                                        each=move || directories.get()
                                        key=|dir| dir.path.clone()
                                        children=move |dir: DirectoryNode| {
                                            let path_clone = dir.path.clone();
                                            
                                            // 创建导航信号
                                            let nav = NavigationSignals {
                                                set_overview_a_directories,
                                                set_overview_a_selected_path,
                                                set_overview_b_directories,
                                                set_preview_path,
                                                set_selected_path,
                                                set_selected_index,
                                            };
                                            
                                            // 创建数据信号
                                            let data = DataSignals {
                                                directories,
                                            };
                                            
                                            // 创建 ItemContext
                                            let context = ItemContext::from_node(
                                                dir,
                                                nav,
                                                data,
                                                false, // OverviewB 中不是 OverviewA
                                            );
                                            
                                            // 使用 Memo 计算是否选中（避免闭包类型问题）
                                            let is_selected = Memo::new(move |_| {
                                                if let Some(index) = selected_index.get() {
                                                    let dirs = directories.get();
                                                    if let Some(dir_idx) = dirs.iter().position(|d| d.path == path_clone) {
                                                        index == dir_idx
                                                    } else {
                                                        false
                                                    }
                                                } else {
                                                    selected_path.get().as_ref() == Some(&path_clone)
                                                }
                                            });
                                            
                                            view! {
                                                <ItemComponent
                                                    context=context
                                                    is_selected=is_selected
                                                />
                                            }
                                        }
                                    />
                                }
                            }
                        >
                            <li class="text-red-500">{move || error.get().unwrap_or_else(|| "未知错误".to_string())}</li>
                        </Show>
                    }
                }
            >
                <li>"加载中..."</li>
            </Show>
        </ul>
    }
}
