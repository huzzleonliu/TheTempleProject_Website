mod node_scan;
mod util;
mod visual_copy;
mod visual_scan;

use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use clap::Parser;
use node_scan::run_node_scan;
use util::{canonicalize_path, resolve_copy_destination};
use visual_copy::run_copy_visual_assets;
use visual_scan::run_visual_scan;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Node,
    VisualScan,
    Copy,
}

/// 命令行参数
#[derive(Parser, Debug)]
#[command(
    name = "node-generate-tool",
    version,
    about = "生成目录/资源 CSV（支持 fileignore，gitignore 语法）"
)]
struct Args {
    /// 根目录（将从该目录递归遍历子目录）
    root: PathBuf,

    /// 输出文件/目录路径：
    /// - --node 模式默认写入 node.csv（CSV）
    /// - --visual-scan 模式默认写入 visual_assets.csv（CSV）
    /// - --copy-visual 模式默认复制到 ~/Desktop，可用 -o 指定目标目录
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

    /// 启用 visual 资源复制：将 visual_assets 目录复制到目标目录，同时应用 fileignore 规则
    #[arg(long = "copy-visual")]
    copy_visual: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let root = canonicalize_path(&args.root)
        .with_context(|| format!("无法解析根目录路径: {}", args.root.display()))?;
    if !root.is_dir() {
        bail!("提供的路径不是目录: {}", root.display());
    }

    let mode = resolve_mode(&args)?;

    match mode {
        Mode::Node => {
            let output_path = args
                .output
                .clone()
                .unwrap_or_else(|| PathBuf::from("node.csv"));
            run_node_scan(&root, &output_path, &args.ignore_file)
        }
        Mode::VisualScan => {
            let output_path = args
                .output
                .clone()
                .unwrap_or_else(|| PathBuf::from("visual_assets.csv"));
            run_visual_scan(&root, &output_path, &args.ignore_file)
        }
        Mode::Copy => {
            let destination = match &args.output {
                Some(path) => resolve_copy_destination(path)?,
                None => resolve_copy_destination(Path::new("~/Desktop"))?,
            };
            run_copy_visual_assets(&root, &destination, &args.ignore_file)
        }
    }
}

fn resolve_mode(args: &Args) -> Result<Mode> {
    let mut count = 0;
    if args.node {
        count += 1;
    }
    if args.visual_scan {
        count += 1;
    }
    if args.copy_visual {
        count += 1;
    }

    if count > 1 {
        bail!("--node、--visual-scan、--copy-visual 互斥，请一次只选择一种模式");
    }

    if args.visual_scan {
        Ok(Mode::VisualScan)
    } else if args.copy_visual {
        Ok(Mode::Copy)
    } else {
        Ok(Mode::Node)
    }
}
