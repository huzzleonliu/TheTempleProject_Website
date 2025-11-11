# 目录节点生成工具 (node-generate-tool)

## 工具说明

这是一个用于为网站数据库生成“目录节点表”和“资源文件表”的命令行工具。工具包含两种模式：
- 节点扫描（默认或 --node）：扫描目录节点，输出节点属性 CSV
- 资源扫描（--visual-scan）：扫描各节点中的 visual_assets/ 下的“直接文件”，输出文件路径 CSV（不递归子目录）
两种 CSV 均可直接导入 PostgreSQL（推荐使用 ltree 扩展）。

## 功能特性

### 1. 统一忽略策略
- 支持通过 `fileignore` 文件（gitignore 语法）跳过不需要的路径
- 忽略文件会从以下三个位置按顺序合并（后规则优先）：
  1. 程序源码目录（`utils/node-generate-tool/`）
  2. 可执行程序所在目录（`target/release/`）
  3. 被扫描的根目录
- 在“资源扫描模式”（`--visual-scan`）下，工具会附加“反忽略”规则强制包含 `visual_assets/`（覆盖 fileignore 中的忽略），确保可以正确扫描。

### 2. 节点扫描（--node 或默认）
- 递归扫描指定根目录下的所有子目录（仅目录）
- 采用上述忽略策略过滤

#### 节点属性检查
对每个目录节点，工具会检查以下属性：

- **has_layout**: 节点内是否有 `layout.md` 文件（说明节点内有明确的排版内容）
- **has_visual_assets**: 节点内是否有 `visual_assets` 目录（说明节点内有图文资源）
- **has_text**: `visual_assets` 目录下 `.md` 文件的数量（非负整数，0 表示没有）
- **has_images**: `visual_assets` 目录下 `.png`、`.webp`、`.jpg` 文件的总数（非负整数，0 表示没有）
- **has_subnodes**: 节点内是否有其他目录（排除 `visual_assets` 和 `project_archive`，说明节点内有子节点）

#### 节点 CSV 输出格式
生成的 CSV（默认文件名：node.csv）包含以下列：
- `path`: 目录路径（ltree 格式，点号分隔，如 `a.b.c`）
- `has_layout`: 布尔值（true/false）
- `has_visual_assets`: 布尔值（true/false）
- `has_text`: 非负整数（`.md` 文件的数量，0 表示没有）
- `has_images`: 非负整数（`.png`、`.webp`、`.jpg` 文件的总数，0 表示没有）
- `has_subnodes`: 布尔值（true/false）

**注意：** 路径使用 ltree 格式（点号分隔），目录名中的特殊字符会被替换为下划线，以符合 ltree 标签要求（只能包含字母、数字、下划线）。

### 3. 资源扫描（--visual-scan）
- 当遇到任意节点下的 `visual_assets/` 目录时，列出该目录内的“直接文件”（不递归子目录）
- 每个文件以“相对于根”的路径，转换为 ltree 形式输出为一行
- CSV 列：`file_path`
- 默认输出文件名：`visual_assets.csv`
- 文件路径会按 ltree 规则清洗（非字母/数字/下划线将替换为 `_`；点号也会被替换为 `_`）

## 使用方法

### 编译
```bash
cd utils/node-generate-tool
cargo build --release
```

### 基本用法
```bash
# 1) 节点扫描（默认即节点扫描，或显式指定 --node）
# 默认输出 node.csv（可通过 -o 指定）
./target/release/node-generate-tool /path/to/data/root
./target/release/node-generate-tool /path/to/data/root --node
./target/release/node-generate-tool /path/to/data/root --node -o nodes.csv

# 指定输出文件
./target/release/node-generate-tool /path/to/data/root -o nodes.csv

# 2) 资源扫描（扫描 visual_assets/ 的直接文件）
# 默认输出 visual_assets.csv（可通过 -o 指定）
./target/release/node-generate-tool /path/to/data/root --visual-scan
./target/release/node-generate-tool /path/to/data/root --visual-scan -o images.csv

# 指定忽略文件名（默认 fileignore）
./target/release/node-generate-tool /path/to/data/root --ignore-file custom.ignore
```

