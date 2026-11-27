//! #2586 "Done means": `camcraft-sim check` runs on a 3-axis program
//! posted with `fanuc.cps` for `3axis-vmc.mch` (the example catalog's
//! `2d/first-part`, 100 x 60 x 20 stock, top at Z20). That machine file
//! states no stroke and no part tree, and the program returns home with
//! G28 in an offset with no `--wcs`: each of these is named as not
//! judged (exit 3), never read as a pass. The stock and holder checks
//! run on the whole program and find nothing.
//!
//! Gate map: covers camcraft-simview src/check.
//!
//! Provenance: the recorded catalog program; our own assertions.

use super::{at, check, text};

#[test]
fn check_runs_on_the_fanuc_catalog_program_for_the_default_vmc() {
    let (nc, post, mch) =
        (at("examples/catalog/2d/first-part/first-part.nc"), at("assets/posts/fanuc.cps"), at("assets/machines/3axis-vmc.mch"));
    let tools = "T1=6:flute=30:holder=30x40,T2=6:flute=20:holder=30x40,T3=10:flute=25:holder=40x40";
    // A 1 mm cell keeps the debug run near 20 s (the default 0.375 mm
    // cell takes about 50 s); the checks and their verdict words agree.
    let o = check(&[&nc, "--post", &post, "--machine", &mch, "--stock", "-50,-30,0,50,30,20", "--tools", tools, "--cell", "1"]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(3), "{out}");
    assert!(!out.contains("FAIL"), "{out}");
    assert!(out.contains("note        post: fanuc.cps (extension nc, programUnits mm inch, work offsets 1..6)"), "{out}");
    assert!(out.contains("PASS        post: "), "{out}");
    assert!(out.contains("PASS        stock: "), "{out}");
    assert!(out.contains("PASS        holder law: "), "{out}");
    assert!(out.contains("NOT JUDGED  machine limits: 3axis-vmc.mch states no stroke for X, Y, Z"), "{out}");
    assert!(out.contains("NOT JUDGED  machine model: 3axis-vmc.mch states no part tree"), "{out}");
    assert!(out.contains("NOT JUDGED  reference moves: lines "), "the closing G28 moves are named:\n{out}");
}
