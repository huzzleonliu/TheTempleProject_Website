use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::Parser;

/// 命令行参数
#[derive(Parser, Debug)]
#[command(name = "visual-assets-copy", version, about = "复制目录下所有 visual_assets 目录到 ~/Desktop，保持原目录结构")]
struct Args {
    /// 源目录（将从此目录递归查找 visual_assets 目录）
    source: PathBuf,

    /// 目标目录（默认: ~/Desktop）
    #[arg(short = 'o', long = "output", default_value = "~/Desktop")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // 解析源目录
    let source = if args.source.is_absolute() {
        args.source
    } else {
        std::env::current_dir()?.join(args.source)
    };

    if !source.is_dir() {
        anyhow::bail!("源目录不存在或不是目录: {}", source.display());
    }

    // 解析目标目录（处理 ~ 符号）
    let output = expand_tilde(&args.output)?;
    if !output.exists() {
        fs::create_dir_all(&output)
            .with_context(|| format!("无法创建目标目录: {}", output.display()))?;
    }

    println!("源目录: {}", source.display());
    println!("目标目录: {}", output.display());
    println!("开始查找 visual_assets 目录...\n");

    // 查找所有 visual_assets 目录
    let visual_assets_dirs = find_visual_assets_dirs(&source)?;
    
    if visual_assets_dirs.is_empty() {
        println!("未找到任何 visual_assets 目录");
        return Ok(());
    }

    println!("找到 {} 个 visual_assets 目录:\n", visual_assets_dirs.len());

    // 复制每个 visual_assets 目录
    let mut copied_count = 0;
    for va_dir in &visual_assets_dirs {
        // 计算相对路径（相对于源目录）
        let rel_path = va_dir
            .strip_prefix(&source)
            .with_context(|| format!("无法计算相对路径: {}", va_dir.display()))?;

        // 构建目标路径（在目标目录下保持原目录结构）
        let target_path = output.join(rel_path);

        println!("复制: {} -> {}", rel_path.display(), target_path.display());

        // 复制目录
        copy_dir(va_dir, &target_path)
            .with_context(|| format!("复制失败: {} -> {}", va_dir.display(), target_path.display()))?;
        
        copied_count += 1;
    }

    println!("\n完成！成功复制 {} 个 visual_assets 目录到 {}", copied_count, output.display());
    Ok(())
}

/// 递归查找所有 visual_assets 目录
fn find_visual_assets_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut result = Vec::new();
    find_visual_assets_dirs_recursive(root, root, &mut result)?;
    Ok(result)
}

/// 递归查找 visual_assets 目录的辅助函数
fn find_visual_assets_dirs_recursive(
    root: &Path,
    current: &Path,
    result: &mut Vec<PathBuf>,
) -> Result<()> {
    let entries = fs::read_dir(current)
        .with_context(|| format!("无法读取目录: {}", current.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| "无法读取目录项")?;
        let path = entry.path();

        if path.is_dir() {
            // 检查是否是 visual_assets 目录
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name == "visual_assets" {
                    result.push(path.clone());
                    // 找到 visual_assets 后，不再递归进入其子目录
                    continue;
                }
            }

            // 递归查找子目录
            find_visual_assets_dirs_recursive(root, &path, result)?;
        }
    }

    Ok(())
}

/// 复制目录（递归）
fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    // 创建目标目录本身
    fs::create_dir_all(target)
        .with_context(|| format!("无法创建目录: {}", target.display()))?;

    // 复制目录内容
    let entries = fs::read_dir(source)
        .with_context(|| format!("无法读取目录: {}", source.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| "无法读取目录项")?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else {
            // 确保目标文件的父目录存在
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("无法创建目录: {}", parent.display()))?;
            }
            fs::copy(&source_path, &target_path)
                .with_context(|| format!("无法复制文件: {} -> {}", source_path.display(), target_path.display()))?;
        }
    }

    Ok(())
}

/// 展开 ~ 符号到用户主目录
fn expand_tilde(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("~/") || path_str == "~" {
        let home = std::env::var("HOME")
            .context("无法获取用户主目录（HOME 环境变量未设置）")?;
        let expanded = path_str.replacen("~", &home, 1);
        Ok(PathBuf::from(expanded))
    } else {
        Ok(path.to_path_buf())
    }
}
