mod device;
mod pages;
#[cfg(all(feature = "screenshot", debug_assertions))]
mod shots;
pub mod theme;
mod widgets;

use std::collections::BTreeSet;

use eframe::egui::{
    self, Align, Align2, Color32, CornerRadius, Layout, Margin, Mesh, RichText, Sense, Stroke,
    StrokeKind, Ui, pos2, vec2,
};
use egui_material_icons::{MaterialIcon, icons::*};
use serde::{Deserialize, Serialize};

use pimpbox_rgb::protocol::{Controller, Mode, Rgb};
use pimpbox_rgb::worker::{self, Command, Event};

use theme::{ACCENT, BORDER, SIDEBAR, TEXT, TEXT_DIM, TEXT_FAINT};
use widgets::Hsv;

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub host: String,
    pub port: u16,
    pub auto_connect: bool,
    pub paint: Hsv,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 6742,
            auto_connect: true,
            paint: Hsv {
                h: 265.0,
                s: 0.65,
                v: 1.0,
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Conn {
    Disconnected(Option<String>),
    Connecting,
    Connected { proto: u32, server_proto: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Overview,
    Device(usize),
    Profiles,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeviceTab {
    Lighting,
    Leds,
    Zones,
    Info,
}

struct Toast {
    text: String,
    error: bool,
    until: f64,
}

pub struct App {
    worker: worker::Handle,
    pub settings: Settings,
    pub conn: Conn,
    pub devices: Vec<Controller>,
    pub profiles: Vec<String>,
    pub page: Page,
    pub tab: DeviceTab,
    /// Selected LED indices (controller-wide) on the current device.
    pub selection: BTreeSet<usize>,
    pub new_profile: String,
    pub sync_color: Hsv,
    toasts: Vec<Toast>,
    #[cfg(all(feature = "screenshot", debug_assertions))]
    tour: Option<shots::Tour>,
}

impl App {
    /// `server` overrides the saved address (`host` or `host:port`) and connects.
    pub fn new(cc: &eframe::CreationContext<'_>, server: Option<String>) -> Self {
        theme::install(&cc.egui_ctx);
        let mut settings: Settings = cc
            .storage
            .and_then(|s| eframe::get_value(s, eframe::APP_KEY))
            .unwrap_or_default();
        if let Some(server) = &server {
            match server.rsplit_once(':') {
                Some((h, p)) if p.parse::<u16>().is_ok() => {
                    settings.host = h.to_owned();
                    settings.port = p.parse().unwrap();
                }
                _ => settings.host = server.clone(),
            }
        }
        let ctx = cc.egui_ctx.clone();
        let worker = worker::spawn(move || ctx.request_repaint());
        let mut app = Self {
            worker,
            conn: Conn::Disconnected(None),
            devices: Vec::new(),
            profiles: Vec::new(),
            page: Page::Overview,
            tab: DeviceTab::Lighting,
            selection: BTreeSet::new(),
            new_profile: String::new(),
            sync_color: settings.paint,
            toasts: Vec::new(),
            #[cfg(all(feature = "screenshot", debug_assertions))]
            tour: shots::Tour::from_env(),
            settings,
        };
        if app.settings.auto_connect || server.is_some() {
            app.connect();
        }
        app
    }

    pub fn send(&self, c: Command) {
        self.worker.send(c);
    }

    pub fn connect(&mut self) {
        self.conn = Conn::Connecting;
        self.send(Command::Connect {
            host: self.settings.host.trim().to_owned(),
            port: self.settings.port,
        });
    }

    pub fn toast(&mut self, ctx: &egui::Context, text: impl Into<String>, error: bool) {
        self.toasts.push(Toast {
            text: text.into(),
            error,
            until: ctx.input(|i| i.time) + 3.5,
        });
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.conn, Conn::Connected { .. })
    }

    pub fn proto(&self) -> u32 {
        match self.conn {
            Conn::Connected { proto, .. } => proto,
            _ => 0,
        }
    }

    // -- Device operations shared by pages ---------------------------------

    pub fn set_mode(&mut self, dev: usize, idx: usize, save: bool) {
        let Some(d) = self.devices.get_mut(dev) else {
            return;
        };
        let Some(mode) = d.modes.get(idx).cloned() else {
            return;
        };
        d.active_mode = idx as i32;
        self.send(Command::UpdateMode {
            dev: dev as u32,
            idx,
            mode,
            save,
        });
    }

    pub fn update_active_mode(&mut self, dev: usize, f: impl FnOnce(&mut Mode)) {
        let Some(d) = self.devices.get_mut(dev) else {
            return;
        };
        let Some(idx) = usize::try_from(d.active_mode).ok() else {
            return;
        };
        if let Some(m) = d.modes.get_mut(idx) {
            f(m);
            self.set_mode(dev, idx, false);
        }
    }

    /// Puts the device into its per-LED mode if it isn't already.
    fn ensure_direct(&mut self, dev: usize) {
        let Some(d) = self.devices.get_mut(dev) else {
            return;
        };
        if d.active_mode().is_some_and(Mode::is_direct_like) {
            return;
        }
        if let Some(i) = d.direct_mode_index() {
            d.active_mode = i as i32;
        }
        self.send(Command::SetCustomMode { dev: dev as u32 });
    }

    pub fn set_leds(&mut self, dev: usize, colors: Vec<Rgb>) {
        self.ensure_direct(dev);
        if let Some(d) = self.devices.get_mut(dev) {
            d.colors = colors.clone();
        }
        self.send(Command::UpdateLeds {
            dev: dev as u32,
            colors,
        });
    }

    pub fn fill_device(&mut self, dev: usize, c: Rgb) {
        if let Some(n) = self.devices.get(dev).map(|d| d.colors.len()) {
            self.set_leds(dev, vec![c; n]);
        }
    }

    // -- Event pump ----------------------------------------------------------

    fn pump(&mut self, ctx: &egui::Context) {
        while let Ok(ev) = self.worker.rx.try_recv() {
            match ev {
                Event::Connected {
                    proto,
                    server_proto,
                } => {
                    self.conn = Conn::Connected {
                        proto,
                        server_proto,
                    };
                    let msg = format!("Connected to {}:{}", self.settings.host, self.settings.port);
                    self.toast(ctx, msg, false);
                }
                Event::Disconnected(err) => {
                    let was = self.is_connected();
                    self.conn = Conn::Disconnected(err.clone());
                    self.devices.clear();
                    self.profiles.clear();
                    if was {
                        let msg = err.unwrap_or_else(|| "Disconnected".into());
                        self.toast(ctx, msg, true);
                    }
                }
                Event::Devices(d) => {
                    if self.devices.len() != d.len() {
                        self.selection.clear();
                    }
                    self.devices = d;
                    if let Page::Device(i) = self.page
                        && i >= self.devices.len()
                    {
                        self.page = Page::Overview;
                    }
                }
                Event::Device(i, c) => {
                    if let Some(d) = self.devices.get_mut(i as usize) {
                        if d.colors.len() != c.colors.len() {
                            self.selection.clear();
                        }
                        *d = *c;
                    }
                }
                Event::Profiles(p) => self.profiles = p,
                Event::Error(e) => {
                    if self.conn == Conn::Connecting {
                        self.conn = Conn::Disconnected(Some(e.clone()));
                    }
                    self.toast(ctx, e, true);
                }
            }
        }
    }

    // -- Layout ----------------------------------------------------------------

    fn sidebar(&mut self, ui: &mut Ui) {
        egui::Panel::left("sidebar")
            .resizable(false)
            .exact_size(272.0)
            .frame(
                egui::Frame::new()
                    .fill(SIDEBAR)
                    .inner_margin(Margin::symmetric(14, 18))
                    .stroke(Stroke::new(1.0, BORDER)),
            )
            .show(ui, |ui| {
                logo(ui);
                ui.add_space(20.0);

                if nav_item(ui, ICON_GRID_VIEW, "Overview", self.page == Page::Overview).clicked() {
                    self.page = Page::Overview;
                }
                if nav_item(ui, ICON_FOLDER, "Profiles", self.page == Page::Profiles).clicked() {
                    self.page = Page::Profiles;
                }

                ui.add_space(16.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);
                    widgets::section_label(ui, "Devices");
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.label(
                            RichText::new(self.devices.len().to_string())
                                .small()
                                .color(TEXT_FAINT),
                        );
                    });
                });
                ui.add_space(4.0);

                let bottom_h = 104.0;
                egui::ScrollArea::vertical()
                    .max_height((ui.available_height() - bottom_h).max(60.0))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        let mut go = None;
                        for (i, d) in self.devices.iter().enumerate() {
                            if device_item(ui, d, self.page == Page::Device(i)).clicked() {
                                go = Some(i);
                            }
                        }
                        if self.devices.is_empty() {
                            ui.add_space(8.0);
                            ui.label(RichText::new("   No devices").color(TEXT_FAINT));
                        }
                        if let Some(i) = go {
                            if self.page != Page::Device(i) {
                                self.selection.clear();
                            }
                            self.page = Page::Device(i);
                        }
                    });

                ui.with_layout(Layout::bottom_up(Align::Min), |ui| {
                    self.connection_pill(ui);
                    ui.add_space(6.0);
                    if nav_item(ui, ICON_SETTINGS, "Settings", self.page == Page::Settings)
                        .clicked()
                    {
                        self.page = Page::Settings;
                    }
                });
            });
    }

    fn connection_pill(&mut self, ui: &mut Ui) {
        let (color, title, sub) = match &self.conn {
            Conn::Connected { proto, .. } => (
                theme::SUCCESS,
                "Connected".to_string(),
                format!(
                    "{}:{} · SDK v{proto}",
                    self.settings.host, self.settings.port
                ),
            ),
            Conn::Connecting => (
                theme::WARN,
                "Connecting…".into(),
                self.settings.host.clone(),
            ),
            Conn::Disconnected(_) => (theme::DANGER, "Offline".into(), "Not connected".into()),
        };
        egui::Frame::new()
            .fill(theme::INSET)
            .corner_radius(theme::RADIUS_SM + 2)
            .inner_margin(Margin::symmetric(12, 10))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                    ui.painter()
                        .circle_filled(r.center(), 7.0, color.gamma_multiply(0.25));
                    ui.painter().circle_filled(r.center(), 4.0, color);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.label(RichText::new(title).strong().color(TEXT));
                        ui.label(RichText::new(sub).small().color(TEXT_DIM));
                    });
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        if self.is_connected() {
                            let b = egui::Button::new(
                                ICON_REFRESH.rich_text().size(18.0).color(TEXT_DIM),
                            )
                            .frame(false);
                            if ui.add(b).on_hover_text("Rescan devices").clicked() {
                                self.send(Command::Refresh);
                            }
                        }
                    });
                });
            });
    }

    fn toasts(&mut self, ctx: &egui::Context) {
        let now = ctx.input(|i| i.time);
        self.toasts.retain(|t| t.until > now);
        if self.toasts.is_empty() {
            return;
        }
        ctx.request_repaint_after(std::time::Duration::from_millis(250));
        egui::Area::new(egui::Id::unique("toasts"))
            .anchor(Align2::RIGHT_BOTTOM, vec2(-20.0, -20.0))
            .order(egui::Order::Foreground)
            .interactable(false)
            .show(ctx, |ui| {
                for t in self.toasts.iter().rev().take(4) {
                    let (icon, c) = if t.error {
                        (ICON_INFO, theme::DANGER)
                    } else {
                        (ICON_CHECK_CIRCLE, theme::SUCCESS)
                    };
                    egui::Frame::new()
                        .fill(theme::CARD_HI)
                        .stroke(Stroke::new(1.0, c.gamma_multiply(0.5)))
                        .corner_radius(theme::RADIUS_SM + 2)
                        .inner_margin(Margin::symmetric(14, 10))
                        .shadow(ui.visuals().popup_shadow)
                        .show(ui, |ui| {
                            ui.set_max_width(380.0);
                            ui.horizontal(|ui| {
                                ui.label(icon.rich_text().size(18.0).color(c));
                                ui.label(RichText::new(&t.text).color(TEXT));
                            });
                        });
                    ui.add_space(6.0);
                }
            });
    }
}

