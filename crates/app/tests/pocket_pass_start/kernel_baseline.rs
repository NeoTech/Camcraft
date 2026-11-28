//! #4153: the app-level baseline of the 2D Pocket on `fig8.step` (the
//! #3354 fixture: EM-0800-4FL, Outside containment, the floor face),
//! recorded on the old planner of `side/path-kernel` for the switch-over
//! proof of the path kernel (#4023). The text is the CL dump with one
//! `; note:` line for each note and one `; warning:` line for each
//! verifier warning, the form of the #3750 baseline. Regenerate only for
//! an intended output change, with `WRITE_GOLDEN=1 cargo test --release
//! -p camcraft-app --test pocket_pass_start -- --ignored kernel_baseline`,
//! then read the diff in crates/app/tests/golden/ (files `kernel_pocket2d_fig8_*`).

use super::{PassStart, PocketStrategy, fig8_world, model, pocket_op};
use crate::common::{gen_to_cltext, write_golden};

/// Generates one case on a fresh world and compares it with its golden.
fn record(case: &str, strategy: PocketStrategy, start: PassStart) {
    let (world, mut h, name, stock) = fig8_world();
    let path = pocket_op(&world, &mut h, &name, stock.0.z, strategy, start, None);
    let e = model::find_by_name(&world, "P1").expect("op");
    let warnings = model::toolpath_of(&world, &e).expect("cached").warnings.clone();
    let mut text = gen_to_cltext(&path);
    for n in &path.notes {
        text.push_str(&format!("; note: {}\n", n.replace('\n', " ")));
    }
    for w in &warnings {
        text.push_str(&format!("; warning: {}\n", format!("{w:?}").replace('\n', " ")));
    }
    write_golden(&format!("kernel_pocket2d_fig8_{case}.cl"), &text);
}

/// #4153 pin: the 2D Pocket on fig8 for Waterline with Auto, Inside and
/// Outside, Offset rings with Outside, and Zigzag.
#[test]
#[ignore = "heavy lane: app heavy: pocket_pass_start (#4153: five fig8 generations run past the 60 s debug budget)"]
fn fig8_pocket_kernel_baseline() {
    let cases = [
        ("waterline_auto", PocketStrategy::Waterline, PassStart::Auto),
        ("waterline_inside", PocketStrategy::Waterline, PassStart::Inside),
        ("waterline_outside", PocketStrategy::Waterline, PassStart::Outside),
        ("rings_outside", PocketStrategy::OffsetRings, PassStart::Outside),
        ("zigzag", PocketStrategy::Zigzag, PassStart::Auto),
    ];
    for (case, strategy, start) in cases {
        record(case, strategy, start);
    }
}

/// Light twin of `fig8_pocket_kernel_baseline`: the Waterline Auto case
/// only (one generation in place of five), against the same golden.
#[test]
#[ignore = "light lane: the Waterline Auto case of the full test only (#4153); the full test is fig8_pocket_kernel_baseline"]
fn fig8_pocket_kernel_baseline_light() {
    record("waterline_auto", PocketStrategy::Waterline, PassStart::Auto);
}
