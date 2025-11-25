mod node_scan;
mod util;
mod visual_copy;
mod visual_scan;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::{Args as ClapArgs, Parser, Subcommand};
use node_scan::run_node_scan;
use util::{canonicalize_path, resolve_copy_destination};
use visual_copy::run_copy_visual_assets;
use visual_scan::run_visual_scan;

/// 顶层命令行定义
#[derive(Parser, Debug)]
#[command(
    name = "node-generate-tool",
    version,
    about = "生成目录/资源 CSV 或复制 visual 资源（支持 fileignore，gitignore 语法）"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// 扫描目录生成 CSV
    Scan {
        #[command(subcommand)]
        target: ScanCommand,
    },
    /// 复制指定类型资源
    Copy {
        #[command(subcommand)]
        target: CopyCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ScanCommand {
    /// 扫描节点目录，生成 node.csv
    Node(ScanArgs),
    /// 扫描 visual_assets，生成 visual_assets.csv
    Visual(ScanArgs),
}

#[derive(Subcommand, Debug)]
enum CopyCommand {
    /// 复制 visual_assets 目录到目标位置
    Visual(CopyArgs),
}

#[derive(ClapArgs, Debug)]
struct ScanArgs {
    /// 根目录（将从该目录递归遍历子目录）
    root: PathBuf,
    /// 输出文件路径（默认：node.csv 或 visual_assets.csv）
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,
    /// 指定 ignore 文件名（默认: fileignore，使用 gitignore 语法）
    #[arg(long = "ignore-file", default_value = "fileignore")]
    ignore_file: String,
}

#[derive(ClapArgs, Debug)]
struct CopyArgs {
    /// 根目录（将从该目录递归遍历子目录）
    root: PathBuf,
    /// 输出目录路径（默认：~/Desktop）
    #[arg(short = 'o', long = "output")]
    output: Option<PathBuf>,
    /// 指定 ignore 文件名（默认: fileignore，使用 gitignore 语法）
    #[arg(long = "ignore-file", default_value = "fileignore")]
    ignore_file: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Scan { target } => match target {
            ScanCommand::Node(scan_args) => run_scan_node(scan_args),
            ScanCommand::Visual(scan_args) => run_scan_visual(scan_args),
        },
        Command::Copy { target } => match target {
            CopyCommand::Visual(copy_args) => run_copy_visual(copy_args),
        },
    }
}

fn run_scan_node(args: ScanArgs) -> Result<()> {
    let root = ensure_root_directory(&args.root)?;
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("node.csv"));
    run_node_scan(&root, &output_path, &args.ignore_file)
}

fn run_scan_visual(args: ScanArgs) -> Result<()> {
    let root = ensure_root_directory(&args.root)?;
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| PathBuf::from("visual_assets.csv"));
    run_visual_scan(&root, &output_path, &args.ignore_file)
}

fn run_copy_visual(args: CopyArgs) -> Result<()> {
    let root = ensure_root_directory(&args.root)?;
    let destination = match &args.output {
        Some(path) => resolve_copy_destination(path)?,
        None => resolve_copy_destination(Path::new("~/Desktop"))?,
    };
    run_copy_visual_assets(&root, &destination, &args.ignore_file)
}

fn ensure_root_directory(root: &Path) -> Result<PathBuf> {
    let root = canonicalize_path(root)
        .with_context(|| format!("无法解析根目录路径: {}", root.display()))?;
    if !root.is_dir() {
        bail!("提供的路径不是目录: {}", root.display());
    }
    Ok(root)
}
