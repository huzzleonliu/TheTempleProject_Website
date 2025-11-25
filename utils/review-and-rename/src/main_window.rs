use crate::utils::{load_texture_from_image, UiUtils};
use docx_lite;
use egui;
use egui_dnd::dnd;
use pdf_extract;
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "gif", "webp"];
const TEXT_DOC_EXTENSIONS: &[&str] = &[
    "txt", "md", "rtf", "json", "csv", "toml", "yaml", "yml", "rs", "ts", "tsx",
];
const DOCX_EXTENSIONS: &[&str] = &["docx"];
const LEGACY_DOC_EXTENSIONS: &[&str] = &["doc"];
const PDF_EXTENSIONS: &[&str] = &["pdf"];
const MAX_PREVIEW_BYTES: usize = 8 * 1024;
const MAX_PREVIEW_CHARS: usize = 8 * 1024;

fn log_message(message: &str) {
    println!("[review-and-rename] {}", message);
    let _ = io::stdout().flush();
}

/// 主窗口状态：左侧文件列表 + 右侧预览 + 底部控制
pub struct MainWindow {
    directory_path: Option<PathBuf>,
    files: Vec<FileEntry>,
    selected_file_id: Option<u64>,
    status_message: Option<String>,
    next_id: u64,
    rename_start_value: usize,
    rename_start_input: String,
}

impl Default for MainWindow {
    fn default() -> Self {
        Self {
            directory_path: None,
            files: Vec::new(),
            selected_file_id: None,
            status_message: None,
            next_id: 0,
            rename_start_value: 1,
            rename_start_input: "1".to_string(),
        }
    }
}

impl eframe::App for MainWindow {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.show(ctx, frame);
    }
}

impl MainWindow {
    /// 顶层布局
    pub fn show(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.show_menu_bar(ctx, frame);
        self.show_bottom_panel(ctx);

        egui::SidePanel::left("file_list_panel")
            .resizable(true)
            .default_width(360.0)
            .show(ctx, |ui| self.show_file_list_panel(ctx, ui));

        egui::CentralPanel::default().show(ctx, |ui| self.show_preview_panel(ctx, ui));
    }

