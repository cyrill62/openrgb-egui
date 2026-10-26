use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, FontId, Layout, Pos2, Rect, RichText, Sense,
    Stroke, StrokeKind, Ui, pos2, vec2,
};
use egui_material_icons::icons::*;

use pimpbox_rgb::protocol::{ColorMode, Controller, Direction, Rgb, Zone, ZoneType, mode_flags};
use pimpbox_rgb::worker::Command;

use super::theme::{self, ACCENT, TEXT, TEXT_DIM, TEXT_FAINT};
use super::widgets::{self, Hsv};
use super::{App, DeviceTab};

pub fn page(app: &mut App, ui: &mut Ui, dev: usize) {
    let Some(d) = app.devices.get(dev) else {
        return;
    };

    header(ui, d);
    ui.add_space(14.0);

    let mut tab = app.tab;
    widgets::segmented(
        ui,
        &mut tab,
        &[
            (DeviceTab::Lighting, "Lighting"),
            (DeviceTab::Leds, "LEDs"),
            (DeviceTab::Zones, "Zones"),
            (DeviceTab::Info, "Info"),
        ],
    );
    app.tab = tab;
    ui.add_space(14.0);

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| match app.tab {
            DeviceTab::Lighting => lighting(app, ui, dev),
            DeviceTab::Leds => leds(app, ui, dev),
            DeviceTab::Zones => zones(app, ui, dev),
            DeviceTab::Info => info(app, ui, dev),
        });
}

fn header(ui: &mut Ui, d: &Controller) {
    ui.horizontal(|ui| {
        let tint = theme::to_c32(theme::accent_of(&d.colors));
        widgets::icon_badge(ui, widgets::device_icon(d.kind), tint, 56.0);
        ui.add_space(6.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 2.0;
            ui.label(
                RichText::new(&d.name)
                    .text_style(theme::title())
                    .color(TEXT),
            );
            let mut sub = vec![d.kind.label().to_string()];
            if !d.vendor.is_empty() {
                sub.insert(0, d.vendor.clone());
            }
            sub.push(format!("{} LEDs", d.colors.len()));
            sub.push(format!("{} zones", d.zones.len()));
            ui.label(RichText::new(sub.join("  ·  ")).color(TEXT_DIM));
        });
    });
    ui.add_space(10.0);
    let w = ui.available_width();
    widgets::led_bar(ui, &d.colors, vec2(w, 6.0));
}

// ---------------------------------------------------------------------------
// Lighting tab
// ---------------------------------------------------------------------------

