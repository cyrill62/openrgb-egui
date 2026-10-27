#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ui;

/// Usage: `pimpbox-rgb [host[:port]]`
fn main() -> eframe::Result {
    let server = std::env::args().nth(1);
    let icon = app_icon();
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("PimpBox RGB")
            .with_app_id("openrgb-egui")
            .with_icon(icon)
            .with_inner_size([1280.0, 820.0])
            .with_min_inner_size([960.0, 620.0]),
        ..Default::default()
    };
    eframe::run_native(
        "PimpBox RGB",
        options,
        Box::new(|cc| Ok(Box::new(ui::App::new(cc, server)))),
    )
}

fn app_icon() -> eframe::egui::IconData {
    let image = egui_extras::image::load_svg_bytes(
        include_bytes!("../assets/logo.svg"),
        &Default::default(),
    )
    .expect("valid app logo SVG");
    let side = image.size[0].max(image.size[1]);
    let x_offset = (side - image.size[0]) / 2;
    let y_offset = (side - image.size[1]) / 2;
    let mut rgba = vec![0; side * side * 4];
    for y in 0..image.size[1] {
        for x in 0..image.size[0] {
            let src = image.pixels[y * image.size[0] + x].to_srgba_unmultiplied();
            let start = ((y + y_offset) * side + x + x_offset) * 4;
            rgba[start..start + 4].copy_from_slice(&src);
        }
    }
    eframe::egui::IconData {
        rgba,
        width: side as u32,
        height: side as u32,
    }
}
