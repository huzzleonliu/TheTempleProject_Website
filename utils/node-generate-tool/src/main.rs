use std::{
    fs::{read_dir, File},
    io::{BufWriter, Write},
    path::{Component, Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use clap::Parser;
use ignore::WalkBuilder;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(name = "node-generate-tool", version, about = "生成目录/资源 CSV（支持 fileignore，gitignore 语法）")] 
struct Args {
    /// 根目录（将从该目录递归遍历子目录）
    root: PathBuf,

    /// 输出 CSV 文件路径（--node 模式默认: node.csv；--visual-scan 模式默认: visual_assets.csv）
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// 指定 ignore 文件名（默认: fileignore，使用 gitignore 语法）
    #[arg(long = "ignore-file", default_value = "fileignore")]
    ignore_file: String,

    /// 启用节点扫描模式（扫描目录，输出节点属性）
    #[arg(long = "node")]
    node: bool,

    /// 启用 visual 资源扫描：输出 visual_assets 目录下的文件（不含子目录），以 ltree 形式记录文件路径
    #[arg(long = "visual-scan")]
    visual_scan: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let root = canonicalize_path(&args.root)
        .with_context(|| format!("无法解析根目录路径: {}", args.root.display()))?;
    if !root.is_dir() {
        bail!("提供的路径不是目录: {}", root.display());
    }

    // 模式优先级：若提供 --visual-scan 则为 visual 模式；否则若提供 --node 则为 node 模式；二者都未提供时，默认 node 模式
    let is_visual = args.visual_scan;
    let is_node = !is_visual && (args.node || !args.node); // 默认 node 模式

    let output_path = match (&args.output, is_visual, is_node) {
        (Some(p), _, _) => p.clone(),
        (None, true, _) => PathBuf::from("visual_assets.csv"),
        (None, _, true) => PathBuf::from("node.csv"),
        // 理论不可达，兜底
        (None, _, _) => PathBuf::from("node.csv"),
    };

    let mut writer = BufWriter::new(
        File::create(&output_path)
            .with_context(|| format!("无法创建输出文件: {}", output_path.display()))?,
    );

    // 写入 CSV 表头（PostgreSQL 可直接导入）
    if is_visual {
        writeln!(writer, "file_path,raw_path,raw_filename").with_context(|| "写入 CSV 表头失败")?;
    } else {
        writeln!(writer, "path,has_subnodes,raw_path,raw_filename")
            .with_context(|| "写入 CSV 表头失败")?;
    }

    // 使用 ignore 的 WalkBuilder，按如下顺序合并忽略文件（gitignore 语法）：
    // 1) 程序源码目录（编译期确定）
    // 2) 可执行程序所在目录
    // 3) 被扫描的根目录
    // 可通过 --ignore-file 指定文件名，默认 fileignore
    let mut builder = WalkBuilder::new(&root);
    builder
        .hidden(false) // 包含隐藏目录，是否忽略交由规则决定
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .follow_links(false);

    // 收集候选 ignore 文件路径
    let mut ignore_files: Vec<PathBuf> = Vec::new();

    // 1) 程序源码目录（编译期的 crate 根目录）
    let program_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ignore_files.push(program_dir.join(&args.ignore_file));

    // 2) 可执行程序所在目录
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().expect("无法获取当前工作目录"));
    ignore_files.push(exe_dir.join(&args.ignore_file));

    // 3) 被扫描的根目录
    ignore_files.push(root.join(&args.ignore_file));

    // 读取并合并所有存在的 ignore 文件内容
    let mut merged_rules = String::new();
    let mut loaded_count = 0;
    for p in &ignore_files {
        if p.exists() {
            if let Ok(content) = std::fs::read_to_string(p) {
                eprintln!("已加载 ignore 文件: {}", p.display());
                if !merged_rules.is_empty() {
                    merged_rules.push('\n');
                }
                merged_rules.push_str(&format!("# From: {}\n", p.display()));
                merged_rules.push_str(&content);
                merged_rules.push('\n');
                loaded_count += 1;
            }
        }
    }
    
    if loaded_count == 0 {
        eprintln!("警告: 未找到任何 ignore 文件");
    } else {
        eprintln!("共加载 {} 个 ignore 文件", loaded_count);
    }

    // 在 visual 扫描模式下，强制包含 visual_assets 目录，覆盖 fileignore 中的忽略规则
    // 通过添加后置的“取消忽略”规则来实现（gitignore 语义：后规则优先）
    if is_visual {
        merged_rules.push_str("\n# visual-scan mode: force include visual_assets\n");
        merged_rules.push_str("!visual_assets/\n");
        merged_rules.push_str("!**/visual_assets/\n");
        merged_rules.push_str("!**/visual_assets/**\n");
    }

    // 如果有合并的规则，创建临时文件在扫描根目录，然后使用 add_custom_ignore_filename
    let temp_ignore_path = if !merged_rules.is_empty() {
        let temp_path = root.join(format!(".{}_merged", args.ignore_file));
        std::fs::write(&temp_path, merged_rules)
            .with_context(|| format!("无法创建临时 ignore 文件: {}", temp_path.display()))?;
        
        // 使用 add_custom_ignore_filename，这样规则是相对于扫描根目录的
        builder.add_custom_ignore_filename(&format!(".{}_merged", args.ignore_file));
        
        Some(temp_path)
    } else {
        None
    };

    let walker = builder.build();

    for dent in walker {
        let dent = match dent {
            Ok(d) => d,
            Err(err) => {
                eprintln!("跳过不可读条目: {err}");
                continue;
            }
        };

        let Some(ft) = dent.file_type() else { continue };

        let p = dent.path();

        if is_visual {
            // visual 扫描模式：当遇到 visual_assets 目录时，列出其中的文件（不递归子目录）
            if ft.is_dir() {
                if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
                    if name == "visual_assets" {
                        // 遍历 visual_assets 下的直接文件
                        if let Ok(entries) = read_dir(p) {
                            for entry in entries.flatten() {
                                let fpath = entry.path();
                                if fpath.is_file() {
                                    if let Some(rel_file) = pathdiff::diff_paths(&fpath, &root) {
                                        if let Some(file_ltree) = path_to_ltree(&rel_file) {
                                            // 获取原始路径（相对于根目录，使用正斜杠）
                                            let raw_path = rel_file.to_string_lossy().replace('\\', "/");
                                            // 获取原始文件名
                                            let raw_filename = fpath
                                                .file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("")
                                                .to_string();
                                            
                                            // 转义 CSV 字段（如果包含逗号、引号或换行符）
                                            let raw_path_escaped = escape_csv_field(&raw_path);
                                            let raw_filename_escaped = escape_csv_field(&raw_filename);
                                            
                                            writeln!(writer, "{},{},{}", file_ltree, raw_path_escaped, raw_filename_escaped)
                                                .with_context(|| "写入 CSV 失败")?;
                                        }
                                    }
                                }
                            }
                        }
                        // visual 模式仅关心文件，继续下一个条目
                        continue;
                    }
                }
            }
            // 其他目录/文件在 visual 模式中忽略
            continue;
        }

        // 目录扫描模式（默认） - 仅输出目录
        if !ft.is_dir() { continue; }

        // 跳过根目录自身
        if same_path(p, &root) { continue; }

        // 计算相对路径
        let rel = match pathdiff::diff_paths(p, &root) {
            Some(r) => r,
            None => continue,
        };

        // 检查目录节点的属性
        let has_subnodes = check_has_subnodes(p);

        // 生成 CSV 行：路径（ltree 格式，点号分隔）+ 布尔值 + 原始路径 + 原始文件名
        if let Some(path_str) = path_to_ltree(&rel) {
            // 获取原始路径（相对于根目录，使用正斜杠）
            let raw_path = rel.to_string_lossy().replace('\\', "/");
            // 获取原始目录名（用于显示节点名称）
            let raw_filename = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            // 转义 CSV 字段（如果包含逗号、引号或换行符）
            let raw_path_escaped = escape_csv_field(&raw_path);
            let raw_filename_escaped = escape_csv_field(&raw_filename);
            
            writeln!(
                writer,
                "{},{},{},{}",
                path_str,
                has_subnodes,
                raw_path_escaped,
                raw_filename_escaped
            )
            .with_context(|| "写入 CSV 失败")?;
        }
    }

    writer.flush().ok();
    
    // 清理临时 ignore 文件
    if let Some(temp_path) = temp_ignore_path {
        let _ = std::fs::remove_file(&temp_path);
    }
    
    println!("已生成: {}", output_path.display());
    Ok(())
}

