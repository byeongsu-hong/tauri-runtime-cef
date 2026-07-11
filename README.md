# tauri-runtime-cef

A [CEF (Chromium Embedded Framework)](https://bitbucket.org/chromiumembedded/cef) runtime for [Tauri](https://tauri.app), maintained standalone so it builds against **published** tauri crates (`tauri-runtime`, `tauri-utils` from crates.io) — no tauri fork, no `[patch.crates-io]`.

Extracted from tauri's unreleased `feat/cef` branch (`tauri-apps/tauri@3b2823b91`, crate `crates/tauri-runtime-cef`) and ported to the published `tauri-runtime` trait surface. When upstream ships an official CEF runtime release, this repo retires.

## Usage

Published tauri has no `cef` feature, `tauri::Cef` alias, or `#[tauri::cef_entry_point]`; their equivalents live here:

```rust
type Cef = tauri_runtime_cef::CefRuntime<tauri::EventLoopMessage>;

fn main() {
    // Must run before anything else: CEF subprocesses re-exec this binary
    // and must register the same custom schemes as the browser process.
    tauri_runtime_cef::configure(tauri_runtime_cef::CefConfig {
        identifier: "com.example.app".into(),
        ..Default::default()
    });
    if std::env::args().any(|arg| arg.starts_with("--type=")) {
        tauri_runtime_cef::run_cef_helper_process();
        return;
    }

    tauri::Builder::<Cef>::default()
        // ...
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Because published tauri only defaults its generic types (`AppHandle`, `WebviewWindow`, …) to wry, apps alias them once (`type AppHandle = tauri::AppHandle<Cef>;`) and build tauri with `default-features = false`.

## How it differs from the feat/cef original

- **Custom schemes** (`tauri://localhost`, `fetch("ipc://localhost/…")`, `asset://localhost`) are registered with Chromium as standard/secure/CORS/fetch-enabled schemes (`OnRegisterCustomSchemes`, list from `CefConfig::custom_schemes`), because published tauri emits the native scheme forms on Linux/macOS rather than feat/cef's `http(s)://<scheme>.localhost` mapping. Both forms are served.
- **Init data** (Chromium switches, cache path, identifier, deep-link schemes) comes from a process-global `CefConfig` instead of feat/cef's generic `RuntimeInitArgs` channel, which published `tauri-runtime` lacks.
- **`on_new_window` handlers degrade to popup deny**: the published handler type wraps a wry platform webview handle a CEF browser can't construct, so an installed handler denies popups outright (the verdict every known caller returns). No handler keeps CEF's native popup behavior.
- feat/cef's trait additions that published tauri never calls (`go_back`/`go_forward` and friends) are inherent methods on `CefWebviewDispatcher`; the address-changed and per-webview runtime-style channels are dormant until upstream ships them.

## Known ceilings

- Verified on Linux (X11). The macOS/Windows paths compile-ported blind — they need a real pass.
- macOS `.app` bundling needs the CEF framework + helper-app layout that feat/cef's tauri-cli produces; the published CLI doesn't do this. Bundle scripting lives with the consuming app for now.
- Deep-link relaunch URLs are dropped on Linux/Windows (published `tauri-runtime` has no `RunEvent::Opened` there).

## License

Apache-2.0 OR MIT, same as upstream Tauri. Original code Copyright 2019-2024 Tauri Programme within The Commons Conservancy; see `LICENSE_APACHE-2.0`, `LICENSE_MIT`, and `LICENSE.spdx`.
