//! The `linuxcnc.cps` script suite (#2303): the post binary on every recorded
//! edge-case stream (`recorded/streams/*.cnc`), its program or its named
//! refusal compared with the golden in `recorded/golden/linuxcnc/`. No app, no
//! toolpath generation: the inputs are recorded, and their generators keep
//! them current (`recorded_streams.rs`, app's `recorded_post_streams.rs`).
//!
//! Regenerate after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test script_linuxcnc`
//!
//! Gate map (M4.4): covers assets/posts/linuxcnc.cps, camcraft_post::post::engine,
//! camcraft_post::post::cnc, the post binary.

#[macro_use]
mod recorded;

post_suite!("linuxcnc.cps");

/// #2528: the machine-less case stops on the missing safe retract alone,
/// so the full-diameter depth is recorded on a machine that states one
/// (`vmc.mch`). The canned cycle goes to the point allowance, Z-12.803.
#[test]
fn drill_full_diameter_on_a_machine_posts_the_full_diameter_depth() {
    let mch = recorded::repo().join("assets/machines/vmc.mch");
    let nc = recorded::check_on_machine("linuxcnc.cps", "drill_full_diameter", &mch);
    let cycle = nc.lines().find(|l| l.contains("G73") || l.contains("G83") || l.contains("G81"));
    assert!(cycle.is_some_and(|l| l.contains("Z-12.803")), "the cycle depth is Z-12.803:\n{nc}");
}
