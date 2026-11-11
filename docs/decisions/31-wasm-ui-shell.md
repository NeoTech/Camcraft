## #31 - App and UI on wasm: rung 2, camcraft-ui links for wasm32 (2026-11-11)

- **Chosen:** make `camcraft-ui` (library and binary) build and link for
  `wasm32-unknown-unknown` by cutting the native-only seams out of the
  shared code, without yet adding a browser runner. Four seams:
  1. **The window.** `egui_app/shell.rs` (1676 lines) is split: the winit
     event loop, window, surface, GPU bring-up and native refusal dialog
     move to `egui_app/shell_window.rs`, the per-frame draw and winit
     `ApplicationHandler` to `egui_app/shell_window/frame.rs`. Both, plus
     `gpu_init`, `frame_watch` and `os_quit`, are
     `#[cfg(not(target_arch = "wasm32"))]`. `shell.rs` keeps what every
     shell shares (`shell_layout`, the dock tabs, the viewport panel, the
     simulation readouts, the headless proof). On wasm32,
     `egui_app::run` refuses the windowed entry by name (exit 2) instead of
     pretending. The moved code is byte-for-byte the old code plus imports.
  2. **File dialogs.** All 13 `rfd::FileDialog` sites go through one seam,
     `crate::file_dialog::FileDialog` (same builder API). Native forwards
     to rfd unchanged; wasm32 answers `None` (the user-visible "cancelled")
     with a stderr line naming #31, until the browser shell brings an async
     picker. `egui-winit` and `rfd` become native-only dependencies.
  3. **The clock.** `std::time::Instant::now()` panics on
     wasm32-unknown-unknown. `camcraft_app::clock::Instant` is std's type
     on native and `web_time::Instant` (browser `performance.now()`) on
     wasm32; app's 6 sites and ui's non-window sites read it. `web-time`
     1.1 was already in the lock through winit/egui, so no new crate enters
     the tree; it is a wasm32-only dependency of camcraft-app. geometry's
     one site (projection profiling) now reads the clock only when
     profiling is switched on by an environment variable, which can never
     be set in a browser - no dependency needed there.
  4. **The proof.** `web/build-app-wasm.sh` also builds and links
     `camcraft-ui.wasm` with the same flecs patch, so the manual `wasm` CI
     job (`workflow_dispatch` only; only its comment changed) covers it.
     On wasm32 the shared shell is unreachable until the web runner calls
     it, so `lib.rs` allows dead code on that target only (543 warnings
     otherwise); native builds keep the lint. The line goes when the
     runner lands.
- **Considered and dropped:**
  - Adding eframe's web runner in the same PR. It needs a JS shim for the
    nine wasi imports, wasm-bindgen CLI 0.2.129 and async GPU bring-up
    before anything can be proved, and none of it can be exercised without
    a browser here. Compiling the shared UI first gives that work a
    buildable base and keeps this PR reviewable.
  - Making `rfd` a dependency on wasm32 too and using its
    `AsyncFileDialog`. The callers are synchronous (`Option<PathBuf>`
    returned in the same frame); an async picker needs a pending-result
    slot in `State`, which belongs with the browser shell.
  - A wasm32 clock seam via our own `performance.now()` binding. web-time
    already does exactly that, is already in the lock, and is MIT/Apache.
- **Still owed (why #31 stays open):** eframe web runner (or egui web
  glue) calling `shell::shell_layout`; async GPU bring-up in place of the
  `pollster::block_on` sites (gpu, trace, composite, headless_gpu); an
  async file picker behind `file_dialog`; run-inline fallbacks for the
  `std::thread::spawn` sites; the JS shim for the wasi imports; wasm-bindgen
  CLI 0.2.129; the in-browser run and its CI job.
- **Cost to change later:** low. The seams are where the browser shell
  plugs in; the native code paths are unchanged.