    /// 菜单栏：打开目录 / 退出
    fn show_menu_bar(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open Directory").clicked() {
                        self.open_directory_dialog(ctx);
                        ui.close();
                    }
                    if ui.button("Exit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        ui.close();
                    }
                });
            });
        });
    }

    /// 底部状态栏：重命名按钮 + 状态提示
    fn show_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let rename_enabled = self.directory_path.is_some() && !self.files.is_empty();
                if ui
                    .add_enabled(rename_enabled, egui::Button::new("按预览顺序重命名"))
                    .clicked()
                {
                    match self.rename_files() {
                        Ok(_) => self.status_message = Some("重命名完成".to_string()),
                        Err(err) => self.status_message = Some(err),
                    }
                }

                ui.separator();
                ui.label("起始值");
                let mut apply_start = false;
                let start_edit = ui.add(
                    egui::TextEdit::singleline(&mut self.rename_start_input).desired_width(80.0),
                );
                if start_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    apply_start = true;
                }
                if ui.button("提交").clicked() {
                    apply_start = true;
                }
                if apply_start {
                    self.apply_rename_start_input();
                }

                ui.separator();
                if ui.button("刷新目录").clicked() {
                    if let Some(dir) = self.directory_path.clone() {
                        if let Err(err) = self.load_directory(ctx, dir.as_path()) {
                            self.status_message = Some(err);
                        }
                    }
                }

                if let Some(dir) = &self.directory_path {
                    ui.label(format!("当前目录: {}", dir.display()));
                } else {
                    ui.label("未选择目录");
                }

                if let Some(msg) = &self.status_message {
                    ui.separator();
                    ui.label(msg);
                }
            });
        });
    }

    fn apply_rename_start_input(&mut self) {
        let trimmed = self.rename_start_input.trim();
        match trimmed.parse::<usize>() {
            Ok(value) if value >= 1 => {
                self.rename_start_value = value;
                self.rename_start_input = value.to_string();
                let message = format!("起始值已更新为 {}", value);
                log_message(&message);
                self.status_message = Some(message);
            }
            Ok(_) => {
                let message = "起始值必须大于等于 1".to_string();
                log_message(&format!("更新起始值失败: {}", message));
                self.status_message = Some(message);
            }
            Err(_) => {
                let message = "请输入有效的正整数作为起始值".to_string();
                log_message(&format!("更新起始值失败: {}", message));
                self.status_message = Some(message);
            }
        }
    }

    /// 左侧文件列表 + 拖拽排序 + 选择
    fn show_file_list_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("文件列表");
        ui.add_space(4.0);
        if ui.button("选择目录").clicked() {
            self.open_directory_dialog(ctx);
        }

        ui.add_space(8.0);
        if self.files.is_empty() {
            ui.label("请选择目录，显示其中的图片 / 文档 / PDF 文件。");
            return;
        }

        let mut selection_request: Option<u64> = None;

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let response = dnd(ui, "file_list_dnd").show_vec(
                    &mut self.files,
                    |ui, entry, handle, state| {
                        let selected = self.selected_file_id == Some(entry.id);
                        let tag = entry.file_type.tag_label();
                        let id = entry.id;

                        let fill = if selected {
                            ui.visuals().selection.bg_fill
                        } else {
                            ui.visuals().faint_bg_color
                        };

                        let row = egui::Frame::group(ui.style())
                            .inner_margin(egui::Margin::symmetric(8, 6))
                            .fill(fill)
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.horizontal(|ui| {
                                    handle.ui(ui, |ui| {
                                        ui.label("≡");
                                    });
                                    ui.label(format!("{:>3}", state.index + 1));
                                    ui.label(tag);
                                    ui.separator();
                                    ui.label(&entry.file_name);
                                });
                            });

                        let row_response = row.response.interact(egui::Sense::click());
                        if row_response.clicked() {
                            selection_request = Some(id);
                        }
                    },
                );

                if response.final_update().is_some() {
                    self.status_message = Some("文件顺序已更新".to_string());
                }
            });

        if let Some(request_id) = selection_request {
            self.select_file(request_id);
        } else if self.selected_entry().is_none() {
            self.selected_file_id = self.files.first().map(|entry| entry.id);
        }
    }

    /// 右侧预览区域
    fn show_preview_panel(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) {
        ui.heading("预览");
        ui.separator();
        if let Some(selected_idx) = self.selected_index() {
            self.ensure_preview_loaded(ctx, selected_idx);
            if let Some(entry) = self.files.get(selected_idx) {
                ui.label(format!("文件名: {}", entry.file_name));
                ui.label(format!("类型: {}", entry.file_type.display_name()));
                ui.label(format!("路径: {}", entry.path.display()));
                ui.separator();
                match entry.file_type {
                    FileType::Image => self.render_image_preview(ui, entry),
                    FileType::Text | FileType::Pdf | FileType::Docx => {
                        self.render_text_preview(ui, entry)
                    }
                    FileType::Unsupported(_) => {
                        ui.label("暂不支持该类型的内容预览。");
                    }
                }
            }
        } else {
            ui.label("单击左侧列表中的文件进行预览。");
        }
    }

    /// 图片预览（自动缩放以适配区域）
    fn render_image_preview(&self, ui: &mut egui::Ui, entry: &FileEntry) {
        if let Some(err) = &entry.preview_error {
            ui.label(format!("预览失败: {}", err));
            return;
        }

        if let Some(texture) = &entry.texture {
            egui::ScrollArea::both()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    UiUtils::draw_checkerboard_background(ui);
                    ui.add(
                        egui::Image::from_texture(texture)
                            .fit_to_original_size(1.0)
                            .texture_options(egui::TextureOptions {
                                magnification: egui::TextureFilter::Nearest,
                                minification: egui::TextureFilter::Nearest,
                                ..Default::default()
                            }),
                    );
                });
        } else {
            if entry.preview_loaded {
                ui.label("暂无可以显示的图片预览。");
            } else {
                ui.label("正在加载图片预览…");
            }
        }
    }

    /// 文本类文档预览：显示前若干字符
    fn render_text_preview(&self, ui: &mut egui::Ui, entry: &FileEntry) {
        if let Some(err) = &entry.preview_error {
            ui.label(format!("预览失败: {}", err));
        } else if let Some(text) = &entry.text_preview {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    ui.monospace(text);
                });
        } else {
            if entry.preview_loaded {
                ui.label("暂无可以显示的文本内容。");
            } else {
                ui.label("正在加载文本预览…");
            }
        }
    }

    /// 打开目录选择对话框
    fn open_directory_dialog(&mut self, ctx: &egui::Context) {
        log_message("打开目录选择对话框");
        if let Some(dir) = rfd::FileDialog::new().pick_folder() {
            let msg = format!("选择目录: {}", dir.display());
            log_message(&msg);
            if let Err(err) = self.load_directory(ctx, dir.as_path()) {
                self.status_message = Some(err);
            }
        } else {
            log_message("用户取消目录选择");
        }
    }

    /// 加载目录中的文件
    fn load_directory(&mut self, _ctx: &egui::Context, dir: &Path) -> Result<(), String> {
        let start_time = Instant::now();
        let msg = format!("开始加载目录: {}", dir.display());
        log_message(&msg);

        let mut entries = Vec::new();
        let dir_iter = fs::read_dir(dir).map_err(|e| format!("读取目录失败: {}", e))?;
        let mut next_id = 0_u64;

        for item in dir_iter {
            let item = item.map_err(|e| format!("读取目录项失败: {}", e))?;
            let path = item.path();
            if !path.is_file() {
                continue;
            }

            let Some(file_type) = Self::classify_file(&path) else {
                let msg = format!("忽略不支持的文件: {}", path.display());
                log_message(&msg);
                continue;
            };

            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| String::from("unknown"));
            let msg = format!("处理文件: {}", file_name);
            log_message(&msg);

            entries.push(FileEntry {
                id: {
                    let id = next_id;
                    next_id = next_id.wrapping_add(1);
                    id
                },
                path,
                file_name,
                file_type,
                texture: None,
                text_preview: None,
                preview_loaded: false,
                preview_error: None,
            });
        }

        if entries.is_empty() {
            log_message("目录中未找到支持的文件类型");
            return Err("目录中未找到支持的文件类型".to_string());
        }

        self.directory_path = Some(dir.to_path_buf());
        self.next_id = next_id;
        self.selected_file_id = entries.first().map(|entry| entry.id);
        self.files = entries;

        let message = format!("已找到 {} 个文件，预览将在选择时加载", self.files.len());
        self.status_message = Some(message);

        let elapsed = start_time.elapsed();
        let msg = format!(
            "目录索引完成，共 {} 个文件，耗时 {:.2?}",
            self.files.len(),
            elapsed
        );
        log_message(&msg);
        Ok(())
    }

    /// 根据扩展名判断类型
    fn classify_file(path: &Path) -> Option<FileType> {
        let ext = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|s| s.to_ascii_lowercase())?;

        if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
            Some(FileType::Image)
        } else if TEXT_DOC_EXTENSIONS.contains(&ext.as_str()) {
            Some(FileType::Text)
        } else if DOCX_EXTENSIONS.contains(&ext.as_str()) {
            Some(FileType::Docx)
        } else if PDF_EXTENSIONS.contains(&ext.as_str()) {
            Some(FileType::Pdf)
        } else if LEGACY_DOC_EXTENSIONS.contains(&ext.as_str()) {
            Some(FileType::Unsupported(ext))
        } else {
            Some(FileType::Unsupported(ext))
        }
    }

    fn ensure_preview_loaded(&mut self, ctx: &egui::Context, index: usize) {
        if index >= self.files.len() {
            return;
        }
        if self.files[index].preview_loaded {
            return;
        }

        let (path, file_name, file_type) = {
            let entry = &self.files[index];
            (
                entry.path.clone(),
                entry.file_name.clone(),
                entry.file_type.clone(),
            )
        };

        log_message(&format!("开始加载预览: {}", file_name));

        enum PreviewOutcome {
            Image(egui::TextureHandle),
            Text(String),
        }

        let result: Result<PreviewOutcome, String> = match file_type {
            FileType::Image => match image::open(&path) {
                Ok(img) => {
                    let texture = load_texture_from_image(&img, ctx);
                    Ok(PreviewOutcome::Image(texture))
                }
                Err(e) => Err(format!("加载图片失败: {}", e)),
            },
            FileType::Text => match Self::read_text_preview(&path) {
                Ok(text) => Ok(PreviewOutcome::Text(text)),
                Err(e) => Err(format!("读取文本失败: {}", e)),
            },
            FileType::Pdf => match Self::read_pdf_preview(&path) {
                Ok(text) => Ok(PreviewOutcome::Text(text)),
                Err(e) => Err(format!("读取 PDF 失败: {}", e)),
            },
            FileType::Docx => match Self::read_docx_preview(&path) {
                Ok(text) => Ok(PreviewOutcome::Text(text)),
                Err(e) => Err(format!("读取 DOCX 失败: {}", e)),
            },
            FileType::Unsupported(ext) => Err(format!("不支持的文件类型: {}", ext)),
        };

        let entry = &mut self.files[index];
        entry.preview_loaded = true;

        match result {
            Ok(PreviewOutcome::Image(texture)) => {
                entry.texture = Some(texture);
                entry.text_preview = None;
                entry.preview_error = None;
                log_message(&format!("图片预览加载完成: {}", file_name));
            }
            Ok(PreviewOutcome::Text(text)) => {
                entry.text_preview = Some(text);
                entry.texture = None;
                entry.preview_error = None;
                log_message(&format!("文本预览加载完成: {}", file_name));
            }
            Err(err) => {
                entry.texture = None;
                entry.text_preview = None;
                entry.preview_error = Some(err.clone());
                log_message(&format!("预览加载失败: {} -> {}", file_name, err));
                self.status_message = Some(format!("预览失败: {}", err));
            }
        }
    }

    /// 文本预览：读取前若干字符
    fn read_text_preview(path: &Path) -> Result<String, String> {
        let msg = format!("读取文本预览: {}", path.display());
        log_message(&msg);
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut buffer = Vec::with_capacity(MAX_PREVIEW_BYTES);
        std::io::Read::by_ref(&mut file)
            .take(MAX_PREVIEW_BYTES as u64)
            .read_to_end(&mut buffer)
            .map_err(|e| e.to_string())?;
        let text = String::from_utf8_lossy(&buffer).to_string();
        let msg = format!("文本预览成功，长度 {}", text.len());
        log_message(&msg);
        Ok(Self::truncate_preview(text))
    }

    fn read_pdf_preview(path: &Path) -> Result<String, String> {
        let msg = format!("读取 PDF 预览: {}", path.display());
        log_message(&msg);
        let text = pdf_extract::extract_text(path).map_err(|e| e.to_string())?;
        if text.trim().is_empty() {
            log_message("PDF 预览为空文本");
            Ok(String::from("（PDF 不包含可提取的文本内容）"))
        } else {
            let msg = format!("PDF 预览成功，原始长度 {}", text.len());
            log_message(&msg);
            Ok(Self::truncate_preview(text))
        }
    }

    fn read_docx_preview(path: &Path) -> Result<String, String> {
        let msg = format!("读取 DOCX 预览: {}", path.display());
        log_message(&msg);
        let text = docx_lite::extract_text(path).map_err(|e| e.to_string())?;
        if text.trim().is_empty() {
            log_message("DOCX 预览为空文本");
            Ok(String::from("（DOCX 文件没有可展示的文本内容）"))
        } else {
            let msg = format!("DOCX 预览成功，原始长度 {}", text.len());
            log_message(&msg);
            Ok(Self::truncate_preview(text))
        }
    }

    fn truncate_preview(text: String) -> String {
        if text.len() <= MAX_PREVIEW_CHARS {
            return text;
        }

        let mut cutoff = MAX_PREVIEW_CHARS;
        while cutoff > 0 && !text.is_char_boundary(cutoff) {
            cutoff -= 1;
        }
        let mut truncated = text[..cutoff].to_string();
        truncated.push_str("\n…（内容已截断预览）");
        truncated
    }

    fn select_file(&mut self, id: u64) {
        if self.files.iter().any(|entry| entry.id == id) {
            self.selected_file_id = Some(id);
        }
    }

    fn selected_index(&self) -> Option<usize> {
        let id = self.selected_file_id?;
        self.files.iter().position(|entry| entry.id == id)
    }

    fn selected_entry(&self) -> Option<&FileEntry> {
        self.selected_index().and_then(|idx| self.files.get(idx))
    }

    /// 根据当前顺序重命名文件（两阶段，避免命名冲突）
    fn rename_files(&mut self) -> Result<(), String> {
        log_message("开始重命名文件");
        let dir = self
            .directory_path
            .clone()
            .ok_or_else(|| "未选择任何目录".to_string())?;
        if self.files.is_empty() {
            log_message("没有文件可供重命名");
            return Err("没有可重命名的文件".to_string());
        }

        let mut final_paths = Vec::with_capacity(self.files.len());
        let originals: HashSet<PathBuf> =
            self.files.iter().map(|entry| entry.path.clone()).collect();

        let file_count = self.files.len();
        let start_number = self.rename_start_value.max(1);
        let max_number = start_number.saturating_add(file_count.saturating_sub(1));
        let width = max_number.to_string().len();
        log_message(&format!(
            "重命名起始值: {}，总计 {} 个文件，序号宽度 {}",
            start_number, file_count, width
        ));

        for (idx, entry) in self.files.iter().enumerate() {
            let ext = entry
                .path
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            let current_number = start_number.saturating_add(idx);
            let stem = format!("{:0width$}", current_number, width = width);

            let new_name = if ext.is_empty() {
                stem.clone()
            } else {
                format!("{}.{}", stem, ext)
            };

            let final_path = dir.join(&new_name);
            if final_path.exists() && !originals.contains(&final_path) {
                let msg = format!("重命名目标已存在: {}", final_path.display());
                log_message(&msg);
                return Err(format!("目标文件已存在: {}", final_path.display()));
            }

            final_paths.push((idx, final_path, new_name));
        }

        // 第一步：全部改为唯一的临时文件名
        for (idx, entry) in self.files.iter_mut().enumerate() {
            let temp_name = format!("__tmp_order_{:04}_{}", idx, entry.file_name);
            let temp_path = dir.join(&temp_name);
            fs::rename(&entry.path, &temp_path)
                .map_err(|e| format!("重命名临时文件失败: {}", e))?;
            entry.path = temp_path;
            entry.file_name = temp_name;
        }

        // 第二步：按顺序命名为起始值递增的序号
        for (idx, final_path, new_name) in final_paths {
            fs::rename(&self.files[idx].path, &final_path)
                .map_err(|e| format!("最终重命名失败: {}", e))?;
            self.files[idx].path = final_path;
            self.files[idx].file_name = new_name;
        }

        let summary = format!(
            "已按顺序重命名 {} 个文件（起始值 {}）",
            self.files.len(),
            start_number
        );
        log_message(&summary);
        self.status_message = Some(summary);
        Ok(())
    }
}

