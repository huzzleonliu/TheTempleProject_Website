use std::{
    fs::{File, read_dir},
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};
use pathdiff::diff_paths;

use crate::util::{build_walker, cleanup_temp_ignore, escape_csv_field, path_to_ltree, same_path};

pub fn run_node_scan(root: &Path, output_path: &Path, ignore_file: &str) -> Result<()> {
    let mut writer = BufWriter::new(
        File::create(output_path)
            .with_context(|| format!("无法创建输出文件: {}", output_path.display()))?,
    );
    writeln!(writer, "path,has_subnodes,raw_path,raw_filename")
        .with_context(|| "写入 CSV 表头失败")?;

    let (walker, temp_ignore_path) = build_walker(root, ignore_file, false)?;
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
            if same_path(dir_path, root) {
                continue;
            }

            let rel = match diff_paths(dir_path, root) {
                Some(r) => r,
                None => continue,
            };

            let has_subnodes = check_has_subnodes(dir_path);

            if let Some(path_str) = path_to_ltree(&rel) {
                let raw_path = rel.to_string_lossy().replace('\\', "/");
                let raw_filename = dir_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();

                let raw_path_escaped = escape_csv_field(&raw_path);
                let raw_filename_escaped = escape_csv_field(&raw_filename);

                writeln!(
                    writer,
                    "{},{},{},{}",
                    path_str, has_subnodes, raw_path_escaped, raw_filename_escaped
                )
                .with_context(|| "写入 CSV 失败")?;
            }
        }
        Ok(())
    })();
    cleanup_temp_ignore(temp_ignore_path);
    result?;

    writer.flush().ok();

    println!("已生成: {}", output_path.display());
    Ok(())
}

fn check_has_subnodes(dir: &Path) -> bool {
    let Ok(entries) = read_dir(dir) else {
        return false;
    };

    let excluded_dirs: &[&str] = &["visual_assets", "project_archive"];

    for entry in entries.flatten() {
        let path = entry.path();
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
