//! Read-only unit-hash diagnostics for Cargo's hierarchical config files.
//! Provenance: Cargo book, Configuration, "Hierarchical structure" and "Configuration format".

use std::path::{Path, PathBuf};

/// Flag each selected file, not a merged/effective configuration. Key hints cover ordinary
/// table/dotted assignments; every file is reported even when its syntax has no recognized hint.
pub fn notes(dirs: &[PathBuf], cargo_home: Option<&Path>) -> Vec<String> {
    let mut paths = std::collections::BTreeSet::new();
    for dir in dirs {
        for parent in dir.ancestors() {
            if let Some(path) = selected(&parent.join(".cargo")) {
                paths.insert(path);
            }
        }
    }
    if let Some(home) = cargo_home {
        for dir in dirs {
            if let Some(path) = selected(&dir.join(home)) {
                paths.insert(path);
            }
        }
    }
    paths.into_iter().map(|path| {
        let detail = match std::fs::read_to_string(&path) {
            Ok(text) => {
                let keys = key_hints(&text);
                if keys.is_empty() { String::new() } else { format!("; key hints: {}", keys.join(", ")) }
            }
            Err(_) => "; cannot read key hints".into(),
        };
        format!("unit env: note: config {}{detail}; may supply rustflags/profile/target inputs; not effective configuration (Cargo resolves precedence; values are unchanged)", path.display())
    }).collect()
}

fn selected(dir: &Path) -> Option<PathBuf> {
    // Cargo prefers `config` over `config.toml` if both exist; do not report the ignored file's keys.
    ["config", "config.toml"].iter().map(|name| dir.join(name)).find(|p| p.exists())
}

