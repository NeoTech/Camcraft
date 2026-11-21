//! Gate map: native Agie grouped jobs for independent punch and die contours through camcraft-cli.
//! Provenance: CAMCraft's original synthetic retained solids and through-openings.
use std::{fs, path::Path, process::Command};

const SCRIPT: &str = include_str!("../../../examples/agie/punch-pair.camscript");
const TRIO: &str = include_str!("../../../examples/agie/punch-trio.camscript");
const GRID: &str = include_str!("../../../examples/agie/punch-grid.camscript");
const DIE_SCRIPT: &str = include_str!("../../../examples/agie/die-pair.camscript");
const PLATE: &str = include_str!("../../../examples/agie/die-pair.obj");
const LEFT: &str = include_str!("../../../examples/agie/pair-left.obj");
const RIGHT: &str = include_str!("../../../examples/agie/pair-right.obj");

fn run(dir: &Path, script: &str) -> std::process::Output {
    let examples = dir.join("examples/agie");
    fs::create_dir_all(&examples).unwrap();
    fs::write(examples.join("pair-left.obj"), LEFT).unwrap();
    fs::write(examples.join("pair-right.obj"), RIGHT).unwrap();
    fs::write(examples.join("die-pair.obj"), PLATE).unwrap();
    fs::write(examples.join("group.camscript"), script).unwrap();
    Command::new(env!("CARGO_BIN_EXE_camcraft-cli"))
        .current_dir(dir)
        .args(["script", "examples/agie/group.camscript"])
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

#[test]
fn two_complete_rims_on_one_plate_bind_independent_inward_works() {
    let temp = tempfile::tempdir().unwrap();
    let result = run(temp.path(), DIE_SCRIPT);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let dir = temp.path().join("DUODI20");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 4);
    let sbl = fs::read_to_string(dir.join("DUODI20.SBL")).unwrap();
    let sbr = fs::read_to_string(dir.join("DUODI20.SBR")).unwrap();
    for (index, hole, wall) in [(1, "-35.500", "-42.50000"), (2, "24.500", "17.50000")] {
        let name = format!("DUODI20{index}.ISO");
        let work = format!("\"DUODI20\",\"CUT{index}\",");
        assert!(sbl.contains(&format!("{work}NULL,ID_PUNCH,c_die")));
        assert!(sbl.contains(&format!("{work}NULL,ID_SEPCUT,\"1.0\"")));
        assert!(sbl.contains(&format!("{work}\"stp1\",ID_POSX,\"{hole}\"")));
        assert!(sbl.contains(&format!("{work}NULL,ID_GEOMETRY,\"AGIE.USING_{name}\"")));
        assert!(sbr.contains(&format!("AGIE.USING_{name} IMPORT {name};\r\n")));
        let iso = fs::read_to_string(dir.join(name)).unwrap();
        assert!(iso.contains(&format!("G00 X{wall} Y0.00000")));
    }
}

#[test]
fn missing_or_unsafe_die_works_leave_no_output_directory() {
    for (case, script) in [
        (
            "same opening twice",
            DIE_SCRIPT.replace("x: 24.5", "x: -35.5"),
        ),
        (
            "unselected opening",
            DIE_SCRIPT.replace(", {name: \"DieR\", x: 24.5}", ""),
        ),
        (
            "pierce in the other opening",
            DIE_SCRIPT.replace("start: [op.x, 0]", "start: [-35.5, 0]"),
        ),
        (
            "partial rim",
            DIE_SCRIPT.replace("rim.chains.map", "rim.chains.slice(1).map"),
        ),
        (
            "extra body",
            DIE_SCRIPT.replace(
                "cam.generate();",
                "cam.import(\"examples/agie/pair-right.obj\", \"extra\"); cam.generate();",
            ),
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let result = run(temp.path(), &script);
        assert!(!result.status.success(), "{case}");
        assert!(!temp.path().join("DUODI20").exists(), "{case}");
    }
}

#[test]
fn three_independently_placed_models_export_their_own_geometry_and_starts() {
    let temp = tempfile::tempdir().unwrap();
    let result = run(temp.path(), TRIO);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let dir = temp.path().join("TRIO20");
    let map = fs::read_to_string(dir.join("TRIO20.SBR")).unwrap();
    let piece = fs::read_to_string(dir.join("TRIO20.SBL")).unwrap();
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 5);
    assert_eq!(piece.matches("JE_Create(WORK,").count(), 3);
    assert_eq!(piece.matches("ID_SEPCUT,\"1.0\"").count(), 3);
    for (index, x) in [-32, 0, 32].into_iter().enumerate() {
        let n = index + 1;
        let iso_name = format!("TRIO20{n}.ISO");
        assert!(map.contains(&format!("AGIE.USING_{iso_name} IMPORT {iso_name};")));
        let work = format!("\"TRIO20\",\"CUT{n}\",");
        assert!(piece.contains(&format!("{work}NULL,ID_GEOMETRY,\"AGIE.USING_{iso_name}\"")));
        assert!(piece.contains(&format!("{work}\"stp1\",ID_POSX,\"{}.000\"", x - 13)));
        let iso = fs::read_to_string(dir.join(iso_name)).unwrap();
        assert!(iso.contains(&format!("G00 X{}.00000 Y0.00000", x - 7)));
    }
}

