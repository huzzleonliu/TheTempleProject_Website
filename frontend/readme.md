# 前端

## 技术实现
目前计划采用Lepos框架

## 设计灵感
结构设计参考ranger或者yazi的tui设计
使用hjkl来进行目录的切换
figma的设计稿可以在这里查看 [Figma](https://www.figma.com/design/8KmXZOFKxiYKAJgizWsDfz/Prototype?m=auto&t=eWtWX2qXGAucgd9u-1)
## 任务计划
- []需要针对每一个项目设计图标
- []每次渲染页面，需要将每一行字作为一个元素进行渲染
- []需要设计一个帮助页面提示用户使用方法

## 当前布局逻辑
- 桌面端统一使用 Overview / Present / Detail 三栏，所有节点变量以 `node` 命名，避免与目录概念混淆。
- Overview 栏负责展示父级节点，Present 栏负责当前层级节点列表，Detail 栏负责渲染被点击节点的内容或其子节点列表。
- 移动端仅保留 Detail 内容区，通过点击列表条目进入下一层级，返回按钮始终允许回到上级。
- Detail 支持 Markdown、PDF、图像与视频内容，同时目录节点会以可点击的列表项列在 Detail 中，保持桌面 / 移动行为一致。

