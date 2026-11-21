//! Gate map: native Agie grouped jobs from two complete modeled punches through camcraft-cli.
//! Provenance: CAMCraft's original synthetic rectangular and five-sided vertical solids.
use std::{fs, path::Path, process::Command};

const SCRIPT: &str = include_str!("../../../examples/agie/punch-pair.camscript");
const LEFT: &str = include_str!("../../../examples/agie/pair-left.obj");
const RIGHT: &str = include_str!("../../../examples/agie/pair-right.obj");

fn run(dir: &Path, script: &str) -> std::process::Output {
    let examples = dir.join("examples/agie");
    fs::create_dir_all(&examples).unwrap();
    fs::write(examples.join("pair-left.obj"), LEFT).unwrap();
    fs::write(examples.join("pair-right.obj"), RIGHT).unwrap();
    fs::write(examples.join("punch-pair.camscript"), script).unwrap();
    Command::new(env!("CARGO_BIN_EXE_camcraft-cli"))
        .current_dir(dir)
        .args(["script", "examples/agie/punch-pair.camscript"])
        .output()
        .unwrap()
}

#[test]
fn the_two_model_owners_have_distinct_programs_holes_and_captures() {
    let temp = tempfile::tempdir().unwrap();
    let result = run(temp.path(), SCRIPT);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let dir = temp.path().join("PAIR20");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 4);
    let sbl = fs::read_to_string(dir.join("PAIR20.SBL")).unwrap();
    let map = fs::read_to_string(dir.join("PAIR20.SBR")).unwrap();
    assert_eq!(sbl.matches("JE_Create(GROUP,").count(), 1);
    assert_eq!(sbl.matches("JE_Create(WORK,").count(), 2);
    assert_eq!(sbl.matches("ID_SEPCUT,\"1.0\"").count(), 2);
    for (num, pierce, wall) in [(1, "-30.000", "-24.00000"), (2, "1.000", "7.00000")] {
        let alias = format!("PAIR20{num}.ISO");
        let context = format!("\"PAIR20\",\"CUT{num}\"");
        assert!(sbl.contains(&format!(
            "{context},NULL,ID_GEOMETRY,\"AGIE.USING_{alias}\""
        )));
        assert!(sbl.contains(&format!("{context},\"stp1\",ID_POSX,\"{pierce}\"")));
        assert!(map.contains(&format!("AGIE.USING_{alias} IMPORT {alias};\r\n")));
        let iso = fs::read_to_string(dir.join(alias)).unwrap();
        assert!(iso.contains(&format!("G00 X{wall} Y0.00000")));
    }
}

#[test]
fn a_shared_source_an_unowned_model_or_a_crossing_thread_path_writes_nothing() {
    for script in [
        SCRIPT.replace("source: \"right\"", "source: \"left\""),
        SCRIPT.replace(
            "cam.generate();",
            "cam.import(\"examples/agie/pair-right.obj\", \"third\");\ncam.generate();",
        ),
        SCRIPT.replace("[1, 0]", "[-32, 0]"),
        SCRIPT.replacen("capture: true", "capture: false", 1),
    ] {
        let temp = tempfile::tempdir().unwrap();
        assert!(!run(temp.path(), &script).status.success());
        assert!(!temp.path().join("PAIR20").exists());
    }
}
