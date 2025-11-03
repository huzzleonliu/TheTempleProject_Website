use std::{
    fs::File,
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

    // 依次添加存在的 ignore 文件
    for p in ignore_files {
        if p.exists() {
            builder.add_ignore(p);
        }
    }

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

        // 计算相对路径并写入 CSV（用逗号分隔路径段）
        let rel = match pathdiff::diff_paths(p, &root) {
            Some(r) => r,
            None => continue,
        };

        if let Some(line) = path_to_csv_line(&rel) {
            writeln!(writer, "{}", line).with_context(|| "写入 CSV 失败")?;
        }
    }

    writer.flush().ok();
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
