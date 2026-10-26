# PimpBox RGB

<p align="center"><img src="www/assets/logo.svg" alt="PimpBox RGB — Pimp My Box: RGB Edition" width="720"></p>

[Website](https://cyrill62.github.io/openrgb-egui/) · [GitHub](https://github.com/cyrill62/openrgb-egui)

A desktop and terminal control panel for [OpenRGB](https://openrgb.org), written
in Rust with [egui](https://github.com/emilk/egui) and
[ratatui](https://ratatui.rs).

This app is a client for a running OpenRGB instance. It connects through the
OpenRGB SDK server and controls devices detected by OpenRGB; it does not replace
the OpenRGB service or detect hardware itself.

## Features

- **Overview:** inspect devices with live LED previews, apply one color to all,
  or turn all LEDs off.
- **Modes:** change hardware modes and supported speed, brightness, direction,
  and mode colors; save settings to compatible devices.
- **LEDs:** view a device's LED layout (including keyboard key maps), select
  LEDs, and paint colors live.
- **Zones:** fill zones and resize addressable ARGB headers where supported.
- **Profiles:** load, save, and delete profiles stored by the OpenRGB server.
- Responds to device hot-plug notifications and remembers the server address.

## Usage

1. Start the OpenRGB SDK server with `openrgb --server`, or use **Start Server**
   in OpenRGB's **SDK Server** tab.
2. Run the application:

   ```sh
   cargo run --release                       # connects to 127.0.0.1:6742
   cargo run --release -- 192.168.1.20:6742  # another host[:port]
   cargo run --release --features tui --bin pimpbox-rgb-tui # terminal interface
   ```

The client supports SDK protocol versions 0–4. Newer servers negotiate down to
version 4.

## Building releases

On Linux or macOS:

```sh
scripts/build.sh                 # both architectures for the host OS
scripts/build.sh linux-x64       # or linux-arm64, macos, windows-x64, all, etc.
```

On Windows (PowerShell):

```powershell
.\scripts\build.ps1              # Windows x64 and ARM64
```

See `scripts/build.sh --help` or `Get-Help .\scripts\build.ps1` for targets and
cross-compilation requirements. Archives are written to `dist/` and include
both `pimpbox-rgb` (desktop) and `pimpbox-rgb-tui` (terminal). Pushing a `v*`
tag starts the GitHub release workflow, which builds Linux (glibc and musl),
macOS, and Windows packages and attaches them to a draft GitHub release.

## Development

```sh
cargo check
cargo test
npm --prefix www run dev                       # Vike website
cargo run --example mock_server -- 26742        # fake server with virtual devices
cargo run -- 127.0.0.1:26742
cargo run --example probe -- 127.0.0.1 6742     # read-only device listing

# Capture a PNG of each page (builds the debug app and starts a mock server):
scripts/screenshots.sh
# PowerShell:
.\scripts\screenshots.ps1
```

The mock server listens on `127.0.0.1:6742` by default. Pass a port after `--`
to choose another port.

## Project layout

| Path | Purpose |
| --- | --- |
| `src/protocol.rs` | SDK packet framing and controller serialization |
| `src/client.rs` | Blocking TCP client |
| `src/worker.rs` | Background thread that owns the connection |
| `src/ui/` | egui application, theme, widgets, and pages |
| `src/tui/` | Optional ratatui terminal application (`tui` feature) |
| `examples/` | Mock SDK server and read-only probe |
| `scripts/` | Release build and packaging scripts |

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidance and
[CHANGELOG.md](CHANGELOG.md) for changes. This project is licensed under
[GPL-2.0-or-later](LICENSE).