fn lighting(app: &mut App, ui: &mut Ui, dev: usize) {
    let d = &app.devices[dev];
    let active = usize::try_from(d.active_mode).ok();
    let modes: Vec<(usize, String, bool)> = d
        .modes
        .iter()
        .enumerate()
        .map(|(i, m)| (i, m.name.clone(), m.is_direct_like()))
        .collect();

    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::section_label(ui, "Mode");
        ui.add_space(4.0);
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 8.0);
            for (i, name, direct) in &modes {
                let icon = mode_icon(name, *direct);
                if widgets::chip(ui, active == Some(*i), name, Some(icon)).clicked() {
                    app.set_mode(dev, *i, false);
                }
            }
        });
    });

    let Some(mode) = app.devices[dev].active_mode().cloned() else {
        return;
    };
    ui.add_space(12.0);

    let has_settings = mode.has(mode_flags::HAS_SPEED)
        || mode.has(mode_flags::HAS_BRIGHTNESS)
        || !mode.directions().is_empty();
    let has_colors = mode.has(mode_flags::HAS_MODE_SPECIFIC_COLOR)
        || mode.has(mode_flags::HAS_RANDOM_COLOR)
        || mode.has(mode_flags::HAS_PER_LED_COLOR);

    if has_settings {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            widgets::section_label(ui, "Parameters");
            ui.add_space(4.0);
            if mode.has(mode_flags::HAS_SPEED) {
                let mut v = mode.speed;
                let r = widgets::slider_row(
                    ui,
                    ICON_SPEED,
                    "Speed",
                    &mut v,
                    (mode.speed_min, mode.speed_max),
                );
                if r.changed() {
                    app.update_active_mode(dev, |m| m.speed = v);
                }
            }
            if mode.has(mode_flags::HAS_BRIGHTNESS) {
                let mut v = mode.brightness;
                let r = widgets::slider_row(
                    ui,
                    ICON_LIGHT_MODE,
                    "Brightness",
                    &mut v,
                    (mode.brightness_min, mode.brightness_max),
                );
                if r.changed() {
                    app.update_active_mode(dev, |m| m.brightness = v);
                }
            }
            let dirs = mode.directions();
            if !dirs.is_empty() {
                ui.horizontal(|ui| {
                    ui.label(ICON_SWAP_HORIZ.rich_text().size(18.0).color(TEXT_DIM));
                    ui.add_sized(vec2(90.0, 24.0), egui::Label::new("Direction"));
                    let mut dir = mode.direction;
                    let opts: Vec<(Direction, &str)> =
                        dirs.iter().map(|d| (*d, d.label())).collect();
                    if widgets::segmented(ui, &mut dir, &opts) {
                        app.update_active_mode(dev, |m| m.direction = dir);
                    }
                });
            }
        });
        ui.add_space(12.0);
    }

    if has_colors {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                widgets::section_label(ui, "Color");
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let mut opts = Vec::new();
                    if mode.has(mode_flags::HAS_PER_LED_COLOR) {
                        opts.push((ColorMode::PerLed, "Per LED"));
                    }
                    if mode.has(mode_flags::HAS_MODE_SPECIFIC_COLOR) {
                        opts.push((ColorMode::ModeSpecific, "Custom"));
                    }
                    if mode.has(mode_flags::HAS_RANDOM_COLOR) {
                        opts.push((ColorMode::Random, "Random"));
                    }
                    if opts.len() > 1 {
                        let mut cm = mode.color_mode;
                        if widgets::segmented(ui, &mut cm, &opts) {
                            app.update_active_mode(dev, |m| m.color_mode = cm);
                        }
                    }
                });
            });
            ui.add_space(6.0);
            match mode.color_mode {
                ColorMode::PerLed => per_led_fill(app, ui, dev),
                ColorMode::ModeSpecific => mode_colors(app, ui, dev),
                ColorMode::Random => {
                    ui.label(RichText::new("The device picks colors at random.").color(TEXT_DIM));
                }
                ColorMode::None => {
                    ui.label(RichText::new("This mode uses built-in colors.").color(TEXT_DIM));
                }
            }
        });
        ui.add_space(12.0);
    }

    if mode.has(mode_flags::MANUAL_SAVE) && app.proto() >= 3 {
        ui.horizontal(|ui| {
            let saved = widgets::secondary_button(ui, ICON_SAVE, "Save to device")
                .on_hover_text(
                    "Write this mode to the device's onboard memory so it persists without OpenRGB",
                )
                .clicked();
            if saved && let Ok(idx) = usize::try_from(app.devices[dev].active_mode) {
                app.set_mode(dev, idx, true);
                app.toast(ui.ctx(), "Saved to device", false);
            }
        });
    }
}

fn mode_icon(name: &str, direct: bool) -> egui_material_icons::MaterialIcon {
    let n = name.to_ascii_lowercase();
    if direct || n.contains("direct") || n.contains("custom") {
        ICON_PALETTE
    } else if n.contains("static") {
        ICON_RADIO_BUTTON_CHECKED
    } else if n.contains("breath") || n.contains("pulse") {
        ICON_BLUR_ON
    } else if n.contains("rainbow") || n.contains("wave") {
        ICON_WAVES
    } else if n.contains("spectrum") || n.contains("cycle") {
        ICON_GRADIENT
    } else if n.contains("flash") || n.contains("strobe") || n.contains("blink") {
        ICON_BOLT
    } else if n == "off" {
        ICON_POWER_SETTINGS_NEW
    } else if n.contains("star") || n.contains("twinkle") || n.contains("random") {
        ICON_AUTO_AWESOME
    } else {
        ICON_FLARE
    }
}

