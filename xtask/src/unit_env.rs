//! The caller environment that enters cargo's unit hash, pinned for every cargo call of a gate (#3150),
//! and the count of unit directories that a gate run adds.
//! Provenance: our own build tooling (#900, #3150); the Cargo book, "Environment variables" (CARGO_INCREMENTAL,
//! CARGO_PROFILE_<name>_<key>) and "Build cache" (a unit's hash covers its profile and its rustflags).
//!
//! Why: cargo names each unit directory (`target/<profile>/build/<package>/<hash>/`) by a hash of its
//! profile, among other inputs. `CARGO_INCREMENTAL` and the `CARGO_PROFILE_*` variables change the
//! profile, so each agent that runs a gate with other values gives every workspace unit a new
//! directory. Cargo never removes the old one: one `camcraft-app` variant is about 600 units of ~190 MB.

use std::fs;
#[path = "unit_env/config.rs"]
mod config;
use std::path::Path;
use std::process::Command;

/// The variables that each gate cargo call gets, whatever the caller set. Incremental is off: sccache
/// cannot cache an incremental unit, and incremental data adds to the size of each unit.
pub const PINNED: [(&str, &str); 1] = [("CARGO_INCREMENTAL", "0")];

/// True for a caller variable that the gate removes: it changes the profile, and with it the unit hash.
pub fn is_removed(name: &str) -> bool {
    name.starts_with("CARGO_PROFILE_") || name == "CARGO_BUILD_INCREMENTAL"
}

/// Caller variables that the gate keeps but reports: they also enter the unit hash.
pub const NOTED: [&str; 4] = ["RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "CARGO_BUILD_RUSTFLAGS", "CARGO_BUILD_TARGET"];

fn is_noted(name: &str) -> bool {
    NOTED.contains(&name) || name.strip_prefix("CARGO_TARGET_").and_then(|s| s.strip_suffix("_RUSTFLAGS")).is_some_and(|s| !s.is_empty())
}

/// The variables of `vars` that the gate removes from its cargo calls, sorted.
pub fn removed(vars: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out: Vec<String> = vars.into_iter().filter(|k| is_removed(k)).collect();
    out.sort();
    out
}

/// Sets the pinned variables on `cmd` and removes the profile variables of the caller.
pub fn apply(cmd: &mut Command) {
    for name in removed(std::env::vars_os().filter_map(|(k, _)| k.into_string().ok())) {
        cmd.env_remove(name);
    }
    cmd.envs(PINNED);
}

/// The lines that a gate prints at its start: the pin, and each caller variable that it removes or notes.
pub fn notes(vars: &[(String, String)]) -> Vec<String> {
    let mut out = vec![format!(
        "unit env: {} for every cargo step (#3150: fixed profile environment; target, rustflags and configuration are preserved)",
        PINNED.iter().map(|(k, v)| format!("{k}={v}")).collect::<Vec<_>>().join(" ")
    )];
    let gone = removed(vars.iter().map(|(k, _)| k.clone()));
    if !gone.is_empty() {
        out.push(format!("unit env: removed {} (it changes the profile and gives each unit a new directory)", gone.join(", ")));
    }
    for (k, _) in vars.iter().filter(|(k, v)| is_noted(k) && !v.is_empty()) {
        out.push(format!("unit env: note: {k} is set; each other value gives each unit a new directory"));
    }
    out
}

/// Notes for the actual Cargo working directories in a gate plan, including UI-only config.
pub fn plan_notes(vars: &[(String, String)], root: &Path, plan: &crate::plan::Plan) -> Vec<String> {
    let mut dirs = vec![root.to_path_buf()];
    if plan.steps.iter().any(|s| s.in_ui) {
        // Two components: a "/" inside one join stays in the noted path on Windows (#4454).
        dirs.push(root.join("crates").join("ui"));
    }
    let home = std::env::var_os("CARGO_HOME").filter(|s| !s.is_empty()).map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(|p| std::path::PathBuf::from(p).join(".cargo")));
    let mut out = notes(vars);
    out.extend(config::notes(&dirs, home.as_deref()));
    out
}

/// The unit directories (`build/<package>/<hash>/` with a `fingerprint/`) under `target/debug` and
/// `target/release` of `root`; a missing directory counts 0.
pub fn units(root: &Path) -> usize {
    ["debug", "release"]
        .iter()
        .map(|p| root.join("target").join(p).join("build"))
        .flat_map(|b| fs::read_dir(b).into_iter().flatten())
        .filter_map(Result::ok)
        .flat_map(|package| fs::read_dir(package.path()).into_iter().flatten())
        .filter_map(Result::ok)
        .filter(|unit| unit.path().join("fingerprint").is_dir())
        .count()
}