/// 文件类型，用于展示标签与预览选择
#[derive(Clone, Debug)]
enum FileType {
    Image,
    Text,
    Pdf,
    Docx,
    Unsupported(String),
}

impl FileType {
    fn display_name(&self) -> String {
        match self {
            FileType::Image => "图片".to_string(),
            FileType::Text => "文本".to_string(),
            FileType::Pdf => "PDF".to_string(),
            FileType::Docx => "DOCX 文档".to_string(),
            FileType::Unsupported(ext) => format!("不支持的文件类型（{}）", ext),
        }
    }

    fn tag_label(&self) -> String {
        match self {
            FileType::Image => "[IMG]".to_string(),
            FileType::Text => "[TXT]".to_string(),
            FileType::Pdf => "[PDF]".to_string(),
            FileType::Docx => "[DOCX]".to_string(),
            FileType::Unsupported(ext) => format!("[{}]", ext.to_uppercase()),
        }
    }
}

/// 单个文件条目的状态
struct FileEntry {
    id: u64,
    path: PathBuf,
    file_name: String,
    file_type: FileType,
    texture: Option<egui::TextureHandle>,
    text_preview: Option<String>,
    preview_loaded: bool,
    preview_error: Option<String>,
}

impl PartialEq for FileEntry {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for FileEntry {}

impl Hash for FileEntry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}