fn per_led_fill(app: &mut App, ui: &mut Ui, dev: usize) {
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(260.0);
            let mut hsv = app.settings.paint;
            if widgets::color_picker(ui, &mut hsv, 260.0) {
                app.settings.paint = hsv;
                app.fill_device(dev, hsv.rgb());
            }
        });
        ui.add_space(16.0);
        ui.vertical(|ui| {
            ui.label(RichText::new("Fill the whole device").strong().color(TEXT));
            ui.label(
                RichText::new(
                    "Changes apply live. Use the LEDs tab to color individual LEDs or keys.",
                )
                .color(TEXT_DIM),
            );
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if widgets::primary_button(ui, ICON_COLORIZE, "Apply color").clicked() {
                    app.fill_device(dev, app.settings.paint.rgb());
                }
                if widgets::secondary_button(ui, ICON_POWER_SETTINGS_NEW, "Turn off").clicked() {
                    app.fill_device(dev, Rgb::BLACK);
                }
            });
        });
    });
}

fn mode_colors(app: &mut App, ui: &mut Ui, dev: usize) {
    let Some(mode) = app.devices[dev].active_mode().cloned() else {
        return;
    };
    let mut colors = mode.colors.clone();
    let mut changed = false;
    ui.horizontal_wrapped(|ui| {
        for (i, c) in colors.iter_mut().enumerate() {
            ui.vertical(|ui| {
                changed |= widgets::color_button(ui, ("mode_color", dev, i), c, vec2(56.0, 56.0));
                ui.label(RichText::new(c.hex()).small().monospace().color(TEXT_FAINT));
            });
        }
        let len = colors.len() as u32;
        ui.vertical(|ui| {
            if len < mode.colors_max {
                let b =
                    egui::Button::new(ICON_ADD.rich_text().size(22.0)).min_size(vec2(56.0, 56.0));
                if ui.add(b).on_hover_text("Add color").clicked() {
                    colors.push(colors.last().copied().unwrap_or(Rgb::WHITE));
                    changed = true;
                }
            }
            if len > mode.colors_min.max(1) {
                let b = egui::Button::new(ICON_CLOSE.rich_text().size(16.0).color(TEXT_DIM))
                    .frame(false);
                if ui.add(b).on_hover_text("Remove last color").clicked() {
                    colors.pop();
                    changed = true;
                }
            }
        });
    });
    if changed {
        app.update_active_mode(dev, |m| m.colors = colors);
    }
}

// ---------------------------------------------------------------------------
// LEDs tab
// ---------------------------------------------------------------------------

fn leds(app: &mut App, ui: &mut Ui, dev: usize) {
    if app.devices[dev].colors.is_empty() {
        widgets::empty_state(
            ui,
            ICON_GRID_OFF,
            "No LEDs",
            "This device exposes no addressable LEDs.",
        );
        return;
    }
    let picker_w = 268.0;
    let map_w = (ui.available_width() - picker_w - 56.0).max(320.0);
    ui.horizontal_top(|ui| {
        ui.vertical(|ui| {
            ui.set_width(map_w);
            led_map(app, ui, dev, map_w);
        });
        ui.add_space(12.0);
        ui.vertical(|ui| {
            theme::card().show(ui, |ui| {
                ui.set_width(picker_w - 36.0);
                paint_panel(app, ui, dev, picker_w - 36.0);
            });
        });
    });
}

fn paint_panel(app: &mut App, ui: &mut Ui, dev: usize, w: f32) {
    let n = app.devices[dev].colors.len();
    let sel = app.selection.len();
    widgets::section_label(ui, "Paint");
    ui.add_space(2.0);
    let mut hsv: Hsv = app.settings.paint;
    if widgets::color_picker(ui, &mut hsv, w) {
        app.settings.paint = hsv;
        if sel > 0 {
            apply_to_selection(app, dev, hsv.rgb());
        }
    }
    ui.add_space(8.0);
    ui.label(
        RichText::new(if sel == 0 {
            "Click or drag over LEDs to select. Ctrl/Shift-click to toggle.".to_string()
        } else {
            format!("{sel} of {n} LEDs selected — color changes apply live.")
        })
        .small()
        .color(TEXT_DIM),
    );
    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        if widgets::primary_button(
            ui,
            ICON_COLORIZE,
            if sel > 0 { "Apply" } else { "Fill all" },
        )
        .clicked()
        {
            if sel > 0 {
                apply_to_selection(app, dev, hsv.rgb());
            } else {
                app.fill_device(dev, hsv.rgb());
            }
        }
        if widgets::secondary_button(ui, ICON_SELECT_ALL, "All").clicked() {
            app.selection = (0..n).collect();
        }
        if sel > 0 && widgets::secondary_button(ui, ICON_DESELECT, "None").clicked() {
            app.selection.clear();
        }
    });
}