fn canonicalize_path(p: &Path) -> Result<PathBuf> {
    // 对于不存在但可访问的路径 canonicalize 可能失败，先尝试直接返回绝对路径
    let abs = if p.is_absolute() { p.to_path_buf() } else { std::env::current_dir()?.join(p) };
    Ok(abs)
}

fn same_path(a: &Path, b: &Path) -> bool {
    // 尽量以组件对比，避免大小写/分隔符细节影响（不跨平台处理大小写）
    normalize_components(a) == normalize_components(b)
}

fn normalize_components(p: &Path) -> Vec<String> {
    p.components()
        .filter_map(|c| match c { Component::Normal(os) => os.to_str().map(|s| s.to_string()), _ => None })
        .collect()
}

/// 将路径转换为 ltree 格式（点号分隔）
/// ltree 标签只能包含字母、数字、下划线（A-Za-z0-9_）
/// 其他字符会被替换为下划线
fn path_to_ltree(rel: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for comp in rel.components() {
        if let Component::Normal(os) = comp {
            let s = os.to_str()?;
            if !s.is_empty() {
                // 清理目录名，使其符合 ltree 格式要求
                let cleaned = sanitize_ltree_label(s);
                if !cleaned.is_empty() {
                    parts.push(cleaned);
                }
            }
        }
    }
    if parts.is_empty() { return None; }
    Some(parts.join("."))
}

/// 清理标签，使其符合 ltree 格式要求
/// ltree 标签只能包含字母、数字、下划线（A-Za-z0-9_）
/// 其他字符会被替换为下划线
fn sanitize_ltree_label(label: &str) -> String {
    label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// 转义 CSV 字段
/// 如果字段包含逗号、引号或换行符，需要用双引号包裹，并转义内部的双引号
fn escape_csv_field(field: &str) -> String {
    // 如果字段包含逗号、引号或换行符，需要转义
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        // 用双引号包裹，并将内部的双引号转义为两个双引号
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// 检查目录内是否有其他目录（排除 visual_assets 和 project_archive）
fn check_has_subnodes(dir: &Path) -> bool {
    let Ok(entries) = read_dir(dir) else {
        return false;
    };

    // 需要排除的目录名
    let excluded_dirs: &[&str] = &["visual_assets", "project_archive"];

    for entry in entries.flatten() {
        let path = entry.path();
        // 只检查目录，排除 visual_assets 和 project_archive
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if !excluded_dirs.contains(&name) {
                    return true;
                }
            }
        }
    }

    false
}
