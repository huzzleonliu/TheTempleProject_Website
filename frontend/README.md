# TheTempleProject 前端（Leptos）

## 项目概览
- 使用 **Leptos 0.8** 构建的单页应用，编译为 WebAssembly，并通过 `wasm-bindgen` 与浏览器交互。
- 桌面端采用三栏布局：左侧 **Overview 栏**、中间 **Present 栏**、右侧 **Detail 栏**，协同展示层级目录与节点内容。
- 数据由后端 Axum API 提供，前端使用基于路径的缓存减少重复请求。

## 组件结构
| 组件 | 作用 | 说明 |
| --- | --- | --- |
| `pages::home::Home` | 页面入口 | 维护全局状态、处理 API 请求、注册键盘事件 |
| `components::overview_column` | Overview 栏 | 展示当前层级的父节点，点击可回退并高亮目标节点 |
| `components::present_column` | Present 栏 | 展示 `current_path` 的直接子节点，支持单击选中、双击进入 |
| `components::detail_panel` | Detail 栏 | 根据选中节点加载子内容或文件详情，支持 Markdown / PDF / 媒体 |
| `components::keyboard_handlers` | 键盘适配层 | 将 `h/j/k/l`、`Shift+J/K` 等快捷键映射到导航/滚动逻辑 |

## 状态与缓存
- `path_cache: RwSignal<HashMap<String, Vec<DirectoryNode>>>`
  - 记录每个路径下的直接子节点，确保同一路径只从后端请求一次。
- `current_path: RwSignal<Option<String>>`
  - `None` 表示根层级，`Some(path)` 表示当前聚焦的目录。
- `selected_index: RwSignal<Option<usize>>`
  - 追踪 Present 栏中的高亮行；导航时通过 Memo 自动同步。
- `detail_path, detail_items, detail_loading, detail_error`
  - 控制 Detail 栏的数据加载、渲染与错误状态。
- `Memo<Vec<DirectoryNode>>`
  - `present_nodes`：当前层级的所有子节点（供 Present 栏使用）。
  - `overview_nodes`：当前层级所在父路径下的节点列表，用于 Overview 栏。
  - `overview_highlight`：需要在 Overview 栏中高亮的路径。

## 导航流程
1. **进入/回退** 调用 `navigate_to(target_path, preferred_index)`：
   - 确保目标路径及祖先已缓存（必要时向后端请求）。
   - 根据 `preferred_index` 或默认策略选择高亮行。
   - 更新预览区域（若子节点仍存在）。
2. **左栏点击**：解析被点击节点的父路径，计算其在父层级中的索引，再调用 `navigate_to`。
3. **键盘快捷键**：
   - `j / k`：在当前层级内移动选中行。
   - `l`：进入当前选中节点（若存在子节点）。
   - `h`：回退到父级目录，并保持原节点高亮。
   - `Shift + J / K`：在 Preview 中滚动。
4. **根层级体验**：当处于根层级时，Overview 栏会展示一个虚拟的 `/` 节点，帮助用户理解层级起点。

## API 交互
- `api::get_root_directories()`：获取根节点列表。
- `api::get_child_directories(path)`：获取指定路径的直接子节点。
- `ensure_children(path)`：缓存薄层包装，判断是否需要真正发起请求。
- `ensure_path_and_ancestors(path)`：预加载路径及其祖先层级，保障回退和面包屑能即时展示。

## 开发与调试
1. 安装 Rust 最新稳定版本，并添加 `wasm32-unknown-unknown` 目标：
   ```bash
   rustup target add wasm32-unknown-unknown
   ```
2. 在 `frontend` 目录执行编译检查：
   ```bash
   cargo check
   ```
3. 推荐使用 `trunk` 或 `leptos` 官方模板提供的开发服务器（具体命令视工作流而定）。
4. 调试时可关注浏览器控制台输出（`web_sys::console::log`）。

## 设计要点
- 所有信号与 memo 均使用 Leptos 新 API（`RwSignal::new`、`Memo::new`、`Effect::new`）避免弃用警告。
- 鼓励保持 UI 探索体验一致：鼠标、键盘、缓存更新逻辑共享同一导航入口 `navigate_to`。
- 通过丰富注释解释关键分支，便于日后维护与二次开发。