fn apply_to_selection(app: &mut App, dev: usize, c: Rgb) {
    let mut colors = app.devices[dev].colors.clone();
    for &i in &app.selection {
        if let Some(x) = colors.get_mut(i) {
            *x = c;
        }
    }
    app.set_leds(dev, colors);
}

/// One drawable LED: controller-wide index plus where to draw it.
struct Cell {
    led: usize,
    rect: Rect,
    round: bool,
}

fn layout_zone(z: &Zone, offset: usize, origin: Pos2, width: f32) -> (Vec<Cell>, f32) {
    let mut cells = Vec::new();
    match (&z.matrix, z.kind) {
        (Some(m), ZoneType::Matrix) if m.width > 0 => {
            let gap = 4.0;
            let size = ((width - gap * (m.width as f32 - 1.0)) / m.width as f32).clamp(14.0, 48.0);
            for row in 0..m.height {
                for col in 0..m.width {
                    if let Some(i) = m.get(row, col) {
                        let min =
                            origin + vec2(col as f32 * (size + gap), row as f32 * (size + gap));
                        cells.push(Cell {
                            led: offset + i as usize,
                            rect: Rect::from_min_size(min, vec2(size, size)),
                            round: false,
                        });
                    }
                }
            }
            (cells, m.height as f32 * (size + gap) - gap)
        }
        _ => {
            let n = z.leds_count as usize;
            let size = 22.0;
            let gap = 8.0;
            let per_row = (((width + gap) / (size + gap)).floor() as usize).max(1);
            for i in 0..n {
                let (row, col) = (i / per_row, i % per_row);
                let min = origin + vec2(col as f32 * (size + gap), row as f32 * (size + gap));
                cells.push(Cell {
                    led: offset + i,
                    rect: Rect::from_min_size(min, vec2(size, size)),
                    round: true,
                });
            }
            let rows = n.div_ceil(per_row).max(1);
            (cells, rows as f32 * (size + gap) - gap)
        }
    }
}

