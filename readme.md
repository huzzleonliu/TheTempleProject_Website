# The Temple Project

## 项目简介
The Temple Project 是一个以 Rust 为核心的全栈网站，目标是构建个人创作与知识库的统一入口。
项目分为nginx网关、前端、后端、数据库与静态资源服务四个部分，通过容器化方式部署，支持后续的长期扩展与维护。

## 技术栈总览
- **前端**：Leptos + WebAssembly，三栏式导航界面，支持键盘快捷键与目录缓存。
- **后端**：Axum + Tokio，提供目录结构 API。
- **数据库**：PostgreSQL（启用 `ltree` 扩展）存储目录层级信息。
- **资源库**：使用Nginx管理资源的访问。
- **网关**：Nginx 作为反向代理与静态资源分发。
## 部署方法
使用podman-compose部署`podman-compose up -d `
将文件资源拷贝进./resource/resource目录
使用utils/refresh_resources.sh装载数据
通过43.131.27.176:8080访问网站

## 资源与数据库自动化
为方便批量更新资源与目录节点，项目提供以下流程：

1. **准备宿主目录**
   - 将真实资源放在宿主机 `/resource/`，`docker-compose.yml` 会将其 bind mount 到 resource 容器的 `/resource`。
   - `database/import_exchange/` 用于 CSV 交换，可由脚本自动写入、数据库容器自动读取。

2. **一键刷新与导入**
   ```bash
   COMPOSE_CMD="podman-compose" ./scripts/refresh_resources.sh ./resource/resource
   ```
   - 未传参时默认扫描 `/resource`（适合容器内运行）；传参可指定其它目录。
   - 脚本会调用 `utils/node-generate-tool` 生成 `node.csv`/`visual_assets.csv`，随后自动执行 `database/scripts/import_csv.sh`，在 `tp_database` 容器内通过 `psql` 完成 `TRUNCATE + \copy`。

3. **注意事项**
   - 确保 `/resource/` 与 `database/import_exchange/` 对当前用户可写。
   - 若使用 `docker-compose`，运行脚本时设置 `COMPOSE_CMD="docker-compose"`。
   - resource 目录可为空，Nginx 仍会启动，但访问将返回 404。只需在上传资源后重新执行脚本即可。


## 前端导航逻辑
前端采用“基于路径的结构驱动”方案，所有导航行为都依赖节点的完整路径进行推导：
1. `PresentColumn` 展示当前路径的直接子节点；
2. `OverviewColumn` 展示当前节点所在父层级，并高亮当前节点；
3. `DetailPanel` 根据 `PresentColumn` 的选中项异步加载更深一层的内容；
4. 所有鼠标和键盘操作汇聚到 `navigate_to(target_path, preferred_index)`，统一管理缓存、选中项和预览更新；
5. 根层级会插入一个虚拟的 `/` 节点，帮助用户理解层级起点。

### 快捷键速览
| 快捷键 | 功能 |
| --- | --- |
| `j / k` | 在 `PresentColumn` 中上下移动选中行 |
| `l` | 进入当前选中节点（若存在子节点） |
| `h` | 回退到父级目录，并高亮原节点 |
| `Shift + J / Shift + K` | 在 `DetailPanel` 中滚动 |

## 项目计划
第一阶段这个项目预计将包含以下几个功能：
- 作为个人网站包含我长期以来的创作作品展示
- 作品说明
- 作品价格
- 身份信息
- 个人简历
- 证书文件
第二阶段这个项目将在个人网站的基础上建设成我的数据库的入口，用来管理：
- 论文管理系统（已读论文，待读论文，正在读论文）
- 自己写的文章
- 思维导图
- 账号密码
- 会员信息及网站
- 钱包地址
- 日常开销
- 个人照片
- 日程信息
第三阶段进行项目优化
- 多设备同步
- 商用API及购买平台

## 后端与数据库
- 后端以 Axum 提供 RESTful API，包括根节点与指定路径子节点查询。
- 数据库通过 `ltree` 存储目录树，支持高效的祖先/后代查询。
- 后端接口返回统一的 `DirectoryNode` 数据结构（含路径、显示名称、是否存在子节点）。

## 目录结构说明
```
TheTempleProject_Website/
├── frontend/                # Leptos 前端
│   ├── src/
│   │   ├── pages/home.rs    # 三栏页面核心逻辑
│   │   ├── components/      # UI 组件（Header、Body: Overview/Present/Detail、Footer 等）
│   │   └── api.rs           # 前端 API 调用封装
│   └── README.md            # 前端功能说明
├── backend/                 #（预留）Axum 服务
├── database/                # 数据库脚本与 README
├── readme.md                # 当前文件
└── podman-compose.yaml      # 容器编排配置
```

## 部署流程
1. 安装 Podman 与 podman-compose。
2. 在项目根目录执行：
   ```bash
   podman-compose up -d
   ```
3. 默认暴露端口：
   - 前端：8081
   - 后端：8082
   - 数据库：5432（通过 8083 暴露）
   - 入口 Nginx：8080
4. 访问 `http://<服务器地址>:8080` 查看站点。

## 网站流程
1. 用户访问网站
2. nginx接收请求并转发到后端服务
3. 后端服务处理请求，查询数据库获取请求的节点信息
4. 后端服务将节点情况返回前端
5. 前端处理返回信息，并且根据返回信息向数据中心服务请求数据
6. 数据中心返回数据
7. 前端解析数据，渲染页面，展示数据和图片

## 节点情况
目前节点存在这样几种情况
- 节点内有其它目录的话，说明节点内有子节点
- 节点内有“layout.md”的文件的话，说明节点内有明确的排版内容
- 节点内有“visual_assets”的目录的话，说明节点内有图文
  - visual_assets目录下有markdown文件的话，说明节点内有文字说明
  - visual_assets目录下有图片文件的话，说明节点内有图片


## TIPS
- 在新机器上部署时注意podman的registry.conf配置
  [etc/containers/registries.conf]
  ```
  unqualified-search-registries = ["docker.io", "quay.io", "registry.access.redhat.com"]
  [[registry]]
  prefix = "docker.io"  # 镜像前缀（如 `docker.io/nginx`）
  location = "docker.io"  # 实际拉取地址
  insecure = false       # 是否允许 HTTP（默认 false，推荐 HTTPS）
```
```



