## #31 - App and UI on wasm: rung 3, a browser runner and awaited GPU start-up (2026-11-11)

- **Chosen: eframe's web runner, not our own canvas glue.** The browser
  shell is `egui_app/shell_web.rs` (wasm32 only): an `eframe::App` whose
  `ui` runs the same `shell::shell_layout` the native window runs, started
  from the page through one exported function, `camcraft_start(canvas_id)`.
  eframe owns what a browser makes different - DOM pointer, key, wheel,
  touch and clipboard events, the text-input agent, the canvas size and
  pixel ratio, the animation-frame loop - and it creates the wgpu instance,
  adapter, device and surface with `await`, which is exactly the start-up
  that could not block.
  - *Why:* eframe 0.36.2 was already in `Cargo.lock` (egui_kittest pulls
    it) on the egui set's hard pin, so no new crate enters the tree; its
    web half does not depend on egui-winit (which does not compile for
    wasm32). Writing the same glue ourselves is about 4000 lines of
    event translation (eframe's `web/` is 4270) that would then have to
    track every egui release.
  - *Dropped:* our own glue over `web-sys` + `egui-wgpu` (the cost above,
    and a second input path to keep in step with the native one); winit's
    own web backend (egui-winit is the blocker, rung 1 found it).
  - *Cost to change later:* low. `shell_web.rs` is one file; the shared
    frame it calls does not know which shell runs it.
- **Chosen: the two shells share the 3D pass.** The scene upload gate, the
  target sizing, the View3D + traced-stock pass and the repaint law moved
  out of the native frame into `egui_app/view_pass.rs`; `shell_window`
  and `shell_web` both call it. The code moved unchanged; the native frame
  got shorter (409 -> about 350 lines).
  - *Why:* a copy in the browser shell would drift, and the viewport is
    where a drift shows as a wrong picture.
  - The browser frame submits the 3D pass inside `App::ui`, before eframe
    paints egui over the canvas, so the viewport texture a frame shows is
    that frame's (the native ordering, kept).
- **Chosen: every blocking GPU wait has an awaited twin; the blocking form
  is native-only.** `TracePass::new`, `Composite::new`, `StockEye::new`
  and `headless_gpu::device` are now thin `pollster::block_on` wrappers,
  `cfg(not(wasm32))`, over `new_async` / `device_async`, which hold the
  unchanged bodies with `.await` at the wait. `TracePass::trace` (the
  CPU-comparable readback the parity tests use) and `device_for_test` stay
  native-only. `pollster` itself is now a native-only dependency, so a
  blocking wait in shared code is a wasm32 compile error, not a hung tab.
  - *Why:* in a browser the device answers an error scope from the event
    loop; blocking the only thread on it never returns. On native the
    futures are ready at once, so the wrappers behave exactly as before.
  - The browser shell does not wait for the stock eye before its first
    frame: it spawns `StockEye::new_async` on the event loop and takes the
    result from a slot on a later frame. Until then, or if the device
    refuses, the deviate sheet stands in (the native fallback, unchanged).
  - `gpu_init.rs` (the native adapter-by-adapter bring-up with its test
    frame) keeps its blocking calls: it is `cfg`'d off wasm32, where
    eframe's awaited start-up takes its place. The browser therefore has
    no `--gpu` adapter choice and no per-adapter fallback chain - the
    browser picks the adapter.
- **Chosen: refuse on the page, never panic.** A missing canvas, a
  refused adapter or surface and a runner that starts without a device all
  end as text in `#camcraft-status` (with `data-state="refused"`) and a
  console error. The page also turns an uncaught error or rejection into
  the same message, so a trap in the module is not a frozen canvas. Quit
  has no window to close in a tab: it is answered in the status line.
- **Chosen: the WASI imports are answered by a small ES module and an
  import map.** flecs' C is linked against the wasi libc (rung 1), so the
  module imports a handful of `wasi_snapshot_preview1` functions.
  wasm-bindgen leaves them as a module import; `web/index.html` maps that
  specifier to `web/wasi_shim.js`, which answers the clocks from the page's
  clocks, sends stdout/stderr to the console, and returns the WASI errno
  for what a tab cannot do (files, blocking waits). `proc_exit` throws, so
  flecs' abort path becomes the page's refusal message. The build script
  fails if the module asks for an import the shim does not export.
  - *Dropped:* a full WASI polyfill package (a new JS dependency for nine
    functions); patching flecs' OS API to avoid libc (a fourth edit to the
    patched copies, and libc is still needed for malloc and printf).
- **Chosen: the start function is an export of a cdylib, not `main`.** The
  page calls `camcraft_start` after it has handed the shim the module's
  memory; starting from `main` would run flecs before the shim can write
  into memory. An export in the library does not survive the link of the
  `camcraft-ui` *binary* (tried: the linker dropped everything and left a
  1.7 MB module), so the build links the library as a cdylib with
  `cargo rustc -p camcraft-ui --lib --crate-type cdylib`. The manifest
  keeps its plain rlib, so native builds link nothing extra. The binary is
  no longer linked for wasm32; its `main` still refuses by name there.
  - *Observed, not changed:* because the wasi libc brings C constructors,
    the linker wraps each export so that it runs them (the glue shows
    `*_command_export` names). The libc's are idempotent and the shell ran
    with them; if a C dependency with real constructors ever joins, export
    `_initialize` instead.
