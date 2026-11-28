//! #2586: each exit code of `camcraft-sim check` - 0 when each check ran
//! and passed, 1 for bad input (never a panic), 3 when a check did not
//! run, with each such check named. Exit 2 is pinned in `findings`.
//!
//! Gate map: covers camcraft-simview src/check.
//!
//! Provenance: our own fixtures.

use super::{at, check, program, text, tmp, tree_mch, STOCK, TOOLS};

#[test]
fn check_exits_0_when_each_check_ran_and_passed() {
    let nc = tmp("ok.nc", &program(""));
    let (post, mch) = (at("assets/posts/fanuc.cps"), tree_mch());
    let o = check(&[&nc, "--post", &post, "--machine", &mch, "--stock", STOCK, "--tools", TOOLS, "--wcs", "G54=0,0,50"]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(0), "{out}");
    for check in ["post", "stock", "holder law", "fixture law", "machine limits", "machine model"] {
        assert!(out.contains(&format!("PASS        {check}: ")), "{check} must pass:\n{out}");
    }
    assert!(!out.contains("NOT JUDGED") && !out.contains("FAIL"), "{out}");
    assert!(out.contains("judged on a generated model"), "{out}");
    assert!(out.ends_with("result: ok (each check ran and passed)\n"), "{out}");
}

#[test]
fn check_exits_3_and_names_the_missing_machine_file() {
    let nc = tmp("no-mch.nc", &program(""));
    let post = at("assets/posts/fanuc.cps");
    let o = check(&[&nc, "--post", &post, "--stock", STOCK, "--tools", TOOLS, "--wcs", "G54=0,0,50"]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(3), "{out}");
    assert!(out.contains("NOT JUDGED  machine limits: no machine file is given (--machine)"), "{out}");
    assert!(out.contains("NOT JUDGED  machine model: no machine file is given (--machine)"), "{out}");
    assert!(out.contains("PASS        stock: "), "the stock check still runs:\n{out}");
}

#[test]
fn check_exits_3_and_names_the_missing_work_offset() {
    let nc = tmp("no-wcs.nc", &program(""));
    let (post, mch) = (at("assets/posts/fanuc.cps"), tree_mch());
    let o = check(&[&nc, "--post", &post, "--machine", &mch, "--stock", STOCK, "--tools", TOOLS]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(3), "{out}");
    assert!(out.contains("NOT JUDGED  machine model: lines 8-12: no machine position is given for G54 (--wcs)"), "{out}");
    // The strokes are in program coordinates: the limits need no offset.
    assert!(out.contains("PASS        machine limits: "), "{out}");
}

#[test]
fn check_exits_3_when_the_holder_stack_is_not_stated() {
    let nc = tmp("no-holder.nc", &program(""));
    let (post, mch) = (at("assets/posts/fanuc.cps"), tree_mch());
    let o = check(&[&nc, "--post", &post, "--machine", &mch, "--stock", STOCK, "--tools", "T1=10", "--wcs", "G54=0,0,50"]);
    let out = text(&o);
    assert_eq!(o.status.code(), Some(3), "{out}");
    assert!(out.contains("NOT JUDGED  holder law: T1: the T map states no flute length and holder"), "{out}");
}

#[test]
fn check_refuses_bad_input_with_exit_1() {
    let good = tmp("good.nc", &program(""));
    let rotary = tmp("rotary.nc", &program("G0 A10.\n"));
    let (post, lathe) = (at("assets/posts/fanuc.cps"), at("assets/machines/lathe.mch"));
    let base = |nc: &str| vec![nc.to_owned(), "--post".into(), post.clone(), "--stock".into(), STOCK.into(), "--tools".into(), TOOLS.into()];
    let with = |nc: &str, more: &[&str]| {
        let mut v = base(nc);
        v.extend(more.iter().map(|s| (*s).to_owned()));
        v
    };
    let cases: Vec<(Vec<String>, &str)> = vec![
        (base("no-such-file.nc"), "cannot read"),
        (with(&rotary, &[]), "rotary axis word"),
        (with(&good, &["--machine", &lathe]), "3-axis milling programs only"),
        (with(&good, &["--machine", "no-such.mch"]), "cannot read machine"),
        (with(&good, &["--wcs", "G60=0,0,0"]), "--wcs names"),
        (with(&good, &["--dialect", "nosuch"]), "--dialect"),
        (vec![good.clone(), "--post".into(), post.clone(), "--stock".into(), STOCK.into(), "--tools".into(), "T2=6".into()], "the program uses T1, and --tools does not state it"),
        (vec![good.clone(), "--stock".into(), STOCK.into(), "--tools".into(), TOOLS.into()], "name the post"),
        (vec![good.clone(), "--dialect".into(), "fanuc".into(), "--stock".into(), "1,1,1,0,0,0".into(), "--tools".into(), TOOLS.into()], "x0 < x1"),
        (vec![good.clone(), "--dialect".into(), "fanuc".into(), "--stock".into(), STOCK.into(), "--tools".into(), "tools.camtools".into()], ".camtools"),
        (vec![good.clone(), "--dialect".into(), "fanuc".into(), "--stock".into(), STOCK.into()], "--tools is required"),
        (vec![], "expected a G-code file"),
    ];
    for (args, words) in cases {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let o = check(&refs);
        let err = String::from_utf8_lossy(&o.stderr);
        assert_eq!(o.status.code(), Some(1), "{args:?}: {}", text(&o));
        assert!(err.starts_with("camcraft-sim: ") && err.contains(words) && !err.contains("panicked"), "{args:?}: {err}");
    }
}
