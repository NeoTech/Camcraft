//! Pin the original Hurco post metadata required by the post naming law.
//! Provenance: project assets/posts/AGENTS.md naming requirements.
//! Gate map: covers assets/posts/hurco-winmax.cps.

#[test]
fn hurco_post_vendor_is_camcraft() {
    let script = include_str!("../../../assets/posts/hurco-winmax.cps");
    assert!(script.lines().any(|line| line.trim() == r#"vendor = "Camcraft";"#));
}