fn key_hints(text: &str) -> Vec<String> {
    // This is a diagnostic hint scan, not a TOML loader. In particular, inline tables and escaped
    // key names are not expanded. The file-level warning above covers those cases too.
    // Multiline values can contain text that looks like a table or assignment. Fall back to the
    // file warning rather than misreport that text as configuration keys.
    if text.contains("\"\"\"") || text.contains("'''") { return Vec::new(); }
    let Ok(key_syntax) = regex::Regex::new(r#"^(?:[A-Za-z0-9_-]+|"[^"\\]*"|'[^']*')(?:\s*\.\s*(?:[A-Za-z0-9_-]+|"[^"\\]*"|'[^']*'))*$"#) else { return Vec::new(); };
    let Ok(parts_syntax) = regex::Regex::new(r#"[A-Za-z0-9_-]+|"[^"\\]*"|'[^']*'"#) else { return Vec::new(); };
    let mut table = String::new();
    let mut keys = std::collections::BTreeSet::new();
    for line in text.lines().map(str::trim) {
        if line.starts_with('#') || line.is_empty() { continue; }
        if let Some(header) = line.strip_prefix('[').and_then(|s| s.split_once(']')) {
            table = header.0.trim().to_string();
            continue;
        }
        let Some((key, _)) = line.split_once('=') else { continue; };
        let key = key.trim();
        let path = if table.is_empty() { key.into() } else { format!("{table}.{key}") };
        if !key_syntax.is_match(&path) { continue; }
        // Only Cargo's build/target rustflags and target selection, or a profile subtree. Avoid
        // unrelated alias/profile and env/RUSTFLAGS values. Never print configuration values.
        let parts: Vec<&str> = parts_syntax.find_iter(&path).map(|s| s.as_str().trim_matches(['\'', '"'])).collect();
        if parts.first() == Some(&"profile") || parts.as_slice() == ["build", "rustflags"] || parts.as_slice() == ["build", "target"] ||
            (parts.first() == Some(&"target") && parts.last() == Some(&"rustflags")) {
            keys.insert(path);
        }
    }
    keys.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn hierarchical_configs_report_paths_and_key_hints_not_effective_values() {
        let base = std::env::temp_dir().join(format!("xtask-config-{}", std::process::id()));
        let root = base.join("project");
        let ui = root.join("crates/ui");
        let home = base.join("cargo-home");
        for dir in [&base, &root, &ui, &home] {
            fs::create_dir_all(dir.join(".cargo")).unwrap();
        }
        fs::write(base.join(".cargo/config.toml"), "[build]\nrustflags = ['private flags']\ntarget = 'custom.json'\n").unwrap();
        fs::write(root.join(".cargo/config.toml"), "[profile.dev]\nopt-level = 2\n").unwrap();
        fs::write(ui.join(".cargo/config"), "[target.'cfg(unix)']\nrustflags = []\n").unwrap();
        // Cargo uses the extensionless file when both names exist.
        fs::write(ui.join(".cargo/config.toml"), "[profile.ignored]\ndebug = true\n").unwrap();
        fs::write(home.join("config.toml"), "build.rustflags = []\nprofile = { release = { debug = true } }\n").unwrap();
        let n = notes(&[root.clone(), ui.clone()], Some(&home));
        let text = n.join("\n");
        for key in ["build.rustflags", "build.target", "profile.dev.opt-level", "target.'cfg(unix)'.rustflags", "profile"] {
            assert!(text.contains(key), "missing {key}: {text}");
        }
        for path in [base.join(".cargo").join("config.toml"), root.join(".cargo").join("config.toml"), ui.join(".cargo").join("config"), home.join("config.toml")] {
            assert_eq!(n.iter().filter(|s| s.contains(&path.display().to_string())).count(), 1, "{text}");
        }
        assert!(!text.contains("profile.ignored"));
        assert!(!text.contains("private flags"));
        assert!(text.contains("not effective"));
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn key_hints_do_not_read_multiline_values_as_keys() {
        assert!(key_hints("[env]\nTEXT = '''\n[profile.fake]\ndebug = true\n'''\n").is_empty());
        assert!(key_hints("[build] # ] a comment\nrustflags = []\n").contains(&"build.rustflags".into()));
        assert!(key_hints("[build]\nrustflags-extra = []\n").is_empty());
    }

    #[test]
    fn quoted_keys_keep_their_toml_boundaries() {
        assert!(key_hints("'profile.fake' = {}\n[alias]\nrustflags = 'check'\n").is_empty());
        assert_eq!(key_hints("'build' . \"rustflags\" = []\n"), ["'build' . \"rustflags\""]);
        assert_eq!(key_hints("[target.'cfg(target_os = \"linux\")']\nrustflags = []\n"), ["target.'cfg(target_os = \"linux\")'.rustflags"]);
    }

    #[test]
    fn relative_cargo_home_is_resolved_for_each_command_directory() {
        let base = std::env::temp_dir().join(format!("xtask-relative-home-{}", std::process::id()));
        for dir in [&base, &base.join("crates/ui")] {
            fs::create_dir_all(dir.join("home")).unwrap();
            fs::write(dir.join("home/config.toml"), "build.target = 'wasm32-unknown-unknown'\n").unwrap();
        }
        let notes = notes(&[base.clone(), base.join("crates/ui")], Some(Path::new("home")));
        assert_eq!(notes.iter().filter(|s| s.contains("home/config.toml")).count(), 2);
        assert!(notes.iter().all(|s| !s.contains("wasm32-unknown-unknown")));
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn unrelated_config_keys_do_not_claim_hash_keys() {
        let base = std::env::temp_dir().join(format!("xtask-config-unrelated-{}", std::process::id()));
        fs::create_dir_all(base.join(".cargo")).unwrap();
        fs::write(base.join(".cargo/config.toml"), "# rustflags = []\n[alias]\nprofile = 'test'\nrustflags = 'check'\n[build]\njobs = 3\n").unwrap();
        let n = notes(std::slice::from_ref(&base), None);
        let path = base.join(".cargo/config.toml").display().to_string();
        let text = n.iter().find(|line| line.contains(&path)).expect("fixture config warning");
        assert!(!text.contains("key hints:"), "{text}");
        assert!(text.contains("config.toml"), "all files are flagged even without recognized syntax");
        let _ = fs::remove_dir_all(base);
    }
}