impl eframe::App for App {
    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.pump(ctx);
        #[cfg(all(feature = "screenshot", debug_assertions))]
        shots::tick(self, ctx);
    }

    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();
        if !self.is_connected() {
            egui::CentralPanel::default()
                .frame(egui::Frame::new().fill(theme::BG))
                .show(ui, |ui| pages::connect_screen(self, ui));
            self.toasts(&ctx);
            return;
        }

        self.sidebar(ui);
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(theme::BG)
                    .inner_margin(Margin::symmetric(28, 22)),
            )
            .show(ui, |ui| match self.page {
                Page::Overview => pages::overview(self, ui),
                Page::Device(i) => device::page(self, ui, i),
                Page::Profiles => pages::profiles(self, ui),
                Page::Settings => pages::settings(self, ui),
            });
        self.toasts(&ctx);
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, &self.settings);
    }

    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::BG.to_normalized_gamma_f32()
    }
}

// ---------------------------------------------------------------------------
// Sidebar pieces
// ---------------------------------------------------------------------------

/// Paints a rainbow ring next to the product name.
pub fn logo_ring(ui: &mut Ui, size: f32, t: f32) {
    let (rect, _) = ui.allocate_exact_size(vec2(size, size), Sense::hover());
    let c = rect.center();
    let (r_out, r_in) = (size / 2.0, size / 2.0 - size * 0.17);
    const N: usize = 48;
    let mut mesh = Mesh::default();
    for i in 0..=N {
        let a = i as f32 / N as f32 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
        let col = theme::to_c32(Rgb::from_hsv(
            i as f32 / N as f32 * 360.0 + t * 60.0,
            0.85,
            1.0,
        ));
        let d = vec2(a.cos(), a.sin());
        mesh.colored_vertex(c + d * r_out, col);
        mesh.colored_vertex(c + d * r_in, col);
    }
    for i in 0..N as u32 {
        let b = i * 2;
        mesh.add_triangle(b, b + 1, b + 2);
        mesh.add_triangle(b + 1, b + 3, b + 2);
    }
    ui.painter().add(mesh);
    ui.painter()
        .circle_filled(c, r_in * 0.42, Color32::from_white_alpha(230));
}