fn led_map(app: &mut App, ui: &mut Ui, dev: usize, width: f32) {
    let d = app.devices[dev].clone();
    let offsets = d.zone_offsets();
    for (zi, z) in d.zones.iter().enumerate() {
        if z.leds_count == 0 {
            continue;
        }
        theme::card().show(ui, |ui| {
            let inner_w = width - 36.0;
            ui.set_width(inner_w);
            ui.horizontal(|ui| {
                ui.label(RichText::new(&z.name).strong().color(TEXT));
                ui.label(
                    RichText::new(format!("{} LEDs", z.leds_count))
                        .small()
                        .color(TEXT_FAINT),
                );
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let b = egui::Button::new(RichText::new("Select zone").small()).small();
                    if ui.add(b).clicked() {
                        let range = offsets[zi]..offsets[zi] + z.leds_count as usize;
                        let all = range.clone().all(|i| app.selection.contains(&i));
                        for i in range {
                            if all {
                                app.selection.remove(&i);
                            } else {
                                app.selection.insert(i);
                            }
                        }
                    }
                });
            });
            ui.add_space(6.0);

            let origin = ui.cursor().min;
            let (cells, h) = layout_zone(z, offsets[zi], origin, inner_w);
            let (rect, resp) = ui.allocate_exact_size(vec2(inner_w, h), Sense::click_and_drag());
            let hit = |p: Pos2| {
                cells
                    .iter()
                    .find(|c| c.rect.expand(2.0).contains(p))
                    .map(|c| c.led)
            };

            let mods = ui.input(|i| i.modifiers);
            if resp.drag_started()
                && !(mods.shift || mods.command)
                && let Some(p) = resp.interact_pointer_pos()
                && hit(p).is_some()
            {
                app.selection.clear();
            }
            if resp.dragged()
                && let Some(led) = resp.interact_pointer_pos().and_then(hit)
            {
                app.selection.insert(led);
            }
            if resp.clicked()
                && let Some(p) = resp.interact_pointer_pos()
            {
                match hit(p) {
                    Some(led) if mods.shift || mods.command => {
                        if !app.selection.remove(&led) {
                            app.selection.insert(led);
                        }
                    }
                    Some(led) => {
                        app.selection.clear();
                        app.selection.insert(led);
                    }
                    None => app.selection.clear(),
                }
            }
            let hovered = resp.hover_pos().and_then(hit);
            if let Some(led) = hovered
                && let Some(l) = d.leds.get(led)
            {
                let c = d.colors.get(led).copied().unwrap_or_default();
                resp.clone()
                    .on_hover_text(format!("{}\n{}", l.name, c.hex()));
            }

            if ui.is_rect_visible(rect) {
                let p = ui.painter();
                for cell in &cells {
                    let c = d.colors.get(cell.led).copied().unwrap_or_default();
                    let fill = theme::led_display(c);
                    let selected = app.selection.contains(&cell.led);
                    let is_hover = hovered == Some(cell.led);
                    let r = if cell.round {
                        CornerRadius::same((cell.rect.width() / 2.0) as u8)
                    } else {
                        CornerRadius::same(6)
                    };
                    // Soft glow beneath lit LEDs.
                    if c.r.max(c.g).max(c.b) > 40 {
                        p.rect_filled(cell.rect.expand(3.0), r + 3, fill.gamma_multiply(0.18));
                    }
                    p.rect_filled(cell.rect, r, fill);
                    if !cell.round {
                        // Keycap bevel.
                        let top = Rect::from_min_max(
                            cell.rect.min,
                            pos2(cell.rect.max.x, cell.rect.min.y + cell.rect.height() * 0.45),
                        );
                        p.rect_filled(top, r, Color32::from_white_alpha(14));
                        let label =
                            key_label(d.leds.get(cell.led).map(|l| l.name.as_str()).unwrap_or(""));
                        if !label.is_empty() && cell.rect.width() >= 26.0 {
                            let font =
                                FontId::proportional((cell.rect.width() * 0.26).clamp(8.0, 11.0));
                            let lum = 0.3 * c.r as f32 + 0.59 * c.g as f32 + 0.11 * c.b as f32;
                            let tc = if lum > 150.0 {
                                Color32::from_black_alpha(190)
                            } else {
                                Color32::from_white_alpha(200)
                            };
                            p.text(cell.rect.center(), Align2::CENTER_CENTER, label, font, tc);
                        }
                    }
                    if selected {
                        p.rect_stroke(
                            cell.rect.expand(2.5),
                            r + 2,
                            Stroke::new(2.0, Color32::WHITE),
                            StrokeKind::Outside,
                        );
                    } else if is_hover {
                        p.rect_stroke(
                            cell.rect.expand(1.5),
                            r + 1,
                            Stroke::new(1.5, ACCENT),
                            StrokeKind::Outside,
                        );
                    }
                }
            }
        });
        ui.add_space(10.0);
    }
}

fn key_label(name: &str) -> String {
    let n = name.strip_prefix("Key: ").unwrap_or(name);
    if n.chars().count() <= 5 {
        n.to_string()
    } else {
        String::new()
    }
}

// ---------------------------------------------------------------------------
// Zones tab
// ---------------------------------------------------------------------------

