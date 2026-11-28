//! Regression pins for the post verdict of the headless check.
//! Provenance: our own report fixture for review #4471.

use super::{inputs::PostManifest, post_checks, program::Program, report::Report};

#[test]
fn a_post_failure_never_prints_pass_after_an_earlier_failure() {
    let program = Program {
        sections: Vec::new(),
        inch: true,
        mm: false,
        first_line: None,
    };
    let post = PostManifest {
        name: "mill-mm.cps".to_owned(),
        extension: "nc".to_owned(),
        serves: None,
        units: Some(vec!["mm".to_owned()]),
        work_offset_max: 6,
        dialect: None,
    };
    let mut report = Report::default();
    report.fail("earlier check", "an earlier check failed");
    post_checks(&program, Some(&post), None, &mut report);
    let out = report.text();
    assert!(
        out.contains("FAIL        post: the program states G20 (inch)"),
        "{out}"
    );
    assert!(!out.contains("PASS        post:"), "{out}");
}
