use egui::{FontData, FontDefinitions, FontFamily};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

const FONT_NAME: &str = "cjk_fallback";
const ENV_FONT_PATH: &str = "EGUI_FONT_PATH";

pub fn configure_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();

    if let Some(font_data) = load_cjk_font() {
        fonts.font_data.insert(
            FONT_NAME.to_string(),
            FontData::from_owned(font_data).into(),
        );

        for family in [FontFamily::Proportional, FontFamily::Monospace] {
            fonts
                .families
                .entry(family)
                .or_default()
                .insert(0, FONT_NAME.to_string());
        }
    } else {
        eprintln!(
            "未找到可用的中文字体，请安装常见的 CJK 字体或通过环境变量 {} 指定字体路径。",
            ENV_FONT_PATH
        );
    }

    ctx.set_fonts(fonts);
}

fn load_cjk_font() -> Option<Vec<u8>> {
    let candidates = collect_font_candidates();
    let mut seen = HashSet::new();

    for path in candidates {
        if !seen.insert(path.clone()) {
            continue;
        }
        if path.is_file() {
            if let Ok(bytes) = fs::read(&path) {
                eprintln!("已加载字体: {}", path.display());
                return Some(bytes);
            }
        }
    }
    None
}

fn collect_font_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(env_path) = std::env::var_os(ENV_FONT_PATH) {
        candidates.push(PathBuf::from(env_path));
    }

    if let Ok(mut exe_dir) = std::env::current_exe() {
        if !exe_dir.pop() {
            exe_dir = PathBuf::new();
        }
        for rel in [
            "fonts/NotoSansSC-Regular.otf",
            "fonts/NotoSansCJK-Regular.ttc",
            "fonts/wqy-microhei.ttc",
        ] {
            candidates.push(exe_dir.join(rel));
        }
    }

    push_platform_candidates(&mut candidates);
    candidates
}

#[cfg(target_os = "linux")]
fn push_platform_candidates(list: &mut Vec<PathBuf>) {
    for path in [
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSansSC-Regular.otf",
        "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
        "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
    ] {
        list.push(PathBuf::from(path));
    }
}

#[cfg(target_os = "windows")]
fn push_platform_candidates(list: &mut Vec<PathBuf>) {
    for path in [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\msjh.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simkai.ttf",
    ] {
        list.push(PathBuf::from(path));
    }
}

#[cfg(target_os = "macos")]
fn push_platform_candidates(list: &mut Vec<PathBuf>) {
    for path in [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/STSong.ttf",
        "/System/Library/Fonts/华文黑体.ttf",
    ] {
        list.push(PathBuf::from(path));
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
fn push_platform_candidates(_list: &mut Vec<PathBuf>) {}
