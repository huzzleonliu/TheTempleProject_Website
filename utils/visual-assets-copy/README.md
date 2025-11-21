# Visual Assets 复制工具

## 工具说明

这是一个用于复制目录下所有 `visual_assets` 目录的命令行工具。工具会递归查找源目录下的所有 `visual_assets` 目录，并将它们复制到目标目录（默认 `~/Desktop`），同时保持原有的目录结构。

## 与 Detail 栏的配合
- 前端 Detail 栏会将 `visual_assets` 中的文件视为节点内容，因此复制脚本需要完整保留层级，确保节点仍能在前端以 `node` 形式渲染。
- 复制结果与 `node-generate-tool` 输出的 `file_path` 保持一致命名，便于在 Overview / Present / Detail 之间对齐节点标识。

## 功能特性

- **递归查找**: 自动递归查找源目录下的所有 `visual_assets` 目录
- **保持目录结构**: 复制时保持原有的目录层级结构
- **灵活的目标目录**: 支持自定义目标目录，默认复制到 `~/Desktop`
- **详细输出**: 显示查找和复制的进度信息

## 使用方法

### 编译
```bash
cd utils/visual-assets-copy
cargo build --release
```

### 基本用法
```bash
# 复制到默认目录 ~/Desktop
./target/release/visual-assets-copy /path/to/source

# 指定目标目录
./target/release/visual-assets-copy /path/to/source -o /path/to/output

# 使用相对路径
./target/release/visual-assets-copy ./data_root
```

## 使用示例

假设有以下目录结构：
```
data_root/
├── project1/
│   └── visual_assets/
│       ├── image1.jpg
│       └── description.md
├── project2/
│   ├── subproject/
│   │   └── visual_assets/
│   │       └── image2.png
│   └── visual_assets/
│       └── image3.jpg
└── other/
    └── files.txt
```

运行工具后，`~/Desktop` 目录下会生成：
```
~/Desktop/
├── project1/
│   └── visual_assets/
│       ├── image1.jpg
│       └── description.md
└── project2/
    ├── subproject/
    │   └── visual_assets/
    │       └── image2.png
    └── visual_assets/
        └── image3.jpg
```

## 工作原理

1. **递归查找**: 工具会递归遍历源目录下的所有子目录
2. **识别 visual_assets**: 当找到名为 `visual_assets` 的目录时，将其添加到复制列表
3. **计算相对路径**: 对于每个 `visual_assets` 目录，计算其相对于源目录的路径
4. **复制目录**: 在目标目录下重建相同的目录结构，并复制所有文件

## 注意事项

- 如果目标目录已存在同名文件或目录，会被覆盖
- 工具会跳过 `visual_assets` 目录内部的子目录（不会递归进入 `visual_assets` 内部查找）
- 支持 `~` 符号展开到用户主目录

## 技术栈

- **语言**: Rust
- **依赖**:
  - `clap`: 命令行参数解析
  - `anyhow`: 错误处理