/// The summary line of the unit count: how many unit directories the run added.
pub fn added_line(before: usize, after: usize) -> String {
    let added = after.saturating_sub(before);
    if added == 0 {
        format!("  unit directories: {after}, none added (the run reused every unit)")
    } else {
        format!("  unit directories: {after}, {added} added (a second run on the same tree must add none, #3150)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_variables_are_removed_and_others_kept() {
        let vars = ["CARGO_PROFILE_DEV_DEBUG", "PATH", "CARGO_BUILD_INCREMENTAL", "CARGO_PROFILE_TEST_OPT_LEVEL", "RUSTFLAGS", "CARGO_BUILD_JOBS"];
        assert_eq!(
            removed(vars.iter().map(|s| s.to_string())),
            ["CARGO_BUILD_INCREMENTAL", "CARGO_PROFILE_DEV_DEBUG", "CARGO_PROFILE_TEST_OPT_LEVEL"]
        );
        assert!(!is_removed("CARGO_INCREMENTAL"), "the gate sets CARGO_INCREMENTAL itself");
    }

    #[test]
    fn every_gate_cargo_call_gets_the_same_unit_env() {
        // Two callers with different profile env must give the same command env (#3150).
        let env_of = |cmd: &Command| {
            let mut e: Vec<(String, Option<String>)> = cmd
                .get_envs()
                .map(|(k, v)| (k.to_string_lossy().into_owned(), v.map(|v| v.to_string_lossy().into_owned())))
                .filter(|(k, _)| k == "CARGO_INCREMENTAL" || is_removed(k))
                .collect();
            e.sort();
            e
        };
        let mut a = Command::new("cargo");
        apply(&mut a);
        let mut b = Command::new("cargo");
        b.env("CARGO_INCREMENTAL", "1");
        apply(&mut b);
        assert_eq!(env_of(&a), env_of(&b));
        assert!(env_of(&a).contains(&("CARGO_INCREMENTAL".into(), Some("0".into()))));
    }

    #[test]
    fn direct_cargo_steps_apply_the_unit_env() {
        // The one place where a gate step starts cargo applies the pin (#3150).
        let src = include_str!("direct.rs");
        let body = src.split("pub fn cargo(").nth(1).unwrap_or_default();
        let body = body.split("\n}\n").next().unwrap_or_default();
        assert!(body.contains("crate::unit_env::apply(&mut cmd)"), "direct::cargo must apply the unit env");
    }

    #[test]
    fn batch_steps_share_the_flags_that_enter_the_unit_hash() {
        // A step with its own profile, features or config would build each crate again under a new hash.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
        let plan = crate::plan::batch(&root);
        assert!(plan.steps.len() > 10, "the batch plan has its steps");
        let hashed = ["--config", "--features", "--all-features", "--no-default-features", "--profile", "--release", "-Z"];
        for s in &plan.steps {
            let bad: Vec<&String> = s.args.iter().filter(|a| hashed.iter().any(|h| a == h || a.starts_with(&format!("{h}=")))).collect();
            assert!(bad.is_empty(), "step {} sets {bad:?}: it changes the unit hash", s.name);
        }
    }

    #[test]
    fn notes_name_the_pin_and_each_removed_or_noted_variable() {
        let vars = vec![
            ("CARGO_PROFILE_DEV_DEBUG".to_string(), "true".to_string()),
            ("RUSTFLAGS".to_string(), "-Ctarget-cpu=native".to_string()),
            ("CARGO_ENCODED_RUSTFLAGS".to_string(), String::new()),
        ];
        let n = notes(&vars);
        assert!(n[0].contains("CARGO_INCREMENTAL=0"));
        assert!(n.iter().any(|l| l.contains("removed CARGO_PROFILE_DEV_DEBUG")));
        assert!(n.iter().any(|l| l.contains("note: RUSTFLAGS")));
        assert!(!n.iter().any(|l| l.contains("CARGO_ENCODED_RUSTFLAGS")), "an empty value is no flag");
        assert_eq!(notes(&[]).len(), 1);
    }

    #[test]
    fn target_inputs_are_noted_without_changing_or_printing_values() {
        let vars = vec![
            ("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS".into(), "private flags".into()),
            ("CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS".into(), "".into()),
            ("CARGO_BUILD_TARGET".into(), "aarch64-unknown-linux-gnu".into()),
            ("CARGO_TARGET_DIR".into(), "target".into()),
            ("CARGO_TARGET_X_LINKER".into(), "cc".into()),
        ];
        let text = notes(&vars).join("\n");
        assert!(text.contains("note: CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS"));
        assert!(text.contains("note: CARGO_BUILD_TARGET"));
        assert!(!text.contains("WASM32"));
        assert!(!text.contains("CARGO_TARGET_DIR"));
        assert!(!text.contains("LINKER"));
        assert!(!text.contains("private flags"));
        assert!(vars.iter().all(|(name, _)| !is_removed(name)));
        assert_eq!(notes(&[("CARGO_BUILD_TARGET".into(), String::new())]).len(), 1);
        assert!(!is_noted("CARGO_TARGET__RUSTFLAGS"));
    }

    #[test]
    fn inherited_profile_values_are_removed_from_the_child() {
        const PHASE: &str = "CAMCRAFT_UNIT_ENV_TEST_PHASE";
        const PROFILE: &str = "CARGO_PROFILE_DEV_DEBUG";
        let phase = std::env::var(PHASE).unwrap_or_default();
        if phase == "clean" {
            assert!(std::env::var_os(PROFILE).is_none());
            assert!(std::env::var_os("CARGO_BUILD_INCREMENTAL").is_none());
            assert_eq!(std::env::var("CARGO_INCREMENTAL").unwrap(), "0");
            assert_eq!(std::env::var("CARGO_BUILD_TARGET").unwrap(), "aarch64-unknown-linux-gnu");
            assert_eq!(std::env::var("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS").unwrap(), "-Ctarget-cpu=generic");
            return;
        }
        let mut cmd = Command::new(std::env::current_exe().unwrap());
        cmd.args(["--exact", "unit_env::tests::inherited_profile_values_are_removed_from_the_child", "--nocapture"]);
        if phase == "caller" {
            assert_eq!(std::env::var(PROFILE).unwrap(), "true");
            apply(&mut cmd);
            cmd.env(PHASE, "clean");
        } else {
            cmd.env(PROFILE, "true").env("CARGO_BUILD_INCREMENTAL", "true").env("CARGO_INCREMENTAL", "1").env(PHASE, "caller");
            cmd.env("CARGO_BUILD_TARGET", "aarch64-unknown-linux-gnu").env("CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_RUSTFLAGS", "-Ctarget-cpu=generic");
        }
        let output = cmd.output().unwrap();
        assert!(output.status.success(), "{output:?}");
    }

    #[test]
    fn plan_notes_only_scan_ui_config_for_ui_steps() {
        let root = std::env::temp_dir().join(format!("xtask-plan-config-{}", std::process::id()));
        // Build the expected path by components: on Windows a "/" inside a join stays in the
        // displayed path, and the note text then differs from this path (#4454).
        let ui = root.join("crates").join("ui").join(".cargo");
        fs::create_dir_all(&ui).unwrap();
        fs::write(ui.join("config.toml"), "[profile.test]\ndebug = true\n").unwrap();
        let mut plan = crate::plan::Plan::default();
        let ui_path = ui.join("config.toml").display().to_string();
        assert!(!plan_notes(&[], &root, &plan).iter().any(|s| s.contains(&ui_path)));
        plan.steps.push(crate::plan::Step::new("UI test", &["test"], "ui"));
        assert!(plan_notes(&[], &root, &plan).iter().any(|s| s.contains(&ui_path)));
        assert_eq!(fs::read_to_string(ui.join("config.toml")).unwrap(), "[profile.test]\ndebug = true\n");
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn plan_notes_join_the_ui_directory_by_components() {
        // A "/" inside a join keeps a mixed separator on Windows, so the noted config path
        // differs from a path built by components (#4454). Linux cannot show the mismatch.
        let src = include_str!("unit_env.rs");
        let body = src.split("pub fn plan_notes(").nth(1).unwrap_or_default();
        let body = body.split("\n}\n").next().unwrap_or_default();
        assert!(body.contains("join(\"crates\").join(\"ui\")"), "plan_notes must join crates and ui as two components");
        assert!(!body.contains("join(\"crates/"), "plan_notes must not join a path that contains a separator");
    }

    #[test]
    fn units_count_nested_unit_directories_only() {
        let root = std::env::temp_dir().join(format!("xtask-unit-env-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        for (profile, unit) in [("debug", "camcraft-app/aaaa"), ("debug", "camcraft-app/bbbb"), ("release", "camcraft-cl/cccc")] {
            let d = root.join("target").join(profile).join("build").join(unit).join("fingerprint");
            assert!(fs::create_dir_all(&d).is_ok());
        }
        // An old-layout build-script directory has no fingerprint/ under its children: not a unit.
        assert!(fs::create_dir_all(root.join("target/debug/build/ahash-1234/out")).is_ok());
        assert_eq!(units(&root), 3);
        assert_eq!(units(&root.join("missing")), 0);
        let _ = fs::remove_dir_all(&root);
        assert!(added_line(3, 3).contains("none added"));
        assert!(added_line(3, 603).contains("600 added"));
    }
}
