//! A selected reentrant DXF die opening of lines and arcs through the public CLI and the
//! native post (#3178). Provenance: the original examples/agie/reentrant-die drawing and
//! the control-checked DIE20 setup of #2957.
use super::dxf::run;
use std::fs;

const SCRIPT: &str = include_str!("../../../../examples/agie/reentrant-die.camscript");
const DXF: &str = include_str!("../../../../examples/agie/reentrant-die.dxf");
const DIE20: &str = include_str!("../../../post/tests/golden/agie-setup/DIE20.SBL");

fn script() -> String {
    SCRIPT.replace("reentrant-die", "racetrack")
}

/// The control-checked die setup with only the stem, the 80 x 60 stock and the hole changed.
fn die_setup(stem: &str, hole: [&str; 2]) -> String {
    let piece = |field: &str, value: &str| format!("NULL,{field},\"{value}\",je_dim)");
    let hole_at = |axis: &str, value: &str| format!("\"stp1\",ID_POS{axis},\"{value}\",je_dim)");
    DIE20
        .replace(&piece("ID_DIML", "70.000"), &piece("ID_DIML", "80.000"))
        .replace(&piece("ID_DIMB", "70.000"), &piece("ID_DIMB", "60.000"))
        .replace(&piece("ID_POSPOSX", "-35.000"), &piece("ID_POSPOSX", "-40.000"))
        .replace(&piece("ID_POSPOSY", "-35.000"), &piece("ID_POSPOSY", "-30.000"))
        .replace(&hole_at("X", "-5.500"), &hole_at("X", hole[0]))
        .replace(&hole_at("Y", "0.000"), &hole_at("Y", hole[1]))
        .replace("DIE20", stem)
}

#[test]
fn a_reentrant_line_and_arc_die_posts_g02_g03_and_the_checked_die_setup() {
    let dir = tempfile::tempdir().unwrap();
    let output = run(dir.path(), &script(), DXF);
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let job = dir.path().join("REENT20");
    // The golden is explained line by line in camcraft_post's opening_tests.
    assert_eq!(
        fs::read(job.join("REENT20.ISO")).unwrap(),
        include_bytes!("../golden/nc/agie-native/REENT20.ISO").as_slice()
    );
    let sbl = fs::read_to_string(job.join("REENT20.SBL")).unwrap();
    assert_ne!(die_setup("REENT20", ["0.000", "-7.000"]), DIE20.replace("DIE20", "REENT20"));
    assert_eq!(sbl, die_setup("REENT20", ["0.000", "-7.000"]));
    assert!(sbl.contains("ID_PUNCH,c_die"));
    assert_eq!(
        fs::read_to_string(job.join("REENT20.SBR")).unwrap(),
        "AGIE.USING_REENT20.ISO IMPORT REENT20.ISO;\r\n"
    );
}

#[test]
fn a_small_concave_arc_an_island_or_an_outside_hole_publishes_nothing() {
    // The lower tooth fillet: R1 about X19 Y-4 becomes R0.2 about X19.8 Y-3.2.
    let wall = "LINE\n8\nprofile\n10\n20\n20\n-10\n11\n20\n21\n-4\n";
    let fillet = "ARC\n8\nprofile\n10\n19\n20\n-4\n40\n1\n50\n0\n51\n90\n";
    let tooth = "LINE\n8\nprofile\n10\n19\n20\n-3\n11\n14\n21\n-3\n";
    for part in [wall, fillet, tooth] {
        assert!(DXF.contains(part), "{part}");
    }
    let small = DXF
        .replace(wall, "LINE\n8\nprofile\n10\n20\n20\n-10\n11\n20\n21\n-3.2\n")
        .replace(fillet, "ARC\n8\nprofile\n10\n19.8\n20\n-3.2\n40\n0.2\n50\n0\n51\n90\n")
        .replace(tooth, "LINE\n8\nprofile\n10\n19.8\n20\n-3\n11\n14\n21\n-3\n");
    let island = DXF.replace("0\nENDSEC\n0\nEOF", "0\nCIRCLE\n8\nmarks\n10\n-10\n20\n0\n40\n0.5\n0\nENDSEC\n0\nEOF");
    for (script, drawing, why) in [
        (script(), small, "Native arc offset rule: the arc about X19.80000 Y-3.20000 has radius 0.20000 mm"),
        (script(), island, "unselected DXF primitive"),
        (script().replace("[0, -7]", "[0, 2]"), DXF.to_owned(), "inside"),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let output = run(dir.path(), &script, &drawing);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success() && stderr.contains(why), "{why}: {stderr}");
        assert!(!dir.path().join("REENT20").exists());
    }
}