fn logo(ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.add_space(4.0);
        logo_ring(ui, 34.0, 0.0);
        ui.add_space(4.0);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(RichText::new("OpenRGB").size(18.0).strong().color(TEXT));
            ui.label(RichText::new("Lighting control").small().color(TEXT_DIM));
        });
    });
}

fn nav_item(ui: &mut Ui, icon: MaterialIcon, label: &str, selected: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let r = CornerRadius::same(theme::RADIUS_SM + 1);
        if selected {
            p.rect_filled(rect, r, ACCENT.gamma_multiply(0.16));
            let bar = egui::Rect::from_min_size(rect.left_top() + vec2(0.0, 9.0), vec2(3.0, 20.0));
            p.rect_filled(bar, CornerRadius::same(2), ACCENT);
        } else if resp.hovered() {
            p.rect_filled(rect, r, Color32::from_white_alpha(8));
        }
        let fg = if selected { TEXT } else { TEXT_DIM };
        p.text(
            rect.left_center() + vec2(14.0, 0.0),
            Align2::LEFT_CENTER,
            icon.codepoint,
            egui::FontId::new(19.0, icon.font_family()),
            if selected { ACCENT } else { fg },
        );
        p.text(
            rect.left_center() + vec2(44.0, 0.0),
            Align2::LEFT_CENTER,
            label,
            egui::FontId::proportional(14.0),
            fg,
        );
    }
    resp
}

