//! Reusable custom widgets.

use eframe::egui::{
    self, Align2, Color32, CornerRadius, FontId, Mesh, Pos2, Rect, Response, RichText, Sense,
    Stroke, StrokeKind, Ui, Vec2, pos2, vec2,
};
use egui_material_icons::{MaterialIcon, icons::*};

use openrgb_egui::protocol::{DeviceType, Rgb};

use super::theme::{self, ACCENT, BORDER, CARD_HI, INSET, TEXT, TEXT_DIM};

// ---------------------------------------------------------------------------
// HSV
// ---------------------------------------------------------------------------

/// HSV kept separately from RGB so the hue survives while saturation or
/// value is zero.
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Hsv {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl Hsv {
    pub fn rgb(self) -> Rgb {
        Rgb::from_hsv(self.h, self.s, self.v)
    }

    pub fn from_rgb(c: Rgb) -> Self {
        let (r, g, b) = (c.r as f32 / 255.0, c.g as f32 / 255.0, c.b as f32 / 255.0);
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let d = max - min;
        let h = if d == 0.0 {
            0.0
        } else if max == r {
            60.0 * ((g - b) / d).rem_euclid(6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
        let s = if max == 0.0 { 0.0 } else { d / max };
        Self { h, s, v: max }
    }

    /// Updates from `c` only if it differs from what we'd produce, keeping hue stable.
    pub fn sync(&mut self, c: Rgb) {
        if self.rgb() != c {
            *self = Self::from_rgb(c);
        }
    }
}

pub const PRESETS: [Rgb; 12] = [
    Rgb::new(255, 0, 0),
    Rgb::new(255, 96, 0),
    Rgb::new(255, 200, 0),
    Rgb::new(120, 255, 0),
    Rgb::new(0, 255, 90),
    Rgb::new(0, 255, 220),
    Rgb::new(0, 150, 255),
    Rgb::new(40, 40, 255),
    Rgb::new(140, 0, 255),
    Rgb::new(255, 0, 200),
    Rgb::new(255, 255, 255),
    Rgb::new(0, 0, 0),
];

// ---------------------------------------------------------------------------
// Color picker
// ---------------------------------------------------------------------------

fn c32(c: Rgb) -> Color32 {
    theme::to_c32(c)
}

/// Full color picker: saturation/value field, hue rail, hex entry and presets.
/// Returns `true` when the color changed.
pub fn color_picker(ui: &mut Ui, hsv: &mut Hsv, width: f32) -> bool {
    let before = *hsv;
    let field_h = (width * 0.62).max(120.0);

    // Saturation/value field.
    let (rect, resp) = ui.allocate_exact_size(vec2(width, field_h), Sense::click_and_drag());
    if let Some(p) = resp.interact_pointer_pos() {
        hsv.s = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0);
        hsv.v = 1.0 - ((p.y - rect.top()) / rect.height()).clamp(0.0, 1.0);
    }
    if ui.is_rect_visible(rect) {
        const N: usize = 24;
        let mut mesh = Mesh::default();
        for yi in 0..=N {
            for xi in 0..=N {
                let (s, v) = (xi as f32 / N as f32, 1.0 - yi as f32 / N as f32);
                let pos = pos2(
                    rect.left() + s * rect.width(),
                    rect.top() + (1.0 - v) * rect.height(),
                );
                mesh.colored_vertex(pos, c32(Rgb::from_hsv(hsv.h, s, v)));
            }
        }
        let row = (N + 1) as u32;
        for yi in 0..N as u32 {
            for xi in 0..N as u32 {
                let i = yi * row + xi;
                mesh.add_triangle(i, i + 1, i + row);
                mesh.add_triangle(i + 1, i + row + 1, i + row);
            }
        }
        let painter = ui.painter_at(rect);
        painter.add(mesh);
        // Round the corners by masking with a thick rounded stroke in the card color.
        painter.rect_stroke(
            rect.expand(3.0),
            CornerRadius::same(theme::RADIUS_SM + 3),
            Stroke::new(6.0, ui.visuals().window_fill),
            StrokeKind::Inside,
        );
        let knob = pos2(
            rect.left() + hsv.s * rect.width(),
            rect.top() + (1.0 - hsv.v) * rect.height(),
        );
        knob_at(ui.painter(), knob, c32(hsv.rgb()));
    }

    // Hue rail.
    ui.add_space(4.0);
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 16.0), Sense::click_and_drag());
    if let Some(p) = resp.interact_pointer_pos() {
        hsv.h = ((p.x - rect.left()) / rect.width()).clamp(0.0, 1.0) * 359.9;
        if hsv.s < 0.05 && hsv.v > 0.0 {
            hsv.s = 1.0;
        }
        if hsv.v < 0.05 {
            hsv.v = 1.0;
        }
    }
    if ui.is_rect_visible(rect) {
        let rail = rect.shrink2(vec2(0.0, 2.0));
        gradient_rail(ui.painter(), rail, |t| Rgb::from_hsv(t * 360.0, 1.0, 1.0));
        let x = rect.left() + hsv.h / 360.0 * rect.width();
        knob_at(
            ui.painter(),
            pos2(x, rect.center().y),
            c32(Rgb::from_hsv(hsv.h, 1.0, 1.0)),
        );
    }

    // Hex + preview.
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        let (sw, _) = ui.allocate_exact_size(vec2(30.0, 30.0), Sense::hover());
        ui.painter()
            .rect_filled(sw, CornerRadius::same(theme::RADIUS_SM), c32(hsv.rgb()));
        ui.painter().rect_stroke(
            sw,
            CornerRadius::same(theme::RADIUS_SM),
            Stroke::new(1.0, BORDER),
            StrokeKind::Inside,
        );
        let id = ui.scope_id().with("hex");
        let mut text = ui
            .data(|d| d.get_temp::<String>(id))
            .unwrap_or_else(|| hsv.rgb().hex());
        let resp = ui.add(
            egui::TextEdit::singleline(&mut text)
                .font(egui::TextStyle::Monospace)
                .desired_width(width - 38.0 - ui.spacing().item_spacing.x)
                .char_limit(7)
                .align(Align2::LEFT_CENTER)
                .margin(vec2(8.0, 7.0)),
        );
        if resp.has_focus() {
            if let Some(c) = Rgb::from_hex(&text) {
                hsv.sync(c);
            }
            ui.data_mut(|d| d.insert_temp(id, text));
        } else {
            ui.data_mut(|d| d.remove::<String>(id));
        }
    });

    // Presets.
    ui.add_space(2.0);
    let per_row = PRESETS.len() / 2;
    let gap = 6.0;
    let size = ((width - gap * (per_row as f32 - 1.0)) / per_row as f32).floor();
    for chunk in PRESETS.chunks(per_row) {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for &c in chunk {
                if swatch(ui, c, size, hsv.rgb() == c).clicked() {
                    *hsv = Hsv::from_rgb(c);
                }
            }
        });
    }

    *hsv != before
}

