use eframe::egui::{self, Align, Layout, Margin, RichText, Sense, Stroke, StrokeKind, Ui, vec2};
use egui_material_icons::icons::*;

use pimpbox_rgb::protocol::Rgb;
use pimpbox_rgb::worker::Command;

use super::theme::{self, ACCENT, BORDER, TEXT, TEXT_DIM, TEXT_FAINT};
use super::widgets;
use super::{App, Conn, Page};

fn page_title(ui: &mut Ui, title: &str, subtitle: &str) {
    ui.label(RichText::new(title).text_style(theme::title()).color(TEXT));
    ui.label(RichText::new(subtitle).color(TEXT_DIM));
    ui.add_space(18.0);
}

fn server_fields(app: &mut App, ui: &mut Ui) -> bool {
    let mut submit = false;
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("Host").small().color(TEXT_DIM));
            let r = ui.add(
                egui::TextEdit::singleline(&mut app.settings.host)
                    .desired_width(220.0)
                    .margin(vec2(10.0, 8.0)),
            );
            submit |= r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
        });
        ui.vertical(|ui| {
            ui.label(RichText::new("Port").small().color(TEXT_DIM));
            ui.add(
                egui::DragValue::new(&mut app.settings.port)
                    .range(1..=65535)
                    .speed(0.0),
            );
        });
    });
    submit
}

// ---------------------------------------------------------------------------
// Connect
// ---------------------------------------------------------------------------

pub fn connect_screen(app: &mut App, ui: &mut Ui) {
    let connecting = app.conn == Conn::Connecting;
    if connecting {
        ui.ctx().request_repaint();
    }
    ui.vertical_centered(|ui| {
        ui.add_space((ui.available_height() * 0.5 - 250.0).max(24.0));
        ui.allocate_ui(vec2(440.0, 0.0), |ui| {
            theme::card()
                .inner_margin(Margin::same(32))
                .shadow(ui.visuals().window_shadow)
                .show(ui, |ui| {
                    ui.set_width(376.0);
                    ui.vertical_centered(|ui| {
                        ui.add(egui::Image::new(&app.banner).fit_to_exact_size(vec2(352.0, 98.0)));
                        ui.label(RichText::new("OpenRGB Lighting control").color(TEXT_DIM));
                    });
                    ui.add_space(22.0);
                    let submit = server_fields(app, ui);
                    ui.add_space(14.0);

                    if let Conn::Disconnected(Some(err)) = &app.conn {
                        egui::Frame::new()
                            .fill(theme::DANGER.gamma_multiply(0.1))
                            .stroke(Stroke::new(1.0, theme::DANGER.gamma_multiply(0.4)))
                            .corner_radius(theme::RADIUS_SM)
                            .inner_margin(Margin::symmetric(12, 8))
                            .show(ui, |ui| {
                                ui.set_width(ui.available_width());
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(ICON_INFO.rich_text().color(theme::DANGER));
                                    ui.label(RichText::new(err).color(TEXT));
                                });
                            });
                        ui.add_space(12.0);
                    }

                    ui.horizontal(|ui| {
                        let label = if connecting {
                            "Connecting…"
                        } else {
                            "Connect"
                        };
                        let b = ui.add_enabled_ui(!connecting, |ui| {
                            widgets::primary_button(ui, ICON_LINK, label)
                        });
                        if (b.inner.clicked() || submit) && !connecting {
                            app.connect();
                        }
                        ui.checkbox(&mut app.settings.auto_connect, "Connect on startup");
                    });

                    ui.add_space(18.0);
                    ui.separator();
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(
                            "Start OpenRGB with --server, or enable the server in OpenRGB's \
                             \"SDK Server\" tab. The default port is 6742.",
                        )
                        .small()
                        .color(TEXT_FAINT),
                    );
                });
        });
    });
}

// ---------------------------------------------------------------------------
// Overview
// ---------------------------------------------------------------------------

pub fn overview(app: &mut App, ui: &mut Ui) {
    let leds: usize = app.devices.iter().map(|d| d.colors.len()).sum();
    page_title(
        ui,
        "Overview",
        &format!("{} devices · {} LEDs", app.devices.len(), leds),
    );

    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            sync_card(app, ui);
            ui.add_space(18.0);
            widgets::section_label(ui, "Devices");
            ui.add_space(6.0);
            device_grid(app, ui);
        });
}

