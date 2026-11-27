//! #2586: `camcraft-sim check` exits 2 when a check fails, and each
//! finding names its NC line: a travel past a stated stroke (#2585, in
//! program coordinates), a
//! machine-part collision on the generated model (#2584), and a rapid
//! through material in the simulated stock.
//!
//! Gate map: covers camcraft-simview src/check.
//!
//! Provenance: our own fixtures; `vmc-tree.mch` is our own machine file.

use super::{at, check, program, text, tmp, tree_mch, STOCK, TOOLS};

fn run(name: &str, extra: &str, stock: &str) -> (Option<i32>, String) {
    let nc = tmp(name, &program(extra));
    let (post, mch) = (at("assets/posts/fanuc.cps"), tree_mch());
    let o = check(&[&nc, "--post", &post, "--machine", &mch, "--stock", stock, "--tools", TOOLS, "--wcs", "G54=0,0,50"]);
    (o.status.code(), text(&o))
}

#[test]
fn check_names_the_line_of_a_travel_past_the_stroke() {
    // Line 12 lifts, line 13 rapids to X350: the X stroke is -300..300.
    let (code, out) = run("travel.nc", "G0 Z10.\nG0 X350.\n", STOCK);
    assert_eq!(code, Some(2), "{out}");
    assert!(
        out.contains("FAIL        machine limits: line 13: the move reaches 350.000 in X, the stroke is -300.000..300.000"),
        "{out}"
    );
    assert_eq!(out.lines().filter(|l| l.starts_with("FAIL ")).count(), 1, "only the travel fails:\n{out}");
    assert!(out.ends_with("result: FAILED\n"), "{out}");
}

#[test]
fn check_names_the_line_of_a_machine_part_collision() {
    // Line 12 feeds down to work Z-60: machine Z-10 with G54 at Z50, so
    // the spindle body (nose at machine Z0 is the table top) enters
    // the Y slide.
    let (code, out) = run("crash.nc", "G1 Z-60. F300.\n", "-40,-20,-80,40,20,0");
    assert_eq!(code, Some(2), "{out}");
    assert!(
        out.contains("FAIL        machine model: line 12: the spindle body of part head meets part y at machine X30.000 Y0.000 Z-"),
        "{out}"
    );
    assert!(out.contains("(judged on a generated model)"), "{out}");
    assert!(!out.contains("machine model: line 11"), "the cut at Z-2 is clear:\n{out}");
    // 60 mm deep with a 20 mm flute: the holder (40 mm) enters the stock.
    assert!(out.contains("FAIL        holder law: line 12: the holder of T1 meets material at X30.000 Y0.000 Z-"), "{out}");
}

#[test]
fn check_names_the_line_of_a_fixture_hit() {
    // The slot (line 11) ends at X30 with a 10 mm tool: its edge reaches
    // X35, into a clamp box that stands from X34.
    let nc = tmp("fixture.nc", &program(""));
    let (post, mch) = (at("assets/posts/fanuc.cps"), tree_mch());
    let o = check(&[
        &nc, "--post", &post, "--machine", &mch, "--stock", STOCK, "--tools", TOOLS, "--wcs", "G54=0,0,50", "--fixture",
        "34,-5,-10,40,5,5",
    ]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(2), "{out}");
    assert!(out.contains("FAIL        fixture law: line 11: the "), "{out}");
    assert!(out.contains("of T1 strikes a fixture box"), "{out}");
}

#[test]
fn check_names_the_line_of_a_rapid_through_material() {
    // Line 12 rapids down from the slot floor (Z-2) into uncut stock.
    let (code, out) = run("rapid.nc", "G0 Z-3.\n", STOCK);
    assert_eq!(code, Some(2), "{out}");
    assert!(out.contains("FAIL        stock: line 12: the rapid runs T1 through material at X30.000 Y0.000 Z-"), "{out}");
    assert!(!out.contains("stock: line 9"), "the rapid to Z2 is in air:\n{out}");
}