fn knob_at(p: &egui::Painter, pos: Pos2, fill: Color32) {
    p.circle_filled(pos + vec2(0.0, 1.0), 9.0, Color32::from_black_alpha(90));
    p.circle_filled(pos, 8.0, Color32::WHITE);
    p.circle_filled(pos, 5.5, fill);
}

pub fn gradient_rail(p: &egui::Painter, rect: Rect, f: impl Fn(f32) -> Rgb) {
    const N: usize = 36;
    let stops: Vec<(f32, Color32)> = (0..=N)
        .map(|i| {
            let t = i as f32 / N as f32;
            (t, c32(f(t)))
        })
        .collect();
    gradient_pill(p, rect, &stops);
}

/// Horizontal gradient through `stops` (`t` in `0..=1`) with round end caps.
pub fn gradient_pill(p: &egui::Painter, rect: Rect, stops: &[(f32, Color32)]) {
    let (Some(first), Some(last)) = (stops.first(), stops.last()) else {
        return;
    };
    let r = rect.height() / 2.0;
    let inner = rect.shrink2(vec2(r, 0.0));
    p.circle_filled(pos2(inner.left(), rect.center().y), r, first.1);
    p.circle_filled(pos2(inner.right(), rect.center().y), r, last.1);
    let mut mesh = Mesh::default();
    for &(t, c) in stops {
        let x = inner.left() + t * inner.width();
        mesh.colored_vertex(pos2(x, rect.top()), c);
        mesh.colored_vertex(pos2(x, rect.bottom()), c);
    }
    for i in 0..(stops.len() as u32).saturating_sub(1) {
        let b = i * 2;
        mesh.add_triangle(b, b + 1, b + 2);
        mesh.add_triangle(b + 1, b + 3, b + 2);
    }
    p.add(mesh);
}