fn device_item(ui: &mut Ui, d: &Controller, selected: bool) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 54.0), Sense::click());
    if ui.is_rect_visible(rect) {
        let p = ui.painter();
        let r = CornerRadius::same(theme::RADIUS_SM + 1);
        if selected {
            p.rect_filled(rect, r, ACCENT.gamma_multiply(0.16));
            p.rect_stroke(
                rect,
                r,
                Stroke::new(1.0, ACCENT.gamma_multiply(0.4)),
                StrokeKind::Inside,
            );
        } else if resp.hovered() {
            p.rect_filled(rect, r, Color32::from_white_alpha(8));
        }
        let tint = theme::to_c32(theme::accent_of(&d.colors));
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(rect.shrink2(vec2(8.0, 7.0)))
                .layout(Layout::left_to_right(Align::Center)),
        );
        widgets::icon_badge(&mut child, widgets::device_icon(d.kind), tint, 34.0);
        child.add_space(2.0);
        child.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 1.0;
            ui.add(
                egui::Label::new(RichText::new(&d.name).color(if selected {
                    TEXT
                } else {
                    theme::TEXT
                }))
                .truncate(),
            );
            let mode = d.active_mode().map(|m| m.name.as_str()).unwrap_or("—");
            ui.add(
                egui::Label::new(
                    RichText::new(format!("{} · {mode}", d.kind.label()))
                        .small()
                        .color(TEXT_DIM),
                )
                .truncate(),
            );
        });
        let bar = egui::Rect::from_min_max(
            pos2(rect.left() + 50.0, rect.bottom() - 6.0),
            pos2(rect.right() - 10.0, rect.bottom() - 3.0),
        );
        widgets::paint_led_bar(ui.painter(), bar, &d.colors);
    }
    resp.on_hover_text(&d.name)
}