fn zones(app: &mut App, ui: &mut Ui, dev: usize) {
    let d = app.devices[dev].clone();
    let offsets = d.zone_offsets();
    if d.zones.is_empty() {
        widgets::empty_state(ui, ICON_GRID_OFF, "No zones", "This device has no zones.");
        return;
    }
    for (zi, z) in d.zones.iter().enumerate() {
        theme::card().show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal(|ui| {
                let icon = match z.kind {
                    ZoneType::Single => ICON_RADIO_BUTTON_CHECKED,
                    ZoneType::Linear => ICON_LINEAR_SCALE,
                    ZoneType::Matrix => ICON_GRID_ON,
                };
                ui.label(icon.rich_text().size(22.0).color(ACCENT));
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing.y = 1.0;
                    ui.label(RichText::new(&z.name).strong().color(TEXT));
                    let kind = match z.kind {
                        ZoneType::Single => "Single",
                        ZoneType::Linear => "Linear",
                        ZoneType::Matrix => "Matrix",
                    };
                    let mut s = format!("{kind} · {} LEDs", z.leds_count);
                    if let Some(m) = &z.matrix {
                        s += &format!(" · {}×{}", m.width, m.height);
                    }
                    if !z.segments.is_empty() {
                        s += &format!(" · {} segments", z.segments.len());
                    }
                    ui.label(RichText::new(s).small().color(TEXT_DIM));
                });
                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                    let start = offsets[zi];
                    let end = (start + z.leds_count as usize).min(d.colors.len());
                    let mut c = app.settings.paint.rgb();
                    if widgets::color_button(ui, ("zone_fill", dev, zi), &mut c, vec2(34.0, 34.0)) {
                        app.settings.paint = Hsv::from_rgb(c);
                        let mut colors = app.devices[dev].colors.clone();
                        colors[start..end].fill(c);
                        app.set_leds(dev, colors);
                    }
                    ui.label(RichText::new("Fill").small().color(TEXT_DIM));
                });
            });
            let start = offsets[zi];
            let end = (start + z.leds_count as usize).min(d.colors.len());
            ui.add_space(4.0);
            let w = ui.available_width();
            widgets::led_bar(ui, &d.colors[start..end], vec2(w, 8.0));

            if z.resizable() {
                ui.add_space(8.0);
                let id = ui.scope_id().with(("resize", dev, zi));
                let mut size: u32 = ui.data(|m| m.get_temp(id)).unwrap_or(z.leds_count);
                ui.horizontal(|ui| {
                    ui.label(ICON_ARROW_RANGE.rich_text().size(18.0).color(TEXT_DIM));
                    ui.label("Size");
                    ui.add(egui::Slider::new(&mut size, z.leds_min..=z.leds_max).suffix(" LEDs"));
                    if size != z.leds_count
                        && widgets::primary_button(ui, ICON_CHECK, "Resize").clicked()
                    {
                        app.send(Command::ResizeZone {
                            dev: dev as u32,
                            zone: zi as u32,
                            size,
                        });
                    }
                });
                ui.data_mut(|m| m.insert_temp(id, size));
            }
        });
        ui.add_space(10.0);
    }
}

// ---------------------------------------------------------------------------
// Info tab
// ---------------------------------------------------------------------------

fn info(app: &mut App, ui: &mut Ui, dev: usize) {
    let d = &app.devices[dev];
    let rows = [
        ("Name", d.name.clone()),
        ("Vendor", d.vendor.clone()),
        ("Type", d.kind.label().to_string()),
        ("Description", d.description.clone()),
        ("Version", d.version.clone()),
        ("Serial", d.serial.clone()),
        ("Location", d.location.clone()),
        ("Modes", d.modes.len().to_string()),
        ("Zones", d.zones.len().to_string()),
        ("LEDs", d.colors.len().to_string()),
    ];
    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::Grid::new("info")
            .num_columns(3)
            .spacing(vec2(24.0, 12.0))
            .show(ui, |ui| {
                for (k, v) in rows {
                    ui.label(RichText::new(k).color(TEXT_DIM));
                    let shown = if v.is_empty() {
                        "—".to_string()
                    } else {
                        v.clone()
                    };
                    ui.label(RichText::new(shown).color(TEXT));
                    if !v.is_empty() {
                        let b = egui::Button::new(
                            ICON_CONTENT_COPY.rich_text().size(15.0).color(TEXT_FAINT),
                        )
                        .frame(false);
                        if ui.add(b).on_hover_text("Copy").clicked() {
                            ui.ctx().copy_text(v);
                        }
                    }
                    ui.end_row();
                }
            });
    });
}