pub fn swatch(ui: &mut Ui, c: Rgb, size: f32, selected: bool) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::click());
    if ui.is_rect_visible(rect) {
        let r = CornerRadius::same(6);
        let grow = if resp.hovered() { 1.0 } else { 0.0 };
        ui.painter().rect_filled(rect.expand(grow), r, c32(c));
        let stroke = if selected {
            Stroke::new(2.0, Color32::WHITE)
        } else {
            Stroke::new(1.0, Color32::from_white_alpha(28))
        };
        ui.painter()
            .rect_stroke(rect.expand(grow), r, stroke, StrokeKind::Inside);
    }
    resp.on_hover_text(c.hex())
}

/// A color chip that opens a picker popup. Returns `true` on change.
pub fn color_button(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash + std::fmt::Debug,
    color: &mut Rgb,
    size: Vec2,
) -> bool {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let r = CornerRadius::same(theme::RADIUS_SM);
        ui.painter().rect_filled(rect, r, c32(*color));
        let stroke = if resp.hovered() {
            Stroke::new(2.0, Color32::from_white_alpha(200))
        } else {
            Stroke::new(1.0, Color32::from_white_alpha(30))
        };
        ui.painter()
            .rect_stroke(rect, r, stroke, StrokeKind::Inside);
    }
    let resp = resp.on_hover_text(color.hex());

    let hsv_id = ui.scope_id().with(("color_button", id_salt));
    let mut changed = false;
    egui::Popup::from_toggle_button_response(&resp)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            let mut hsv = ui
                .data(|d| d.get_temp::<Hsv>(hsv_id))
                .unwrap_or_else(|| Hsv::from_rgb(*color));
            hsv.sync(*color);
            if color_picker(ui, &mut hsv, 240.0) {
                *color = hsv.rgb();
                changed = true;
            }
            ui.data_mut(|d| d.insert_temp(hsv_id, hsv));
        });
    changed
}

// ---------------------------------------------------------------------------
// Misc building blocks
// ---------------------------------------------------------------------------

pub fn device_icon(kind: DeviceType) -> MaterialIcon {
    use DeviceType::*;
    match kind {
        Motherboard => ICON_DEVELOPER_BOARD,
        Dram => ICON_MEMORY,
        Gpu => ICON_VIDEOGAME_ASSET,
        Cooler => ICON_MODE_FAN,
        LedStrip => ICON_LINEAR_SCALE,
        Keyboard => ICON_KEYBOARD,
        Mouse => ICON_MOUSE,
        MouseMat => ICON_GRID_VIEW,
        Headset | HeadsetStand => ICON_HEADSET,
        Gamepad => ICON_SPORTS_ESPORTS,
        Light => ICON_LIGHTBULB,
        Speaker => ICON_SPEAKER,
        Virtual => ICON_EXTENSION,
        Storage => ICON_STORAGE,
        Case => ICON_DESKTOP_WINDOWS,
        Microphone => ICON_MIC,
        Accessory | Unknown => ICON_DEVICES_OTHER,
        Keypad => ICON_DIALPAD,
        Laptop => ICON_LAPTOP,
        Monitor => ICON_MONITOR,
    }
}

