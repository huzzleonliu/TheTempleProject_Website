use image::DynamicImage;

/// 将 `DynamicImage` 转换为 `egui` 可用的纹理
pub fn load_texture_from_image(img: &DynamicImage, ctx: &egui::Context) -> egui::TextureHandle {
    let rgba_image = img.to_rgba8();
    let size = [rgba_image.width() as usize, rgba_image.height() as usize];
    let pixels = rgba_image.into_raw();

    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, &pixels);
    let texture_options = egui::TextureOptions {
        magnification: egui::TextureFilter::Nearest,
        minification: egui::TextureFilter::Nearest,
        ..Default::default()
    };
    ctx.load_texture("loaded_image", color_image, texture_options)
}

/// 图片浏览器状态：缩放 + 平移 + “首次适应屏幕”标记
#[derive(Debug, Clone)]
pub struct ImageViewState {
    pub zoom: f32,
    pub pan: egui::Vec2,
    fitted_for: Option<u64>,
}

impl Default for ImageViewState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            pan: egui::Vec2::ZERO,
            fitted_for: None,
        }
    }
}

impl ImageViewState {
    #[allow(dead_code)]
    pub fn reset_fit(&mut self) {
        self.fitted_for = None;
    }
}

/// 显示可缩放/可拖拽的图片浏览器。
///
/// - **缩放**：鼠标滚轮（在图片区域上）
/// - **平移**：按住左键拖拽
/// - **适应屏幕**：双击图片区域，或首次加载该图片时自动适配
pub fn show_image_browser(
    ui: &mut egui::Ui,
    texture: &egui::TextureHandle,
    state: &mut ImageViewState,
    image_id: u64,
) {
    let avail = ui.available_size();
    let (rect, response) = ui.allocate_exact_size(avail, egui::Sense::drag());

    UiUtils::draw_checkerboard_background_rect(ui.painter(), rect);

    let tex_size = texture.size_vec2();
    if tex_size.x <= 0.0 || tex_size.y <= 0.0 {
        return;
    }

    // 首次加载时：按可用区域做 fit-to-screen（保持比例）
    if state.fitted_for != Some(image_id) {
        let fit_x = rect.width() / tex_size.x;
        let fit_y = rect.height() / tex_size.y;
        let fit = fit_x.min(fit_y).max(0.01);
        state.zoom = fit;
        state.pan = egui::Vec2::ZERO;
        state.fitted_for = Some(image_id);
    }

    // 双击：重新适应屏幕
    if response.double_clicked() {
        state.fitted_for = None;
    }

    // 拖拽：平移
    if response.dragged() {
        state.pan += response.drag_delta();
    }

    // 滚轮：缩放（围绕鼠标位置缩放）
    if response.hovered() {
        let scroll = ui.input(|i| i.smooth_scroll_delta.y);
        if scroll.abs() > 0.0 {
            let old_zoom = state.zoom;
            // 平滑缩放：把每次滚轮的缩放幅度压小一些，避免“缩放倍数过大”
            // 经验值：scroll ~= 120 时约 6% 缩放
            let factor = (scroll / 2000.0).exp();
            let new_zoom = (state.zoom * factor).clamp(0.02, 40.0);

            if let Some(pointer) = ui.input(|i| i.pointer.hover_pos()) {
                // 以鼠标为缩放中心：缩放前后鼠标下的“世界坐标点”保持不变
                // screen = center + pan + world * zoom
                let center = rect.center();
                let world = (pointer - center - state.pan) / old_zoom;
                state.zoom = new_zoom;
                state.pan = pointer - center - world * new_zoom;
            } else {
                state.zoom = new_zoom;
            }
        }
    }

    // 画图（clip 到 rect）
    let painter = ui.painter();
    let img_size = tex_size * state.zoom;
    let center = rect.center() + state.pan;
    let img_rect = egui::Rect::from_center_size(center, img_size);
    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));

    let clip = painter.with_clip_rect(rect);
    clip.image(texture.id(), img_rect, uv, egui::Color32::WHITE);
}

/// UI工具函数（图片预览相关）
pub struct UiUtils;

impl UiUtils {
    /// 绘制棋盘格背景
    #[allow(dead_code)]
    pub fn draw_checkerboard_background(ui: &mut egui::Ui) {
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();

        let checker_size = 20.0;
        let cols = (rect.width() / checker_size).ceil() as i32;
        let rows = (rect.height() / checker_size).ceil() as i32;

        for row in 0..rows {
            for col in 0..cols {
                let x = rect.min.x + col as f32 * checker_size;
                let y = rect.min.y + row as f32 * checker_size;

                let checker_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x, y),
                    egui::Vec2::new(checker_size, checker_size),
                );

                let is_light = (row + col) % 2 == 0;
                let color = if is_light {
                    egui::Color32::from_gray(200)
                } else {
                    egui::Color32::from_gray(150)
                };

                painter.rect_filled(checker_rect, 0.0, color);
            }
        }
    }

    /// 在指定 rect 内绘制棋盘格背景（避免依赖 `allocate_ui_at_rect`）。
    pub fn draw_checkerboard_background_rect(painter: &egui::Painter, rect: egui::Rect) {
        let checker_size = 20.0;
        let cols = (rect.width() / checker_size).ceil() as i32;
        let rows = (rect.height() / checker_size).ceil() as i32;

        for row in 0..rows {
            for col in 0..cols {
                let x = rect.min.x + col as f32 * checker_size;
                let y = rect.min.y + row as f32 * checker_size;

                let checker_rect = egui::Rect::from_min_size(
                    egui::Pos2::new(x, y),
                    egui::Vec2::new(checker_size, checker_size),
                );

                let is_light = (row + col) % 2 == 0;
                let color = if is_light {
                    egui::Color32::from_gray(200)
                } else {
                    egui::Color32::from_gray(150)
                };

                painter.rect_filled(checker_rect, 0.0, color);
            }
        }
    }
}


