#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;

/// Usage: `openrgb-egui [host[:port]]`
fn main() -> eframe::Result {
    let server = std::env::args().nth(1);
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("OpenRGB")
            .with_app_id("openrgb-egui")
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "openrgb-egui",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc, server)))),
    )
}
