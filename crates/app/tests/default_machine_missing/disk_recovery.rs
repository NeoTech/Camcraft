//! Disk recovery through the public default-machine reader (#2324).
//! Provenance: the file lookup and recovery contract of #2282 and #2319.
//! Gate map: covers app::posting::default_machine_checked, app::posting::files::default_mch_text.

use camcraft_app::posting::default_machine_checked;
use std::path::Path;
use std::process::Command;

const CHILD: &str = "CAMCRAFT_TEST_DEFAULT_MACHINE_DISK_RECOVERY";
const TEST: &str =
    "disk_recovery::a_repaired_default_machine_file_is_read_again_through_the_disk_reader";

#[test]
fn a_repaired_default_machine_file_is_read_again_through_the_disk_reader() {
    if std::env::var_os(CHILD).is_some() {
        check_disk_recovery();
        return;
    }

    // A copied test executable has fresh OnceLocks and reads only the
    // assets beside it. No process-wide environment or shipped file changes.
    let root = tempfile::tempdir().expect("isolated install folder");
    // One join per component: "assets/machines" keeps a '/' on Windows (#4120).
    std::fs::create_dir_all(root.path().join("assets").join("machines"))
        .expect("machines folder");
    let source = std::env::current_exe().expect("test executable");
    let exe = root
        .path()
        .join(source.file_name().expect("executable name"));
    std::fs::copy(source, &exe).expect("copy test executable");
    let output = Command::new(exe)
        .args(["--exact", TEST, "--nocapture", "--test-threads=1"])
        .env(CHILD, "1")
        .output()
        .expect("run isolated disk recovery test");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "disk recovery child failed: {}\n{stdout}\n{stderr}",
        output.status
    );
    assert!(
        stdout.contains("1 passed"),
        "the child must run the recovery test, not zero tests: {stdout}"
    );
}

fn check_disk_recovery() {
    let exe = std::env::current_exe().expect("isolated test executable");
    let machines = exe
        .parent()
        .expect("install folder")
        .join("assets")
        .join("machines");
    let file = machines.join("3axis-vmc.mch");
    assert!(!file.exists(), "the default machine starts missing");
    let missing = default_machine_checked().expect_err("missing file must refuse");
    assert!(
        missing.contains("3axis-vmc.mch") && missing.contains("cannot be read"),
        "{missing}"
    );
    assert!(
        missing.contains(machines.to_str().expect("UTF-8 install path")),
        "the reader must use the isolated folder: {missing}"
    );

    std::fs::write(&file, "{}").expect("write empty object");
    let empty = default_machine_checked().expect_err("empty object must refuse");
    assert!(
        empty.contains("3axis-vmc.mch") && empty.contains("no machine name"),
        "the reader must read the changed file: {empty}"
    );

    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/machines/3axis-vmc.mch");
    std::fs::copy(shipped, &file).expect("copy shipped default machine");
    let good = default_machine_checked().expect("the repaired file must be read without a restart");
    assert_eq!(good["name"], "VMC 3-AXIS");

    // Only the successfully parsed machine is kept, not a refused text.
    std::fs::write(&file, "{}").expect("break the file after the good read");
    let again = default_machine_checked().expect("the good parsed machine is kept");
    assert_eq!(again, good);
}
