mod layout;
mod logic;
mod utils;

use layout::MainWindow;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Image Viewer",
        options,
        Box::new(|cc| {
            layout::configure_fonts(&cc.egui_ctx);
            Ok(Box::new(MainWindow::default()))
        }),
    )
}