#[test]
fn a_nonadjacent_collision_fails_before_publishing_any_group_resource() {
    let dir = tempfile::tempdir().unwrap();
    // Place work 3 on work 1 while keeping work 2 clear of both.
    let script = TRIO.replace("[32, 0]]", "[-32, 0]]");
    let result = run(dir.path(), &script);
    let message = String::from_utf8_lossy(&result.stderr);
    assert!(!result.status.success());
    assert!(
        message.contains("TRIO201 and TRIO203 share a cutting area"),
        "{message}"
    );
    assert!(!dir.path().join("TRIO20").exists());
}

#[test]
fn nine_model_owners_resolve_all_resources_and_the_same_registered_quality() {
    let temp = tempfile::tempdir().unwrap();
    let result = run(temp.path(), GRID);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let dir = temp.path().join("NINE20");
    assert_eq!(fs::read_dir(&dir).unwrap().count(), 11);
    let map = fs::read_to_string(dir.join("NINE20.SBR")).unwrap();
    let piece = fs::read_to_string(dir.join("NINE20.SBL")).unwrap();
    assert_eq!(map.lines().count(), 9);
    assert_eq!(piece.matches("JE_Create(WORK,").count(), 9);
    assert_eq!(piece.matches("ID_SEPCUT,\"1.0\"").count(), 9);
    assert_eq!(piece.matches("JE_CreateQuality(").count(), 1);
    assert!(piece.contains(r#"JE_CreateQuality(je_piece,"CCB030","0.30,1.50,1.00,F,F,F",je_q1)"#));
    for i in 0..9 {
        let (x, y) = ((i % 3 - 1) * 32, (i / 3 - 1) * 32);
        let alias = format!("NINE20{}.ISO", i + 1);
        let work = format!("\"NINE20\",\"CUT{}\",", i + 1);
        assert!(map.contains(&format!("AGIE.USING_{alias} IMPORT {alias};")));
        assert!(piece.contains(&format!("{work}NULL,ID_GEOMETRY,\"AGIE.USING_{alias}\"")));
        assert!(piece.contains(&format!("{work}\"stp1\",ID_POSX,\"{}.000\"", x - 13)));
        assert!(piece.contains(&format!("{work}\"stp1\",ID_POSY,\"{y}.000\"")));
        assert!(piece.contains(&format!("{work}NULL,ID_QUALITY,je_q1,je_dim")));
        let iso = fs::read_to_string(dir.join(alias)).unwrap();
        assert!(iso.contains(&format!("G00 X{}.00000 Y{y}.00000", x - 7)));
    }
}

#[test]
fn exceeding_nine_works_or_leaving_an_unused_owner_writes_nothing() {
    for script in [
        // Add a tenth independent operation and model before any import.
        GRID.replacen(
            "for (const [i, center]",
            "centers.push([0, 16]);\nfor (const [i, center]",
            1,
        ),
        GRID.replace(
            "cam.generate();",
            "cam.import(\"examples/agie/pair-left.obj\", \"unused\"); cam.generate();",
        ),
        GRID.replace(
            "source: \"Retained\" + (i + 1)",
            "source: \"Retained\" + (i === 8 ? 1 : i + 1)",
        ),
    ] {
        let temp = tempfile::tempdir().unwrap();
        assert!(!run(temp.path(), &script).status.success());
        assert!(!temp.path().join("NINE20").exists());
    }
}