## 工作流程

1. **更新数据中心**: 在数据中心目录中添加、修改或删除目录/资源
2. **运行工具**:
   - 节点扫描：生成最新的节点表 CSV（`node.csv` 或自定义名称）
   - 资源扫描：生成最新的资源文件表 CSV（`visual_assets.csv` 或自定义名称）
3. **导入数据库**: 将生成的 CSV 文件导入 PostgreSQL，替换旧的数据
4. **后端查询**: 后端根据“节点表”和“资源文件表”进行高效检索与渲染

## 数据库集成

### 启用 ltree 扩展
```sql
CREATE EXTENSION IF NOT EXISTS ltree;
```

### 创建表结构（目录节点）
```sql
CREATE TABLE directory_nodes (
    path ltree PRIMARY KEY,
    has_layout BOOLEAN NOT NULL,
    has_visual_assets BOOLEAN NOT NULL,
    has_text INTEGER NOT NULL,
    has_images INTEGER NOT NULL,
    has_subnodes BOOLEAN NOT NULL
);

-- 创建索引以优化查询性能
CREATE INDEX idx_path_gist ON directory_nodes USING GIST (path);
CREATE INDEX idx_path_btree ON directory_nodes USING BTREE (path);
```

### 建议：文件资源表结构（可选）
```sql
CREATE TABLE file_nodes (
    file_path ltree PRIMARY KEY,   -- 完整文件路径（ltree 格式）
    parent_path ltree NOT NULL,    -- 父目录路径（可通过 subpath(file_path, 0, -1) 计算）
    file_name TEXT NOT NULL,       -- 文件名（可冗余保存）
    file_size BIGINT,              -- 可选：大小
    file_type TEXT,                -- 可选：MIME
    modified_time TIMESTAMP,       -- 可选：修改时间
    created_time TIMESTAMP         -- 可选：创建时间
);
CREATE INDEX idx_file_parent_gist ON file_nodes USING GIST (parent_path);
CREATE INDEX idx_file_parent_btree ON file_nodes USING BTREE (parent_path);
```

### 导入 CSV
```sql
-- 目录节点
COPY directory_nodes FROM '/path/to/node.csv' WITH (FORMAT csv, HEADER true);

-- 资源文件（如采用 file_nodes 表）
-- 你可以先导入到临时表（只有 file_path 一列），再用 SQL 生成其他列
-- 例如：
-- CREATE TEMP TABLE tmp_files(file_path ltree);
-- COPY tmp_files FROM '/path/to/visual_assets.csv' WITH (FORMAT csv, HEADER true);
-- INSERT INTO file_nodes(file_path, parent_path, file_name)
-- SELECT 
--   file_path,
--   subpath(file_path, 0, -1),
--   split_part(file_path::text, '.', nlevel(file_path))  -- 仅示例，按需要实现提取逻辑
-- FROM tmp_files;
```

### 后端查询示例

#### 基本查询
```sql
-- 查找有排版内容的节点
SELECT path FROM directory_nodes WHERE has_layout = true;

-- 查找有图片的节点（图片数量大于 0）
SELECT path FROM directory_nodes WHERE has_images > 0;

-- 查找有多个图片的节点（图片数量大于等于 5）
SELECT path FROM directory_nodes WHERE has_images >= 5;

-- 查找有完整图文资源的节点（有文字说明和图片）
SELECT path FROM directory_nodes 
WHERE has_visual_assets = true AND has_text > 0 AND has_images > 0;
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
  AND has_images > 0;

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

- 节点扫描：只扫描目录，不扫描文件
- 资源扫描：只扫描各节点的 `visual_assets/` 目录下的“直接文件”（不递归子目录）
- 路径使用点号分隔（ltree），便于在数据库中存储和查询
- 布尔值输出为 `true`/`false`，PostgreSQL 可直接识别
- 图片文件支持常见格式：jpg, jpeg, png, gif, webp, svg, bmp, ico
- 文字文件识别 `.md` 和 `.markdown` 扩展名

