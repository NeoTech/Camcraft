# 1278 - Windows: no console window on double-click

- **Chosen:** `#![cfg_attr(windows, windows_subsystem = "windows")]` on
  `crates/ui/src/main.rs`, in debug and release alike, plus `AttachConsole(ATTACH_PARENT_PROCESS)`
  at the top of `main` so a terminal launch still shows stdout/stderr logging.
- **Why always, not release-only:** one behaviour on every build; a debug build run from
  a terminal still logs there through the attach. Failure of the attach (double-click) is ignored.
- **FFI instead of windows-sys:** windows-sys is only a transitive dependency; using it
  directly means a new Cargo.toml dependency with feature flags (maintainer decision). A
  single `extern "system"` declaration for kernel32 needs none. The only `unsafe` is that call.
- **Dropped:** `AllocConsole` fallback or a file log when no console exists (out of scope;
  tracing keeps writing to a stderr that is simply closed). The CLI binary stays a console app.