- **Chosen: wasm-bindgen's CLI is pinned to the crate in the lock.**
  `web/build-app-wasm.sh` reads the version from `Cargo.lock`, uses a
  matching `wasm-bindgen` on PATH, and otherwise installs that version
  under `target/wasm-tools/bindgen` (never into `~/.cargo`). A mismatched
  CLI refuses the module, so the pin is the lock, not a number in the
  script.
- **Chosen: `trace.rs` gave up its readback.** The native-only
  `TracePass::trace` and its `decode` moved to `trace/readback.rs`
  (`cfg(not(wasm32))`), unchanged. `trace.rs` was 411 lines before this
  rung and the async twin would have grown it; it is 343 now.
- **What ran (2026-11-11, this machine):** Chrome headless (`--headless=new`,
  SwiftShader) loaded the page from a loopback static server. The module
  instantiated, flecs built the world, the shell drew its first frame
  (ribbon, browser tree, viewport with axes and view cube, properties,
  status line) and a click on the ribbon's Stock door opened the Stock
  page. Chrome offered no WebGPU adapter there, so wgpu ran on WebGL2:
  the awaited stock eye was refused by name before its await ("binds no
  storage buffers") and the sheet stood in. The WebGPU path, and therefore
  an error scope actually awaited in a browser, was not exercised.
- **Threads.** No new fallback was needed for the spawn sites: all of them
  already use `thread::Builder::spawn` and handle its error - the model
  import runs inline, the playback worker and the CPU eye refuse by name -
  and wasm32 answers every spawn with that error. Generation goes through
  rayon, which runs on the calling thread where it cannot start a pool.
  What that means for a user in a browser (playback refuses instead of
  playing) is recorded as owed, not hidden.
- **Still owed (why #31 stays open):** an async file picker behind
  `file_dialog` (nothing can be opened or saved in the browser yet); an
  inline playback path where the worker cannot spawn; the in-browser CI
  job; the Pages deployment (#688); splitting the viewport panel out of
  `shell.rs` (still 923 lines; only an import and a `cfg` changed here);
  a WebGPU run; and the wasm32 `allow(dead_code)` in `lib.rs`, which now
  covers 17 native-only helpers that need gating one by one.

### Review round (2026-11-11, PR #847)

- **Chosen: one diagnostics seam for the UI crate, `crate::diag`.**
  `diag_warn!` / `diag_info!` take the arguments `eprintln!` / `println!`
  took. Natively they print to stderr / stdout as before; on wasm32, where
  std drops both streams, they call `console.warn` / `console.log`. All 35
  direct prints in the UI code that is compiled for the browser moved to
  it (the stock eye falling back to the CPU march, the file dialog
  answering "cancelled", import and ribbon notes, the wasm32 entry
  refusals). A law test (`diag::tests::shared_ui_code_prints_only_through_the_diag_seam`)
  walks the crate sources and fails on a direct print outside the seam,
  the native-only files and the tests.
  - *Why not the `tracing` facade:* camcraft-app has it, but its subscriber
    writes to stderr and is installed from the native `main`; the UI crate
    does not depend on tracing, and a console writer for it is a separate
    piece of work. No new dependency was added: the console binding is
    eframe's re-exported web-sys, already a wasm32 dependency.
  - *Still owed:* camcraft-app's own diagnostics (its `tracing` events and
    its direct prints) are still dropped in a browser. They need a console
    writer installed by the browser shell.
- **Chosen: `poll_oneoff` succeeds at once and says so.** The shim used to
  answer ENOSYS under a comment that promised an immediate return. The
  callers are the wasi libc's sleeps (flecs' frame pacing) and poll. An
  error from a sleep is ignored by flecs but is a failure to any caller
  that checks it, and "the sleep is over" is the answer that lets every
  caller go on. So the call now returns success and reports each clock
  subscription as elapsed; a descriptor subscription gets an ENOTSUP
  event. No time passes, so the first skipped wait is written to the
  console with its length. A caller that loops until a clock has advanced
  still terminates, because the clocks are real.
- **Chosen: the shim answers the nine imports and nothing else.** The
  extra functions (fd_read, environ, sched_yield and others) are gone:
  the build fails by name if the module asks for a tenth, which is better
  than a guessed answer nobody reads. Unknown clock ids answer EINVAL and
  a write, close or seek on a descriptor that is not open answers EBADF,
  each with one console warning. The two expected probe answers (ESPIPE
  for a seek on stdout, EBADF ending the preopen walk) are documented and
  not reported.
- **Chosen: the wasm-bindgen install is announced and can be refused.**
  The script prints the exact `cargo install` command and that it fetches
  from crates.io before running it; `CAMCRAFT_WASM_NO_INSTALL=1` turns it
  into a refusal naming the version to put on PATH. Whether CI and the
  release build may run that install is the maintainer decision this
  switch exists for.
- **Chosen: the pixel-ratio clamp lives in `view_pass::target_extent`.**
  Both shells clamped (or, for the first browser frame, forgot to clamp)
  the ratio before calling it. It now holds the ratio to 0.1..=8 itself
  (1 for NaN), so no caller can size a zero or runaway target.