/// Rounded square with an icon, tinted by `tint` (e.g. the device's average color).
pub fn icon_badge(ui: &mut Ui, icon: MaterialIcon, tint: Color32, size: f32) -> Response {
    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    if ui.is_rect_visible(rect) {
        let r = CornerRadius::same((size * 0.28) as u8);
        ui.painter().rect_filled(rect, r, tint.gamma_multiply(0.18));
        ui.painter().rect_stroke(
            rect,
            r,
            Stroke::new(1.0, tint.gamma_multiply(0.35)),
            StrokeKind::Inside,
        );
        let fg = brighten(tint);
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            icon.codepoint,
            FontId::new(size * 0.55, icon.font_family()),
            fg,
        );
    }
    resp
}

/// Lifts dark tints so icons drawn in them stay legible.
pub fn brighten(c: Color32) -> Color32 {
    let m = c.r().max(c.g()).max(c.b());
    if m < 40 {
        return TEXT_DIM;
    }
    let k = 230.0 / m as f32;
    let f = |v: u8| ((v as f32 * k).min(255.0) * 0.85 + 255.0 * 0.15) as u8;
    Color32::from_rgb(f(c.r()), f(c.g()), f(c.b()))
}

pub fn section_label(ui: &mut Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .small()
            .strong()
            .color(theme::TEXT_FAINT)
            .extra_letter_spacing(1.2),
    );
}

pub fn icon_text(icon: MaterialIcon, text: &str) -> RichText {
    RichText::new(format!("{}  {}", icon.codepoint, text))
}

/// Accent-filled call-to-action button.
pub fn primary_button(ui: &mut Ui, icon: MaterialIcon, text: &str) -> Response {
    ui.add(
        egui::Button::new(icon_text(icon, text).color(Color32::WHITE).strong())
            .fill(ACCENT)
            .corner_radius(theme::RADIUS_SM)
            .min_size(vec2(0.0, 34.0)),
    )
}

pub fn secondary_button(ui: &mut Ui, icon: MaterialIcon, text: &str) -> Response {
    ui.add(
        egui::Button::new(icon_text(icon, text))
            .fill(CARD_HI)
            .corner_radius(theme::RADIUS_SM)
            .min_size(vec2(0.0, 34.0)),
    )
}

pub fn danger_button(ui: &mut Ui, icon: MaterialIcon, text: &str) -> Response {
    ui.add(
        egui::Button::new(icon_text(icon, text).color(theme::DANGER))
            .fill(theme::DANGER.gamma_multiply(0.12))
            .corner_radius(theme::RADIUS_SM)
            .min_size(vec2(0.0, 34.0)),
    )
}

