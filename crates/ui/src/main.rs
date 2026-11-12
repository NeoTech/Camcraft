// GUI entry point. Windows: built as a GUI-subsystem exe so a double-click opens
// no console window (#1278); the console and error-box plumbing is in win_console.
// Spec: Microsoft's public documentation of the PE subsystem flag and the Win32
// console and message-box APIs.
#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
mod win_console;

fn main() {
    #[cfg(windows)]
    {
        win_console::attach_parent_console();
        win_console::install_panic_hook();
    }
    camcraft_app::logging::init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = camcraft_ui::run(&args);
    #[cfg(windows)]
    if code != 0 {
        win_console::report_exit_failure(code);
    }
    std::process::exit(code);
}
