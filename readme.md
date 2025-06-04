# The Temple Project

## 项目说明
这是我的个人网站的仓库，作者为Huzz，项目的名称为The Temple Project

## 部署方法
使用podman-compose部署
```
````
```podman-compose up -d ```

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

## 部署结构
网站计划分为四个部分
- 前端 使用Leptos Rust框架
- 后端 使用Rust + axum框架
- 数据库 使用PostgreSQL，用来保存数据中心的链接地址
- 服务器 目前部署在腾讯云上，使用nginx做路由管理
- 数据中心 计划使用nginx做路由管理
项目使用podman进行容器化部署，使用podman-compose进行编排
仓库计划长期保留，一边后续迭代和迁移

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


  ```