/// Pill-shaped selectable chip.
pub fn chip(ui: &mut Ui, selected: bool, text: &str, icon: Option<MaterialIcon>) -> Response {
    let font = egui::TextStyle::Button.resolve(ui.style());
    let label = match icon {
        Some(i) => format!("{}  {}", i.codepoint, text),
        None => text.to_owned(),
    };
    let galley = ui.painter().layout_no_wrap(label, font, TEXT);
    let size = vec2(galley.size().x + 28.0, 34.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    if ui.is_rect_visible(rect) {
        let t = ui.ctx().animate_bool(resp.id, selected);
        let hover = resp.hovered() as u8 as f32;
        let bg = CARD_HI.lerp_to_gamma(ACCENT.gamma_multiply(0.28), t);
        let bg = bg.lerp_to_gamma(Color32::from_rgb(0x2E, 0x34, 0x44), hover * (1.0 - t) * 0.8);
        let r = CornerRadius::same(17);
        ui.painter().rect_filled(rect, r, bg);
        let stroke_c = BORDER.lerp_to_gamma(ACCENT, t);
        ui.painter()
            .rect_stroke(rect, r, Stroke::new(1.0, stroke_c), StrokeKind::Inside);
        let text_c = theme::TEXT_DIM.lerp_to_gamma(Color32::WHITE, t.max(hover * 0.7));
        ui.painter().galley_with_override_text_color(
            rect.center() - galley.size() / 2.0,
            galley,
            text_c,
        );
    }
    resp
}

/// Segmented control; returns `true` when the selection changed.
pub fn segmented<T: PartialEq + Copy>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> bool {
    let mut changed = false;
    egui::Frame::new()
        .fill(INSET)
        .corner_radius(theme::RADIUS_SM + 2)
        .inner_margin(3)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 2.0;
                for &(v, label) in options {
                    let sel = *value == v;
                    let text = if sel {
                        RichText::new(label).color(Color32::WHITE).strong()
                    } else {
                        RichText::new(label).color(TEXT_DIM)
                    };
                    let b = egui::Button::new(text)
                        .fill(if sel { ACCENT } else { Color32::TRANSPARENT })
                        .corner_radius(theme::RADIUS_SM)
                        .min_size(vec2(64.0, 28.0));
                    if ui.add(b).clicked() && !sel {
                        *value = v;
                        changed = true;
                    }
                }
            });
        });
    changed
}

/// Compact horizontal strip showing every LED's color.
pub fn led_bar(ui: &mut Ui, colors: &[Rgb], size: Vec2) -> Response {
    let (rect, resp) = ui.allocate_exact_size(size, Sense::hover());
    if ui.is_rect_visible(rect) {
        paint_led_bar(ui.painter(), rect, colors);
    }
    resp
}

pub fn paint_led_bar(p: &egui::Painter, rect: Rect, colors: &[Rgb]) {
    let stops: Vec<(f32, Color32)> = match colors {
        [] => vec![(0.0, INSET), (1.0, INSET)],
        [c] => vec![(0.0, theme::led_display(*c)), (1.0, theme::led_display(*c))],
        _ => {
            let n = colors.len() - 1;
            colors
                .iter()
                .enumerate()
                .map(|(i, &c)| (i as f32 / n as f32, theme::led_display(c)))
                .collect()
        }
    };
    gradient_pill(p, rect, &stops);
}

/// Labelled slider row with the value shown on the right.
pub fn slider_row(
    ui: &mut Ui,
    icon: MaterialIcon,
    label: &str,
    value: &mut u32,
    range: (u32, u32),
) -> Response {
    let (lo, hi) = range;
    let inverted = lo > hi;
    let (min, max) = if inverted { (hi, lo) } else { (lo, hi) };
    // Present "faster/brighter" to the right regardless of the device's convention.
    let mut shown = if inverted { min + max - *value } else { *value };
    let resp = ui
        .horizontal(|ui| {
            ui.label(icon.rich_text().size(18.0).color(TEXT_DIM));
            ui.add_sized(vec2(90.0, 24.0), egui::Label::new(label));
            let pct = if max > min {
                (shown - min) as f32 / (max - min) as f32 * 100.0
            } else {
                100.0
            };
            let resp = ui.add(egui::Slider::new(&mut shown, min..=max).show_value(false));
            ui.label(
                RichText::new(format!("{pct:.0}%"))
                    .monospace()
                    .color(TEXT_DIM),
            );
            resp
        })
        .inner;
    *value = if inverted { min + max - shown } else { shown };
    resp
}

/// Small helper to paint dim text centered in a big empty area.
pub fn empty_state(ui: &mut Ui, icon: MaterialIcon, title: &str, body: &str) {
    ui.vertical_centered(|ui| {
        ui.add_space(ui.available_height() * 0.25);
        ui.label(icon.rich_text().size(56.0).color(theme::TEXT_FAINT));
        ui.add_space(6.0);
        ui.label(RichText::new(title).heading().color(TEXT));
        ui.label(RichText::new(body).color(TEXT_DIM));
    });
}
