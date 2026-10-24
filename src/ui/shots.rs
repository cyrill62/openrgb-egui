//! Automated screenshot tour (feature `screenshot`), used to review the UI.

use std::path::PathBuf;

use eframe::egui;

use openrgb_egui::protocol::Rgb;
use openrgb_egui::worker::Command;

use super::{App, Conn, DeviceTab, Page};

const SETTLE_FRAMES: u32 = 20;

pub struct Tour {
    dir: PathBuf,
    step: usize,
    frames: u32,
}

impl Tour {
    pub fn from_env() -> Option<Self> {
        let dir = PathBuf::from(std::env::var_os("OPENRGB_EGUI_SHOTS")?);
        std::fs::create_dir_all(&dir).ok()?;
        Some(Self {
            dir,
            step: 0,
            frames: 0,
        })
    }
}

type Step = (&'static str, fn(&mut App));

fn steps() -> Vec<Step> {
    vec![
        ("overview", |a| a.page = Page::Overview),
        ("keyboard-lighting", |a| {
            a.page = Page::Device(0);
            a.tab = DeviceTab::Lighting;
        }),
        ("keyboard-leds", |a| {
            a.page = Page::Device(0);
            a.tab = DeviceTab::Leds;
            a.selection = (17..30).collect();
        }),
        ("motherboard-lighting", |a| {
            a.page = Page::Device(2);
            a.tab = DeviceTab::Lighting;
            a.selection.clear();
        }),
        ("ram-breathing", |a| {
            a.page = Page::Device(3);
            a.tab = DeviceTab::Lighting;
        }),
        ("motherboard-zones", |a| {
            a.page = Page::Device(2);
            a.tab = DeviceTab::Zones;
        }),
        ("cooler-leds", |a| {
            a.page = Page::Device(6);
            a.tab = DeviceTab::Leds;
        }),
        ("after-commands", |a| {
            // Exercise the write path against the mock server.
            a.set_mode(3, 1, false);
            a.fill_device(0, Rgb::new(0, 150, 255));
            a.send(Command::ResizeZone {
                dev: 2,
                zone: 2,
                size: 12,
            });
            a.send(Command::Refresh);
            a.page = Page::Overview;
        }),
        ("profiles", |a| a.page = Page::Profiles),
        ("settings", |a| a.page = Page::Settings),
    ]
}

pub fn tick(app: &mut App, ctx: &egui::Context) {
    let Some(t) = app.tour.as_mut() else { return };
    ctx.request_repaint();
    if app.devices.is_empty() {
        // No server: capture the connect screen instead.
        if matches!(app.conn, Conn::Disconnected(Some(_))) {
            t.frames += 1;
            if t.frames == SETTLE_FRAMES {
                save(ctx, t.dir.join("connect.png"));
            } else if t.frames > SETTLE_FRAMES * 2 {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        return;
    }
    let steps = steps();
    let Some((name, setup)) = steps.get(t.step) else {
        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        return;
    };
    t.frames += 1;
    if t.frames == 1 {
        setup(app);
    } else if t.frames == SETTLE_FRAMES {
        save(ctx, t.dir.join(format!("{:02}-{name}.png", t.step)));
    } else if t.frames >= SETTLE_FRAMES * 2 {
        t.step += 1;
        t.frames = 0;
    }
}

fn save(ctx: &egui::Context, path: PathBuf) {
    ctx.request_screenshot(move |img| {
        let [w, h] = img.size;
        let bytes: Vec<u8> = img.pixels.iter().flat_map(|c| c.to_array()).collect();
        if let Err(e) =
            image::save_buffer(&path, &bytes, w as u32, h as u32, image::ColorType::Rgba8)
        {
            eprintln!("screenshot {}: {e}", path.display());
        }
    });
}
