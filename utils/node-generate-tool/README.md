# 目录节点生成工具 (node-generate-tool)

## 工具说明

这是一个用于为网站数据库生成数据节点表的命令行工具。该工具会扫描指定的目录结构，为每个目录节点生成包含布尔属性的 CSV 文件，可直接导入 PostgreSQL 数据库。

## 功能特性

### 1. 目录扫描
- 递归扫描指定根目录下的所有子目录
- 支持通过 `fileignore` 文件（gitignore 语法）跳过不需要的目录
- 忽略文件会从以下三个位置按顺序合并：
  1. 程序源码目录（`utils/node-generate-tool/`）
  2. 可执行程序所在目录（`target/release/`）
  3. 被扫描的根目录

### 2. 节点属性检查
对每个目录节点，工具会检查以下属性：

- **has_layout**: 节点内是否有 `layout.md` 文件（说明节点内有明确的排版内容）
- **has_visual_assets**: 节点内是否有 `visual_assets` 目录（说明节点内有图文资源）
- **has_text**: `visual_assets` 目录下是否有 markdown 文件（说明节点内有文字说明）
- **has_images**: `visual_assets` 目录下是否有图片文件（说明节点内有图片）
- **has_subnodes**: 节点内是否有其他目录（排除 `visual_assets` 和 `project_archive`，说明节点内有子节点）

### 3. CSV 输出格式
生成的 CSV 文件包含以下列：
- `path`: 目录路径（ltree 格式，点号分隔，如 `a.b.c`）
- `has_layout`: 布尔值（true/false）
- `has_visual_assets`: 布尔值（true/false）
- `has_text`: 布尔值（true/false）
- `has_images`: 布尔值（true/false）
- `has_subnodes`: 布尔值（true/false）

**注意：** 路径使用 ltree 格式（点号分隔），目录名中的特殊字符会被替换为下划线，以符合 ltree 标签要求（只能包含字母、数字、下划线）。

## 使用方法

### 编译
```bash
cd utils/node-generate-tool
cargo build --release
```

### 基本用法
```bash
# 扫描目录并生成 CSV（默认输出到当前目录的 directories.csv）
./target/release/node-generate-tool /path/to/data/root

# 指定输出文件
./target/release/node-generate-tool /path/to/data/root -o nodes.csv

# 指定忽略文件名（默认 fileignore）
./target/release/node-generate-tool /path/to/data/root --ignore-file custom.ignore
```

## 工作流程

1. **更新数据中心**: 在数据中心目录中添加、修改或删除目录节点
2. **运行工具**: 执行工具扫描数据中心目录，生成最新的节点表 CSV
3. **导入数据库**: 将生成的 CSV 文件导入 PostgreSQL，替换旧的节点表
4. **后端查询**: 后端根据节点表的布尔属性，决定从数据中心获取哪些文件

## 数据库集成

### 启用 ltree 扩展
```sql
CREATE EXTENSION IF NOT EXISTS ltree;
```

### 创建表结构
```sql
CREATE TABLE directory_nodes (
    path ltree PRIMARY KEY,
    has_layout BOOLEAN NOT NULL,
    has_visual_assets BOOLEAN NOT NULL,
    has_text BOOLEAN NOT NULL,
    has_images BOOLEAN NOT NULL,
    has_subnodes BOOLEAN NOT NULL
);

-- 创建索引以优化查询性能
CREATE INDEX idx_path_gist ON directory_nodes USING GIST (path);
CREATE INDEX idx_path_btree ON directory_nodes USING BTREE (path);
```

### 导入 CSV
```sql
COPY directory_nodes FROM '/path/to/nodes.csv' WITH (FORMAT csv, HEADER true);
```

### 后端查询示例

#### 基本查询
```sql
-- 查找有排版内容的节点
SELECT path FROM directory_nodes WHERE has_layout = true;

-- 查找有图片的节点
SELECT path FROM directory_nodes WHERE has_images = true;

-- 查找有完整图文资源的节点
SELECT path FROM directory_nodes 
WHERE has_visual_assets = true AND has_text = true AND has_images = true;
```

#### ltree 树形查询（强大功能）
```sql
-- 查找某个节点的所有子节点（后代）
SELECT * FROM directory_nodes 
WHERE path <@ '1_OnceAndOnceAgain.handmadeBook';

-- 查找某个节点的所有父节点（祖先）
SELECT * FROM directory_nodes 
WHERE path @> '1_OnceAndOnceAgain.handmadeBook.Book';

-- 查找某个节点的直接子节点
SELECT * FROM directory_nodes 
WHERE path ~ '1_OnceAndOnceAgain.handmadeBook.*{1}';

-- 查找某个节点的所有兄弟节点（同级）
SELECT * FROM directory_nodes 
WHERE subpath(path, 0, -1) = '1_OnceAndOnceAgain.handmadeBook';

-- 查找路径深度为 2 的所有节点
SELECT * FROM directory_nodes 
WHERE nlevel(path) = 2;

-- 查找某个节点的路径长度
SELECT path, nlevel(path) as depth FROM directory_nodes;

-- 查找某个节点的父路径
SELECT path, subpath(path, 0, -1) as parent_path FROM directory_nodes;
```

#### 组合查询示例
```sql
-- 查找某个节点下所有有图片的子节点
SELECT * FROM directory_nodes 
WHERE path <@ '1_OnceAndOnceAgain.handmadeBook' 
  AND has_images = true;

-- 查找某个节点的所有有子节点的后代
SELECT * FROM directory_nodes 
WHERE path <@ '1_OnceAndOnceAgain' 
  AND has_subnodes = true;
```

## 文件结构要求

工具假设目录节点遵循以下结构：

```
data_root/
├── node1/
│   ├── layout.md              # 可选：排版内容
│   ├── visual_assets/         # 可选：图文资源目录
│   │   ├── description.md     # 可选：文字说明
│   │   └── image.jpg          # 可选：图片文件
│   └── subnode/               # 可选：子节点
└── node2/
    └── ...
```

## 忽略文件配置

在以下任一位置创建 `fileignore` 文件（使用 gitignore 语法）：

1. `utils/node-generate-tool/fileignore` - 程序源码目录
2. `target/release/fileignore` - 可执行程序目录
3. `<扫描根目录>/fileignore` - 被扫描的根目录

示例 `fileignore` 内容：
```
# 忽略构建目录
target/
node_modules/

# 忽略隐藏目录
.git/
.vscode/

# 忽略特定目录
**/dist/
**/build/
```

## 技术栈

- **语言**: Rust
- **依赖**:
  - `clap`: 命令行参数解析
  - `ignore`: 目录遍历和 gitignore 语法支持
  - `anyhow`: 错误处理
  - `pathdiff`: 路径计算

## 注意事项

- 工具只扫描目录，不扫描文件
- 路径使用逗号分隔，便于在数据库中存储和查询
- 布尔值输出为 `true`/`false`，PostgreSQL 可直接识别
- 图片文件支持常见格式：jpg, jpeg, png, gif, webp, svg, bmp, ico
- 文字文件识别 `.md` 和 `.markdown` 扩展名

