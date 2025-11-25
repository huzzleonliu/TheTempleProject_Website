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

/// UI工具函数
pub struct UiUtils;

impl UiUtils {
    /// 绘制棋盘格背景
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
}
