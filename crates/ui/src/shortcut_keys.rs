//! Platform key names for the shortcut hints (tips, hover text, status
//! lines). Provenance: egui's `Modifiers::command` (Cmd on macOS, Ctrl
//! elsewhere) is the key the keyboard layer binds, so the hints name the
//! same key; clean-room, no third-party text.

/// The key the platform's command shortcuts use: `Cmd` on macOS, `Ctrl`
/// elsewhere. This is the one place that decides the word.
pub const fn command_key() -> &'static str {
    if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" }
}

/// A command shortcut written for this platform: `chord("Shift+O")` is
/// `Cmd+Shift+O` on macOS and `Ctrl+Shift+O` elsewhere.
pub fn chord(rest: &str) -> String {
    format!("{}+{rest}", command_key())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_key_names_the_platform_key() {
        #[cfg(target_os = "macos")]
        assert_eq!(command_key(), "Cmd");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(command_key(), "Ctrl");
    }

    #[test]
    fn chord_prefixes_the_platform_key() {
        assert_eq!(chord("Z"), format!("{}+Z", command_key()));
        assert_eq!(chord("Shift+O"), format!("{}+Shift+O", command_key()));
    }
}