fn sync_card(app: &mut App, ui: &mut Ui) {
    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        ui.horizontal_top(|ui| {
            ui.vertical(|ui| {
                ui.set_width(260.0);
                widgets::section_label(ui, "Sync all devices");
                ui.add_space(4.0);
                let mut hsv = app.sync_color;
                if widgets::color_picker(ui, &mut hsv, 260.0) {
                    app.sync_color = hsv;
                }
            });
            ui.add_space(20.0);
            ui.vertical(|ui| {
                ui.add_space(18.0);
                ui.label(RichText::new("One color, everywhere").heading().color(TEXT));
                ui.label(
                    RichText::new(
                        "Switches every device that supports per-LED control to its direct \
                         mode and applies the chosen color.",
                    )
                    .color(TEXT_DIM),
                );
                ui.add_space(12.0);
                let preview = app.sync_color.rgb();
                let w = ui.available_width().min(420.0);
                widgets::led_bar(ui, &[preview], vec2(w, 10.0));
                ui.add_space(12.0);
                ui.horizontal(|ui| {
                    if widgets::primary_button(ui, ICON_SYNC, "Apply to all").clicked() {
                        apply_all(app, preview);
                        app.toast(
                            ui.ctx(),
                            format!("Applied {} to all devices", preview.hex()),
                            false,
                        );
                    }
                    if widgets::secondary_button(ui, ICON_POWER_SETTINGS_NEW, "All off").clicked() {
                        apply_all(app, Rgb::BLACK);
                    }
                });
            });
        });
    });
}

fn apply_all(app: &mut App, c: Rgb) {
    for i in 0..app.devices.len() {
        if app.devices[i].direct_mode_index().is_some() {
            app.fill_device(i, c);
        }
    }
}

fn device_grid(app: &mut App, ui: &mut Ui) {
    let gap = 14.0;
    let min_w = 270.0;
    let avail = ui.available_width();
    let cols = ((avail + gap) / (min_w + gap)).floor().max(1.0) as usize;
    let w = (avail - gap * (cols as f32 - 1.0)) / cols as f32;
    let mut go = None;
    for (row, chunk) in app.devices.chunks(cols).enumerate() {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = gap;
            for (j, d) in chunk.iter().enumerate() {
                let i = row * cols + j;
                let h = 128.0;
                let (rect, resp) = ui.allocate_exact_size(vec2(w, h), Sense::click());
                if !ui.is_rect_visible(rect) {
                    continue;
                }
                let hovered = resp.hovered();
                let p = ui.painter();
                let r = theme::RADIUS;
                p.rect_filled(rect, r, if hovered { theme::CARD_HI } else { theme::CARD });
                p.rect_stroke(
                    rect,
                    r,
                    Stroke::new(
                        1.0,
                        if hovered {
                            ACCENT.gamma_multiply(0.6)
                        } else {
                            BORDER
                        },
                    ),
                    StrokeKind::Inside,
                );
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect.shrink(16.0))
                        .layout(Layout::top_down(Align::Min)),
                );
                child.horizontal(|ui| {
                    let tint = theme::to_c32(theme::accent_of(&d.colors));
                    widgets::icon_badge(ui, widgets::device_icon(d.kind), tint, 40.0);
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 1.0;
                        ui.add(
                            egui::Label::new(RichText::new(&d.name).strong().color(TEXT))
                                .truncate(),
                        );
                        let vendor = if d.vendor.is_empty() {
                            d.kind.label()
                        } else {
                            &d.vendor
                        };
                        ui.label(RichText::new(vendor).small().color(TEXT_DIM));
                    });
                });
                child.add_space(10.0);
                child.horizontal(|ui| {
                    let mode = d.active_mode().map(|m| m.name.as_str()).unwrap_or("—");
                    egui::Frame::new()
                        .fill(ACCENT.gamma_multiply(0.16))
                        .corner_radius(10)
                        .inner_margin(Margin::symmetric(8, 2))
                        .show(ui, |ui| {
                            ui.label(RichText::new(mode).small().color(theme::TEXT));
                        });
                    ui.label(
                        RichText::new(format!("{} LEDs", d.colors.len()))
                            .small()
                            .color(TEXT_FAINT),
                    );
                });
                child.add_space(10.0);
                let bw = child.available_width();
                widgets::led_bar(&mut child, &d.colors, vec2(bw, 8.0));
                if resp.clicked() {
                    go = Some(i);
                }
            }
        });
        ui.add_space(gap - ui.spacing().item_spacing.y);
    }
    if let Some(i) = go {
        app.selection.clear();
        app.page = Page::Device(i);
    }
}

