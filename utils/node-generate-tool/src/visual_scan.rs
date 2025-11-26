use std::{
    fs::{File, read_dir},
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::{Context, Result};
use pathdiff::diff_paths;

use crate::util::{build_walker, cleanup_temp_ignore, escape_csv_field, path_to_ltree};

pub fn run_visual_scan(root: &Path, output_path: &Path, ignore_file: &str) -> Result<()> {
    let mut writer = BufWriter::new(
        File::create(output_path)
            .with_context(|| format!("无法创建输出文件: {}", output_path.display()))?,
    );
    writeln!(writer, "file_path,raw_path,raw_filename").with_context(|| "写入 CSV 表头失败")?;

    let (walker, temp_ignore_path, _) = build_walker(root, ignore_file, true)?;
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

            if let Ok(entries) = read_dir(dir_path) {
                for entry in entries.flatten() {
                    let file_path = entry.path();
                    if !file_path.is_file() {
                        continue;
                    }

                    let Some(rel_file) = diff_paths(&file_path, root) else {
                        continue;
                    };

                    if let Some(file_ltree) = path_to_ltree(&rel_file) {
                        let raw_path = rel_file.to_string_lossy().replace('\\', "/");
                        let raw_filename = file_path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("")
                            .to_string();

                        let raw_path_escaped = escape_csv_field(&raw_path);
                        let raw_filename_escaped = escape_csv_field(&raw_filename);

                        writeln!(
                            writer,
                            "{},{},{}",
                            file_ltree, raw_path_escaped, raw_filename_escaped
                        )
                        .with_context(|| "写入 CSV 失败")?;
                    }
                }
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
