use std::{fs, path::Path};

use anyhow::{Context, Result};
use pathdiff::diff_paths;

use crate::util::{build_walker, cleanup_temp_ignore, copy_dir};

pub fn run_copy_visual_assets(root: &Path, destination: &Path, ignore_file: &str) -> Result<()> {
    if !destination.exists() {
        fs::create_dir_all(destination)
            .with_context(|| format!("无法创建目标目录: {}", destination.display()))?;
    }

    println!("源目录: {}", root.display());
    println!("目标目录: {}", destination.display());
    println!("开始查找 visual_assets 目录...\n");

    let (walker, temp_ignore_path, consolidated_ignore_rules) =
        build_walker(root, ignore_file, true)?;
    let mut copied_count = 0usize;
    let result: Result<()> = (|| {
        for dent in walker {
            let dent = match dent {
                Ok(d) => d,
                Err(err) => {
                    eprintln!("跳过不可读条目: {err}");
                    continue;
                }
            };

            let Some(ft) = dent.file_type() else { continue };
            if !ft.is_dir() {
                continue;
            }

            let dir_path = dent.path();
            if !matches!(
                dir_path.file_name().and_then(|n| n.to_str()),
                Some("visual_assets")
            ) {
                continue;
            }

            let Some(rel) = diff_paths(dir_path, root) else {
                continue;
            };
            let target_path = destination.join(&rel);

            println!("复制: {} -> {}", rel.display(), target_path.display());
            copy_dir(dir_path, &target_path).with_context(|| {
                format!(
                    "复制失败: {} -> {}",
                    dir_path.display(),
                    target_path.display()
                )
            })?;

            copied_count += 1;
        }
        Ok(())
    })();
    cleanup_temp_ignore(temp_ignore_path);
    result?;

    if let Some(rules) = consolidated_ignore_rules {
        let ignore_output_path = destination.join(ignore_file);
        fs::write(&ignore_output_path, rules).with_context(|| {
            format!(
                "无法写入整理后的 ignore 文件: {}",
                ignore_output_path.display()
            )
        })?;
        println!(
            "已在目标目录生成整理后的 ignore 文件: {}",
            ignore_output_path.display()
        );
    } else {
        println!("未发现可合并的 ignore 规则，跳过导出整理文件");
    }

    if copied_count == 0 {
        println!("未找到任何 visual_assets 目录（或全部被 ignore 规则过滤）");
    } else {
        println!(
            "\n完成！成功复制 {} 个 visual_assets 目录到 {}",
            copied_count,
            destination.display()
        );
    }

    Ok(())
}
