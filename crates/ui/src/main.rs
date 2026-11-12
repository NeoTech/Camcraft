// GUI entry point. Windows: built as a GUI-subsystem exe so a double-click opens
// no console window (#1278); see attach_parent_console. Spec: Microsoft's public
// documentation of the PE subsystem flag and AttachConsole.
#![cfg_attr(windows, windows_subsystem = "windows")]

/// A GUI-subsystem exe starts with no console, so launched from a terminal its
/// stdout/stderr (tracing logs) would go nowhere. Attach to the parent's console
/// when there is one; with none (double-click) the call just fails and is ignored.
#[cfg(windows)]
fn attach_parent_console() {
    const ATTACH_PARENT_PROCESS: u32 = u32::MAX;
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn AttachConsole(process_id: u32) -> i32;
    }
    // SAFETY: AttachConsole takes a plain integer, touches no Rust memory, and
    // reports failure by return value, which is deliberately ignored.
    let _ = unsafe { AttachConsole(ATTACH_PARENT_PROCESS) };
}

#[cfg(not(windows))]
fn attach_parent_console() {}

fn main() {
    attach_parent_console();
    camcraft_app::logging::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(camcraft_ui::run(&args));
}
