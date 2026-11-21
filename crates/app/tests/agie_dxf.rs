//! Gate map: a stock-placed DXF punch through the public CLI and direct native post.
use std::{fs, path::Path, process::Command};
const SCRIPT: &str = include_str!("../../../examples/agie/racetrack.camscript");
const DXF: &str = include_str!("../../../examples/agie/racetrack.dxf");

fn run(dir: &Path, script: &str, drawing: &str) -> std::process::Output {
    let paths = dir.join("examples/agie");
    fs::create_dir_all(&paths).unwrap();
    fs::write(paths.join("racetrack.dxf"), drawing).unwrap();
    fs::write(paths.join("racetrack.camscript"), script).unwrap();
    Command::new(env!("CARGO_BIN_EXE_camcraft-cli"))
        .current_dir(dir)
        .args(["script", "examples/agie/racetrack.camscript"])
        .output()
        .unwrap()
}

#[test]
fn translated_and_rotated_dxf_punch_keeps_its_two_true_caps() {
    let dir = tempfile::tempdir().unwrap();
    let output = run(dir.path(), SCRIPT, DXF);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let iso = fs::read_to_string(dir.path().join("RACE20/RACE20.ISO")).unwrap();
    assert_eq!(iso.lines().count(), 8);
    assert!(iso.contains("G00 X2.00000 Y-9.00000"));
    assert!(iso.contains("G03 X10.00000 Y11.00000 I0.00000 J10.00000"));
    assert!(iso.contains("G03 X-6.00000 Y-9.00000 I0.00000 J-10.00000"));
}

#[test]
fn moved_stale_open_or_misplaced_profile_refuses_before_publication() {
    let open = DXF.replacen("50\n0\n51\n180", "50\n5\n51\n180", 1);
    let extra = DXF.replacen(
        "0\nENDSEC\n0\nEOF",
        "0\nCIRCLE\n8\nprofile\n10\n3\n20\n3\n40\n1\n0\nENDSEC\n0\nEOF",
        1,
    );
    for (script,drawing) in [
        (SCRIPT.to_owned(),open),(SCRIPT.to_owned(),extra),
        (SCRIPT.replace("cam.post(", "cam.call(\"move_dxf_profile\", {name: \"racetrack\", position:[3,1], angle:90});\ncam.post("),DXF.into()),
        (SCRIPT.replace("position: [2, 1]", "position: [38, 1]"),DXF.into()),
        (SCRIPT.replace("[2, -16]", "[2, 1]"),DXF.into()),
    ] {
        let dir=tempfile::tempdir().unwrap();
        let result=run(dir.path(),&script,&drawing);
        assert!(!result.status.success());
        assert!(!dir.path().join("RACE20").exists());
    }
}

const CIRCLE_SCRIPT: &str = include_str!("../../../examples/agie/circle-punch.camscript");
const CIRCLE_DXF: &str = include_str!("../../../examples/agie/circle-punch.dxf");

#[test]
fn a_source_circle_keeps_the_diameter_endpoints_and_radial_threading_hole() {
    let dir = tempfile::tempdir().unwrap();
    // The runner supplies one drawing and one script at the racetrack test paths.
    let script = CIRCLE_SCRIPT.replace("circle-punch", "racetrack");
    let result = run(dir.path(), &script, CIRCLE_DXF);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let iso = fs::read_to_string(dir.path().join("CIRCLE20/CIRCLE20.ISO")).unwrap();
    assert_eq!(
        iso.lines().collect::<Vec<_>>(),
        [
            "( CIRCLE20.ISO )",
            "N0001 G00 X-10.00000 Y0.00000 ;",
            "N0002 G03 X10.00000 Y0.00000 I10.00000 J0.00000 ;",
            "N0003 G03 X-10.00000 Y0.00000 I-10.00000 J0.00000 ;",
            "N0004 M02 ;",
        ]
    );
    let sbl = fs::read_to_string(dir.path().join("CIRCLE20/CIRCLE20.SBL")).unwrap();
    assert!(sbl.contains("ID_POSX,\"-17.000\""));
    assert!(sbl.contains("ID_SEPCUT,\"1.0\"") && sbl.contains("ID_PUNCH,c_punch"));
}

#[test]
fn a_nonradial_hole_or_unqualified_circle_radius_cannot_publish_a_job() {
    let script = CIRCLE_SCRIPT.replace("circle-punch", "racetrack");
    for (s, d) in [
        (
            script.replace("[-17, 0]", "[-17, 0.001]"),
            CIRCLE_DXF.to_owned(),
        ),
        (
            script.replace("[-17, 0]", "[-12, 0]"),
            CIRCLE_DXF.to_owned(),
        ),
        (script.clone(), CIRCLE_DXF.replace("40\n10\n", "40\n0.4\n")),
        (
            script.replace(
                "cam.post(",
                "cam.call(\"move_dxf_profile\", {name:\"racetrack\",position:[0,2]});\ncam.post(",
            ),
            CIRCLE_DXF.to_owned(),
        ),
        (script, CIRCLE_DXF.replace("40\n10\n", "40\n5.99\n")),
        (
            CIRCLE_SCRIPT.replace("circle-punch", "racetrack"),
            CIRCLE_DXF.replace("40\n10\n", "40\n15.01\n"),
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        assert!(!run(dir.path(), &s, &d).status.success());
        assert!(!dir.path().join("CIRCLE20").exists());
    }
}
