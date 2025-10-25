# Dioxus Build Notes

## Summary

The project has been successfully refactored to use Dioxus framework. The desktop build works correctly.

## Desktop Build

**Status: ✅ Working**

To build the desktop app:
```bash
cargo build --release
```

The resulting binary will be at `target/release/pubky-wiki`.

## Web Build

**Status: ⚠️ Partial - Needs Additional Work**

The web build faces challenges due to native dependencies in the `pubky` crate:
- The `pubky` crate has native dependencies (mio, tokio, etc.) that don't compile to WebAssembly
- To make the web build work, one of these approaches would be needed:
  1. Create a backend API that handles pubky operations, with the frontend making HTTP requests
  2. Use conditional compilation to separate web-only code from desktop-only code
  3. Wait for pubky to add wasm support or use wasm-compatible alternatives

To attempt a web build (will currently fail):
```bash
cargo build --target wasm32-unknown-unknown --release
```

## Architecture

- **Framework**: Dioxus 0.5.6
- **Desktop**: Uses dioxus-desktop (native rendering with webview)
- **Web**: Uses dioxus-web (compiles to WebAssembly)
- **Markdown**: markdown crate for rendering
- **Styling**: Inline CSS

## Key Changes from egui

1. Replaced egui/eframe with Dioxus
2. Changed from immediate mode to reactive UI with signals
3. Converted all UI components to Dioxus functional components
4. Used `use_signal` for state management instead of mutable struct fields
5. Async operations now use `spawn` with Dioxus's async runtime

## Dependencies

Desktop dependencies installed:
- libgtk-3-dev
- webkit2gtk-4.1-dev
- libayatana-appindicator3-dev
- librsvg2-dev
- libssl-dev
- pkg-config

## Notes

- The desktop app builds successfully and should work for the intended use case
- CSS is inlined in the binary using `include_str!` macro
- The app requires Rust nightly due to some dependency requirements
