## #31 - Browser app diagnostics, dropped model bytes, and inline CpuEye fallback

### What was decided

- Add `camcraft_app::logging::dispatch(writer, browser)` using `.without_time().with_ansi(false)` for browser builds, and call `camcraft_app::logging::init()` in `camcraft_start`.
- Route bare-name model drops in `file_drop::take_drop` through `check_drop_bytes` and `deliver_model_bytes`, reading in-memory or async browser bytes into `file_pick::deliver`.
- Refactor `CpuEye` into `Mode::Threaded` and `Mode::Inline(Box<Inline>)` sharing `march_job`, falling back to inline on `wasm32` or `ErrorKind::Unsupported`.
- Request frame repaint in `WebShell::ui` when `StockEye::cpu_pending` returns true.
- Ship these slices under `Refs #31` without touching `PlayWorker` or `shell.rs`.

### Why

- `wasm32` lacks `SystemTime::now()`, OS threads, and local filesystem paths for dropped files.
- Reusing `tracing_subscriber::fmt` without timestamps or ANSI codes writes diagnostics to stderr without new crate dependencies.
- Boxing `Inline` satisfies `clippy::large_enum_variant` without suppressing the lint.
- Handling bare-name byte drops in `take_drop` keeps `classify` and `MESH_EXTENSIONS` untouched so PR `#2150` stays conflict-free.
- Deferring `PlayWorker` (`#2192`) and `shell.rs` (`#2110`, `#1330`) avoids stacking on blocked or unready PRs.

### Considered and dropped

- Adding `tracing-wasm` to `Cargo.toml`: dropped because Step 6 forbids dependency edits and stderr formatting already solves the panic.
- Adding `#[allow(clippy::large_enum_variant)]` on `Mode`: dropped in favor of `Box<Inline>`.
- Editing `file_drop::classify`: dropped because open PR `#2150` (`needs-maintainer`) modifies `classify` and `MESH_EXTENSIONS`.
- Implementing inline `PlayWorker` playback here: dropped because open PR `#2192` edits `stock_play_worker.rs`.

### Cost to change later

- Low. All changes are local to four modules and covered by native unit and integration tests.
