use eframe::egui::{
    self, Color32, CornerRadius, FontFamily, FontId, Margin, Shadow, Stroke, TextStyle, Visuals,
};

use pimpbox_rgb::protocol::Rgb;

pub const BG: Color32 = Color32::from_rgb(0x0D, 0x0F, 0x14);
pub const SIDEBAR: Color32 = Color32::from_rgb(0x12, 0x15, 0x1C);
pub const CARD: Color32 = Color32::from_rgb(0x18, 0x1C, 0x25);
pub const CARD_HI: Color32 = Color32::from_rgb(0x20, 0x25, 0x31);
pub const INSET: Color32 = Color32::from_rgb(0x0F, 0x12, 0x18);
pub const BORDER: Color32 = Color32::from_rgb(0x26, 0x2B, 0x37);
pub const TEXT: Color32 = Color32::from_rgb(0xE7, 0xE9, 0xEF);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0x8B, 0x92, 0xA3);
pub const TEXT_FAINT: Color32 = Color32::from_rgb(0x5C, 0x63, 0x73);
pub const ACCENT: Color32 = Color32::from_rgb(0x8B, 0x5C, 0xF6);
pub const ACCENT_2: Color32 = Color32::from_rgb(0x22, 0xD3, 0xEE);
pub const SUCCESS: Color32 = Color32::from_rgb(0x34, 0xD3, 0x99);
pub const WARN: Color32 = Color32::from_rgb(0xFB, 0xBF, 0x24);
pub const DANGER: Color32 = Color32::from_rgb(0xF8, 0x71, 0x71);

pub const RADIUS: u8 = 12;
pub const RADIUS_SM: u8 = 8;

pub fn title() -> TextStyle {
    TextStyle::Name("title".into())
}

pub fn subheading() -> TextStyle {
    TextStyle::Name("subheading".into())
}

pub fn install(ctx: &egui::Context) {
    egui_material_icons::initialize(ctx);
    ctx.all_styles_mut(|style| {
        use FontFamily::Proportional;
        style.text_styles = [
            (title(), FontId::new(26.0, Proportional)),
            (TextStyle::Heading, FontId::new(19.0, Proportional)),
            (subheading(), FontId::new(15.5, Proportional)),
            (TextStyle::Body, FontId::new(14.0, Proportional)),
            (TextStyle::Button, FontId::new(14.0, Proportional)),
            (TextStyle::Small, FontId::new(11.5, Proportional)),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
        ]
        .into();

        let s = &mut style.spacing;
        s.item_spacing = egui::vec2(8.0, 8.0);
        s.button_padding = egui::vec2(12.0, 6.0);
        s.interact_size.y = 30.0;
        s.slider_width = 260.0;
        s.slider_rail_height = 6.0;
        s.combo_width = 180.0;
        s.window_margin = Margin::same(14);
        s.menu_margin = Margin::same(8);

        style.visuals = visuals();
    });
}

fn visuals() -> Visuals {
    let mut v = Visuals::dark();
    v.panel_fill = BG;
    v.window_fill = CARD;
    v.window_stroke = Stroke::new(1.0, BORDER);
    v.window_corner_radius = CornerRadius::same(RADIUS);
    v.menu_corner_radius = CornerRadius::same(RADIUS_SM);
    v.window_shadow = Shadow {
        offset: [0, 8],
        blur: 28,
        spread: 0,
        color: Color32::from_black_alpha(140),
    };
    v.popup_shadow = v.window_shadow;
    v.extreme_bg_color = INSET;
    v.faint_bg_color = CARD_HI;
    v.code_bg_color = INSET;
    v.hyperlink_color = ACCENT_2;
    v.warn_fg_color = WARN;
    v.error_fg_color = DANGER;
    v.selection.bg_fill = ACCENT.gamma_multiply(0.55);
    v.selection.stroke = Stroke::new(1.0, TEXT);
    v.slider_trailing_fill = true;

    let r = CornerRadius::same(RADIUS_SM);
    let w = &mut v.widgets;
    w.noninteractive.bg_fill = CARD;
    w.noninteractive.weak_bg_fill = CARD;
    w.noninteractive.bg_stroke = Stroke::new(1.0, BORDER);
    w.noninteractive.fg_stroke = Stroke::new(1.0, TEXT);
    w.noninteractive.corner_radius = r;

    w.inactive.bg_fill = CARD_HI;
    w.inactive.weak_bg_fill = CARD_HI;
    w.inactive.bg_stroke = Stroke::NONE;
    w.inactive.fg_stroke = Stroke::new(1.0, TEXT);
    w.inactive.corner_radius = r;

    w.hovered.bg_fill = Color32::from_rgb(0x2A, 0x30, 0x3E);
    w.hovered.weak_bg_fill = Color32::from_rgb(0x2A, 0x30, 0x3E);
    w.hovered.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0x3A, 0x41, 0x52));
    w.hovered.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    w.hovered.corner_radius = r;
    w.hovered.expansion = 0.0;

    w.active.bg_fill = ACCENT;
    w.active.weak_bg_fill = Color32::from_rgb(0x33, 0x3A, 0x4A);
    w.active.bg_stroke = Stroke::new(1.0, ACCENT);
    w.active.fg_stroke = Stroke::new(1.5, Color32::WHITE);
    w.active.corner_radius = r;
    w.active.expansion = 0.0;

    w.open = w.active;
    w.open.bg_fill = CARD_HI;
    v
}

pub fn card() -> egui::Frame {
    egui::Frame::new()
        .fill(CARD)
        .stroke(Stroke::new(1.0, BORDER))
        .corner_radius(RADIUS)
        .inner_margin(Margin::same(18))
}

pub fn to_c32(c: Rgb) -> Color32 {
    Color32::from_rgb(c.r, c.g, c.b)
}

/// Raises very dark LED colors so "off" LEDs remain visible on the dark UI.
pub fn led_display(c: Rgb) -> Color32 {
    let lum = c.r.max(c.g).max(c.b);
    if lum < 18 {
        Color32::from_rgb(0x22, 0x26, 0x30)
    } else {
        to_c32(c)
    }
}

/// The most vivid LED color, used to tint device icons. (An average of a
/// rainbow would be grey.)
pub fn accent_of(colors: &[Rgb]) -> Rgb {
    let vividness = |c: &Rgb| {
        let max = c.r.max(c.g).max(c.b) as u32;
        let min = c.r.min(c.g).min(c.b) as u32;
        (max - min) * 2 + max
    };
    colors
        .iter()
        .copied()
        .max_by_key(vividness)
        .unwrap_or(Rgb::BLACK)
}
