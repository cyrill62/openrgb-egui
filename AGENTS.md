# Agent instructions

Guidance for automated contributors working in this repository:

- Read `README.md`, `Cargo.toml`, and the relevant source before changing
  behavior. Keep documentation consistent with the actual command-line and
  SDK behavior.
- Preserve the separation between `protocol.rs`, `client.rs`, `worker.rs`, and
  `ui/`: protocol and network operations should not block the egui frame loop.
- Keep wire-format parsing and serialization consistent with the OpenRGB SDK
  protocol. Validate lengths and bounds before consuming packet data.
- Prefer small, targeted changes. Avoid unrelated formatting, dependency
  updates, or generated files.
- Do not claim checks were run unless they were. For code changes, use focused
  checks and report their results. Common commands are `cargo check` and
  `cargo test`.
- Update `README.md` for user-facing usage changes and `CHANGELOG.md` when
  recording a release or notable change.
- The package license is GPL-2.0-or-later; retain that licensing when adding or
  adapting code.
