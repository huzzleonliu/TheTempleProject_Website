use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use anyhow::{Context, Result};
use ignore::{Walk, WalkBuilder};

pub fn canonicalize_path(p: &Path) -> Result<PathBuf> {
    let abs = if p.is_absolute() {
        p.to_path_buf()
    } else {
        std::env::current_dir()?.join(p)
    };
    Ok(abs)
}

pub fn build_walker(
    root: &Path,
    ignore_file: &str,
    include_visual: bool,
) -> Result<(Walk, Option<PathBuf>)> {
    let mut builder = WalkBuilder::new(root);
    builder
        .hidden(false)
        .git_ignore(false)
        .git_global(false)
        .git_exclude(false)
        .follow_links(false);

    let mut ignore_files: Vec<PathBuf> = Vec::new();

    let program_dir: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    ignore_files.push(program_dir.join(ignore_file));

    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| std::env::current_dir().expect("无法获取当前工作目录"));
    ignore_files.push(exe_dir.join(ignore_file));

    ignore_files.push(root.join(ignore_file));

    let mut merged_rules = String::new();
    let mut loaded_count = 0;
    for path in &ignore_files {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                eprintln!("已加载 ignore 文件: {}", path.display());
                if !merged_rules.is_empty() {
                    merged_rules.push('\n');
                }
                merged_rules.push_str(&format!("# From: {}\n", path.display()));
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

    if include_visual {
        merged_rules.push_str("\n# visual modes: force include visual_assets\n");
        merged_rules.push_str("!visual_assets/\n");
        merged_rules.push_str("!**/visual_assets/\n");
        merged_rules.push_str("!**/visual_assets/**\n");
    }

    let temp_ignore_path = if !merged_rules.is_empty() {
        let temp_path = root.join(format!(".{}_merged", ignore_file));
        fs::write(&temp_path, merged_rules)
            .with_context(|| format!("无法创建临时 ignore 文件: {}", temp_path.display()))?;

        builder.add_custom_ignore_filename(&format!(".{}_merged", ignore_file));
        Some(temp_path)
    } else {
        None
    };

    Ok((builder.build(), temp_ignore_path))
}

pub fn cleanup_temp_ignore(temp_path: Option<PathBuf>) {
    if let Some(path) = temp_path {
        if let Err(err) = fs::remove_file(&path) {
            eprintln!("清理临时 ignore 文件失败: {} ({err})", path.display());
        }
    }
}

pub fn resolve_copy_destination(path: &Path) -> Result<PathBuf> {
    let expanded = expand_tilde(path)?;
    if expanded.is_absolute() {
        Ok(expanded)
    } else {
        Ok(std::env::current_dir()?.join(expanded))
    }
}

pub fn expand_tilde(path: &Path) -> Result<PathBuf> {
    let path_str = path.to_string_lossy();
    if path_str.starts_with("~/") || path_str == "~" {
        let home = std::env::var("HOME").context("无法获取用户主目录（HOME 环境变量未设置）")?;
        let expanded = path_str.replacen('~', &home, 1);
        Ok(PathBuf::from(expanded))
    } else {
        Ok(path.to_path_buf())
    }
}

pub fn copy_dir(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("无法创建目录: {}", target.display()))?;

    let entries =
        fs::read_dir(source).with_context(|| format!("无法读取目录: {}", source.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| "无法读取目录项")?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());

        if source_path.is_dir() {
            copy_dir(&source_path, &target_path)?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)
                    .with_context(|| format!("无法创建目录: {}", parent.display()))?;
            }
            fs::copy(&source_path, &target_path).with_context(|| {
                format!(
                    "无法复制文件: {} -> {}",
                    source_path.display(),
                    target_path.display()
                )
            })?;
        }
    }

    Ok(())
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    normalize_components(a) == normalize_components(b)
}

fn normalize_components(p: &Path) -> Vec<String> {
    p.components()
        .filter_map(|c| match c {
            Component::Normal(os) => os.to_str().map(|s| s.to_string()),
            _ => None,
        })
        .collect()
}

pub fn path_to_ltree(rel: &Path) -> Option<String> {
    let mut parts = Vec::new();
    for comp in rel.components() {
        if let Component::Normal(os) = comp {
            let s = os.to_str()?;
            if !s.is_empty() {
                let cleaned = sanitize_ltree_label(s);
                if !cleaned.is_empty() {
                    parts.push(cleaned);
                }
            }
        }
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join("."))
}

pub fn sanitize_ltree_label(label: &str) -> String {
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

pub fn escape_csv_field(field: &str) -> String {
    if field.contains(',') || field.contains('"') || field.contains('\n') || field.contains('\r') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}
