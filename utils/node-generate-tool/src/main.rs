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
#[command(name = "node-generate-tool", version, about = "生成目录路径 CSV（支持 fileignore，gitignore 语法）")] 
struct Args {
    /// 根目录（将从该目录递归遍历子目录）
    root: PathBuf,

    /// 输出 CSV 文件路径（默认: 当前目录下 directories.csv）
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,

    /// 指定 ignore 文件名（默认: fileignore，使用 gitignore 语法）
    #[arg(long = "ignore-file", default_value = "fileignore")]
    ignore_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let root = canonicalize_path(&args.root)
        .with_context(|| format!("无法解析根目录路径: {}", args.root.display()))?;
    if !root.is_dir() {
        bail!("提供的路径不是目录: {}", root.display());
    }

    let output_path = args
        .output
        .unwrap_or_else(|| PathBuf::from("directories.csv"));

    let mut writer = BufWriter::new(
        File::create(&output_path)
            .with_context(|| format!("无法创建输出文件: {}", output_path.display()))?,
    );

    // 写入 CSV 表头（PostgreSQL 可直接导入）
    writeln!(writer, "path,has_layout,has_visual_assets,has_text,has_images,has_subnodes")
        .with_context(|| "写入 CSV 表头失败")?;

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

        // 仅输出目录
        let Some(ft) = dent.file_type() else { continue }; 
        if !ft.is_dir() { continue; }

        let p = dent.path();

        // 跳过根目录自身
        if same_path(p, &root) { continue; }

        // 计算相对路径
        let rel = match pathdiff::diff_paths(p, &root) {
            Some(r) => r,
            None => continue,
        };

        // 检查目录节点的各种属性
        let has_layout = check_has_layout(p);
        let has_visual_assets = check_has_visual_assets(p);
        let (has_text, has_images) = if has_visual_assets {
            check_visual_assets_content(p)
        } else {
            (false, false)
        };
        let has_subnodes = check_has_subnodes(p);

        // 生成 CSV 行：路径（逗号分隔）+ 布尔值（true/false）
        if let Some(path_str) = path_to_csv_line(&rel) {
            writeln!(
                writer,
                "{},{},{},{},{},{}",
                path_str,
                has_layout,
                has_visual_assets,
                has_text,
                has_images,
                has_subnodes
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

fn path_to_csv_line(rel: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for comp in rel.components() {
        if let Component::Normal(os) = comp {
            let s = os.to_str()?;
            if !s.is_empty() { parts.push(s.to_string()); }
        }
    }
    if parts.is_empty() { return None; }
    Some(parts.join(","))
}

/// 检查目录内是否有 layout.md 文件
fn check_has_layout(dir: &Path) -> bool {
    dir.join("layout.md").is_file()
}

/// 检查目录内是否有 visual_assets 目录
fn check_has_visual_assets(dir: &Path) -> bool {
    dir.join("visual_assets").is_dir()
}

/// 检查 visual_assets 目录下的内容
/// 返回 (has_text, has_images)
fn check_visual_assets_content(dir: &Path) -> (bool, bool) {
    let va_dir = dir.join("visual_assets");
    if !va_dir.is_dir() {
        return (false, false);
    }

    let Ok(entries) = read_dir(&va_dir) else {
        return (false, false);
    };

    let mut has_text = false;
    let mut has_images = false;

    // 常见图片扩展名
    let image_exts: &[&str] = &["jpg", "jpeg", "png", "gif", "webp", "svg", "bmp", "ico"];

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            // 检查 markdown 文件
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") {
                    has_text = true;
                }
                // 检查图片文件
                if image_exts.iter().any(|&e| ext.eq_ignore_ascii_case(e)) {
                    has_images = true;
                }
            }
        }
        // 如果已经找到两者，可以提前退出
        if has_text && has_images {
            break;
        }
    }

    (has_text, has_images)
}

/// 检查目录内是否有其他目录（排除 visual_assets）
fn check_has_subnodes(dir: &Path) -> bool {
    let Ok(entries) = read_dir(dir) else {
        return false;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        // 只检查目录，排除 visual_assets
        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name != "visual_assets" {
                    return true;
                }
            }
        }
    }

    false
}
