use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// 抽象“可重命名条目”，把 UI 层的 `FileEntry` 与重命名算法解耦。
pub trait RenamableEntry {
    fn path(&self) -> &Path;
    fn file_name(&self) -> &str;
    fn set_path(&mut self, path: PathBuf);
    fn set_file_name(&mut self, name: String);
}

/// 按当前条目顺序重命名（两阶段，避免命名冲突）。
///
/// - **start_number**: 起始序号（会被 clamp 到 >= 1）
/// - 目标文件名形如：`001.ext` / `002.ext`（ext 来自原文件）
pub fn rename_in_order<E: RenamableEntry>(
    dir: &Path,
    entries: &mut [E],
    start_number: usize,
) -> Result<(), String> {
    if entries.is_empty() {
        return Err("没有可重命名的文件".to_string());
    }

    let originals: HashSet<PathBuf> = entries.iter().map(|e| e.path().to_path_buf()).collect();

    let file_count = entries.len();
    let start_number = start_number.max(1);
    let max_number = start_number.saturating_add(file_count.saturating_sub(1));
    let width = max_number.to_string().len();

    let mut final_paths: Vec<(usize, PathBuf, String)> = Vec::with_capacity(entries.len());
    for (idx, entry) in entries.iter().enumerate() {
        // 获取原始文件名
        let mut original_name = entry.file_name().to_string();
        // 如果文件名中包含 _ 且第一个部分是数字，则认为文件已重命名，取第二个部分作为原始文件名
        let have_renamed = original_name.contains("_") && original_name.split("_").nth(0).is_some_and(|s| s.parse::<usize>().is_ok());
        if have_renamed {
            original_name = original_name.split("_").nth(1).unwrap().to_string();
        }
        // 获取文件扩展名
        let ext = entry
            .path()
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();

        let current_number = start_number.saturating_add(idx);
        let stem = format!("{:0width$}", current_number, width = width);
        let new_name = if ext.is_empty() {
            stem.clone()
        } else {
            format!("{}_{}.{}", stem, original_name, ext)
        };

        let final_path = dir.join(&new_name);
        if final_path.exists() && !originals.contains(&final_path) {
            return Err(format!("目标文件已存在: {}", final_path.display()));
        }

        final_paths.push((idx, final_path, new_name));
    }

    // 第一步：全部改为唯一的临时文件名（避免目标名与源名互相覆盖）
    for (idx, entry) in entries.iter_mut().enumerate() {
        let temp_name = format!("__tmp_order_{:04}_{}", idx, entry.file_name());
        let temp_path = dir.join(&temp_name);
        fs::rename(entry.path(), &temp_path).map_err(|e| format!("重命名临时文件失败: {}", e))?;
        entry.set_path(temp_path);
        entry.set_file_name(temp_name);
    }

    // 第二步：按顺序命名为起始值递增的序号
    for (idx, final_path, new_name) in final_paths {
        fs::rename(entries[idx].path(), &final_path).map_err(|e| format!("最终重命名失败: {}", e))?;
        entries[idx].set_path(final_path);
        entries[idx].set_file_name(new_name);
    }

    Ok(())
}