// ---------------------------------------------------------------------------
// Profiles
// ---------------------------------------------------------------------------

pub fn profiles(app: &mut App, ui: &mut Ui) {
    page_title(ui, "Profiles", "Profiles are stored by the OpenRGB server.");
    if app.proto() < 2 {
        widgets::empty_state(
            ui,
            ICON_FOLDER_OFF,
            "Profiles unavailable",
            "This server's SDK protocol doesn't support profiles. Update OpenRGB to use them.",
        );
        return;
    }

    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::section_label(ui, "Save current lighting");
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            let r = ui.add(
                egui::TextEdit::singleline(&mut app.new_profile)
                    .hint_text("Profile name")
                    .desired_width(280.0)
                    .margin(vec2(10.0, 8.0)),
            );
            let enter = r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            let name = app.new_profile.trim().to_owned();
            let ok = !name.is_empty();
            let clicked = ui
                .add_enabled_ui(ok, |ui| widgets::primary_button(ui, ICON_SAVE, "Save"))
                .inner
                .clicked();
            if ok && (clicked || enter) {
                app.send(Command::SaveProfile(name));
                app.new_profile.clear();
            }
        });
    });
    ui.add_space(14.0);

    if app.profiles.is_empty() {
        widgets::empty_state(
            ui,
            ICON_FOLDER_OPEN,
            "No profiles yet",
            "Save your current lighting to create one.",
        );
        return;
    }

    let mut action = None;
    egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for name in &app.profiles {
                theme::card()
                    .inner_margin(Margin::symmetric(18, 12))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.label(ICON_FOLDER.rich_text().size(22.0).color(ACCENT));
                            ui.label(RichText::new(name).strong().color(TEXT));
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                if widgets::danger_button(ui, ICON_DELETE, "Delete").clicked() {
                                    action = Some((false, name.clone()));
                                }
                                if widgets::primary_button(ui, ICON_PLAY_ARROW, "Load").clicked() {
                                    action = Some((true, name.clone()));
                                }
                            });
                        });
                    });
                ui.add_space(8.0);
            }
        });
    match action {
        Some((true, name)) => {
            app.send(Command::LoadProfile(name.clone()));
            app.toast(ui.ctx(), format!("Loaded \u{201C}{name}\u{201D}"), false);
        }
        Some((false, name)) => {
            app.send(Command::DeleteProfile(name.clone()));
            app.toast(ui.ctx(), format!("Deleted \u{201C}{name}\u{201D}"), false);
        }
        None => {}
    }
}

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

pub fn settings(app: &mut App, ui: &mut Ui) {
    page_title(ui, "Settings", "Connection and application preferences.");

    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::section_label(ui, "SDK server");
        ui.add_space(4.0);
        let submit = server_fields(app, ui);
        ui.add_space(8.0);
        ui.checkbox(&mut app.settings.auto_connect, "Connect on startup");
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if widgets::primary_button(ui, ICON_SYNC, "Reconnect").clicked() || submit {
                app.connect();
            }
            if widgets::secondary_button(ui, ICON_LINK_OFF, "Disconnect").clicked() {
                app.send(Command::Disconnect);
            }
            if widgets::secondary_button(ui, ICON_REFRESH, "Rescan devices").clicked() {
                app.send(Command::Refresh);
            }
        });
    });
    ui.add_space(14.0);

    theme::card().show(ui, |ui| {
        ui.set_width(ui.available_width());
        widgets::section_label(ui, "Server");
        ui.add_space(4.0);
        if let Conn::Connected {
            proto,
            server_proto,
        } = app.conn
        {
            egui::Grid::new("server_info")
                .spacing(vec2(24.0, 8.0))
                .show(ui, |ui| {
                    ui.label(RichText::new("Address").color(TEXT_DIM));
                    ui.label(format!("{}:{}", app.settings.host, app.settings.port));
                    ui.end_row();
                    ui.label(RichText::new("Server protocol").color(TEXT_DIM));
                    ui.label(format!("v{server_proto}"));
                    ui.end_row();
                    ui.label(RichText::new("Negotiated protocol").color(TEXT_DIM));
                    ui.label(format!("v{proto}"));
                    ui.end_row();
                    ui.label(RichText::new("Devices").color(TEXT_DIM));
                    ui.label(app.devices.len().to_string());
                    ui.end_row();
                });
        }
    });
}
