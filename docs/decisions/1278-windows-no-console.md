# 1278 - Windows: no console window on double-click

- **Chosen:** `#![cfg_attr(windows, windows_subsystem = "windows")]` on
  `crates/ui/src/main.rs`, in debug and release alike, plus `AttachConsole(ATTACH_PARENT_PROCESS)`
  at the top of `main` so a terminal launch still shows stdout/stderr logging.
- **Why always, not release-only:** one behaviour on every build; a debug build run from
  a terminal still logs there through the attach. Failure of the attach (double-click) is ignored.
- **FFI instead of windows-sys:** windows-sys is only a transitive dependency; using it
  directly means a new Cargo.toml dependency with feature flags (maintainer decision). A
  handful of `extern "system"` declarations (kernel32, user32) in `crates/ui/src/win_console.rs`
  need none.
- **Std handles after the attach (review item 1):** `AttachConsole` does not give a
  GUI-subsystem process std handles. After a successful attach, each of STD_OUTPUT_HANDLE /
  STD_ERROR_HANDLE whose `GetStdHandle` is NULL or INVALID_HANDLE_VALUE is pointed at a
  `CreateFileW("CONOUT$")` handle with `SetStdHandle`; a valid handle (redirect, pipe) is left
  alone. Rust's std (`library/std/src/sys/stdio/windows.rs`, `write` -> `get_handle`) calls
  `GetStdHandle` on every write, so `std::io::stderr` - the tracing writer - picks it up.
- **Visible start-up failure (review item 2):** with no console attached, `main` shows a
  `MessageBoxW` when `run` returns non-zero, and a panic hook chained after the previous one
  shows the panic text the same way. One box per process (a panic then a non-zero exit shows
  one). With a console attached no box is shown: the terminal already has the log. The hook
  only formats and calls MessageBoxW, nothing in it can panic. Dropped: `AllocConsole` to show
  the log on failure (a console flashing up is what #1278 removes).
- **Log with no console:** with no console and no redirect, tracing still writes to a missing
  stderr and the log is lost; a per-user log file is follow-up #1323. The CLI binary stays a
  console app.
