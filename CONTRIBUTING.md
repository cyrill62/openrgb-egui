# Contributing

Thanks for helping improve OpenRGB egui. Bug reports, focused fixes, and
well-scoped feature contributions are welcome.

## Before you start

- For bugs, include your operating system, OpenRGB version, SDK protocol version
  if known, steps to reproduce, and relevant logs or screenshots. Remove private
  network details and other sensitive information.
- For larger changes, open an issue or discussion first so the expected behavior
  can be agreed on before substantial implementation work.
- Keep changes focused and avoid unrelated formatting or refactoring.

## Development setup

Install a stable Rust toolchain and the platform dependencies required by
`eframe`/`egui` (including native windowing and graphics development libraries
on Linux). Then build and run:

```sh
cargo check
cargo run
```

For UI development without hardware, run the mock SDK server in one terminal
and connect the app in another:

```sh
cargo run --example mock_server -- 26742
cargo run -- 127.0.0.1:26742
```

The `probe` example lists devices from a running server without changing them:

```sh
cargo run --example probe -- 127.0.0.1 6742
```

## Changes and review

- Preserve the existing module boundaries: SDK protocol and connection logic
  live outside the UI, and the worker owns the blocking connection so UI work
  remains responsive.
- Keep protocol parsing robust against malformed or incomplete server data.
- When adding visible behavior, consider whether the mock server can represent
  it and whether the README or changelog should be updated.
- Run the checks relevant to your change before submitting. At minimum, use
  `cargo check`; run `cargo test` when changing tested behavior. Include the
  commands and outcomes in your pull request description.
- For UI changes, include a screenshot when it helps reviewers understand the
  result. A screenshot tour can be captured with:

  ```sh
  OPENRGB_EGUI_SHOTS=shots cargo run --features screenshot -- 127.0.0.1:6742
  ```

- Do not commit generated `target/` output or local screenshots unless they are
  intentional project assets.

## Pull requests

Describe the user-visible change, relevant implementation details, and how you
checked it. Link related issues and call out any known limitations. Keep each
pull request reviewable in scope and size.

## License

Contributions are distributed under this project's GPL-2.0-or-later license.
By submitting a contribution, you agree that it may be used and redistributed
under those terms.
