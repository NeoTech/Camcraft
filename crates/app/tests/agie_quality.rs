//! Gate map: a saved Wire Contour condition selects the native Agie quality after a project reload.
use std::{fs, process::Command};

const SCRIPT: &str = include_str!("../../../examples/agie/fine-punch.camscript");

#[test]
fn the_saved_condition_selects_the_exact_requested_quality() {
    let temp = tempfile::tempdir().unwrap();
    let example = temp.path().join("examples/agie");
    fs::create_dir_all(&example).unwrap();
    fs::write(
        example.join("punch.obj"),
        include_str!("../../../examples/agie/punch.obj"),
    )
    .unwrap();
    let mut script = SCRIPT.replace(
        "cam.generate();",
        "cam.save(\"fine.camproj\");\ncam.load(\"fine.camproj\");\ncam.generate();",
    );
    script = script.replace("cam.post(", "const receipt = cam.post(");
    script.push_str(
        r#"if (receipt.working_steps_generated !== false ||
              !receipt.controller_review.includes("CCB035") ||
              !receipt.controller_review.includes("245, 155, 140, 134")) {
            throw new Error("The post must report the selected quality and unarmed state");
        }"#,
    );
    fs::write(example.join("fine-punch.camscript"), script).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_camcraft-cli"))
        .current_dir(temp.path())
        .args(["script", "examples/agie/fine-punch.camscript"])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let sbl = fs::read_to_string(temp.path().join("FINE20/FINE20.SBL")).unwrap();
    let baseline = include_str!("golden/nc/agie-native/CCSEP20.SBL");
    assert_eq!(
        sbl,
        baseline
            .replace("CCSEP20", "FINE20")
            .replace("CCB030", "CCB035")
            .replace("0.30,1.50,1.00,F,F,F", "0.35,1.50,1.00,F,F,F")
    );
}
