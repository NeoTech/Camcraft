//! #3750 equivalence pins: the shared ring planner (`ring_set`,
//! `ring_links`, `ring_entry`) gives the 2D Offset, the 2D Pocket and the
//! 3-D level the exact output of the head before it. Each golden was
//! recorded on that head (origin/main 138a592a1) and must not move.
//! Regenerate only for an intended output change, with
//! `WRITE_GOLDEN=1 cargo test -p camcraft-motion --test pass_start_levels shared_planner`,
//! then read the diff in crates/motion/tests/golden/.

use super::{band, params, rect, region};
use camcraft_motion::cl::{Toolpath, text::to_text};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_motion::machining2d::{
    CornerRelief, PassStart, PocketParams, PocketStrategy, StartSide, level_ring_plan, pocket_started, pocket_started_on,
};

fn disc(cx: f64, r: f64) -> Vec<Point2> {
    (0..128)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / 128.0;
            Point2::new(cx + r * a.cos(), r * a.sin())
        })
        .collect()
}

/// The dog-bone part of #3741 (two discs of radius 12, a neck 8 mm wide)
/// in a stock box of 80 x 50, and the band a 2D Offset round it sweeps.
fn dog_bone() -> (Region, Region) {
    let part = region(vec![disc(-15.0, 12.0)])
        .union(&region(vec![disc(15.0, 12.0)]))
        .and_then(|p| p.union(&region(vec![rect(-15.0, -4.0, 15.0, 4.0)])))
        .expect("part");
    let stock = region(vec![rect(-40.0, -25.0, 40.0, 25.0)]);
    let band = stock.offset(super::R).and_then(|s| s.difference(&part)).expect("band");
    (stock, band)
}

/// Compares `text` with `tests/golden/<name>`, or writes it under
/// `WRITE_GOLDEN`.
fn golden(name: &str, text: &str) {
    let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name);
    if std::env::var_os("WRITE_GOLDEN").is_some() {
        std::fs::write(&file, text).expect("cannot write the golden file");
        return;
    }
    let want = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("cannot read golden {}: {e}", file.display()));
    assert!(want.replace("\r\n", "\n") == text, "golden {} moved: the shared ring planner changed the output", file.display());
}

/// The CL text of `path` with its notes, one `; note:` line each.
fn dump(path: &Toolpath) -> String {
    let mut s = to_text(path).expect("the path has a CL text form");
    for n in &path.notes {
        s.push_str(&format!("; note: {}\n", n.replace('\n', " ")));
    }
    s
}

fn offset_params(strategy: PocketStrategy, entry: Entry3D) -> PocketParams {
    let mut p = params();
    p.stepover = 2.0;
    p.strategy = strategy;
    p.entry = entry;
    p
}

/// #3750 pin: the 2D Offset round the dog-bone, for Auto (with and
/// without a stock footprint), Inside and Outside, for Waterline and
/// Offset rings, with a Helix and an Arc entry, posts the same CL as the
/// head before the shared planner.
#[test]
fn offset2d_rings_unchanged_by_the_shared_planner() {
    let (stock, band) = dog_bone();
    let cases: [(&str, PassStart, bool, PocketStrategy, Entry3D); 8] = [
        ("auto", PassStart::Auto, true, PocketStrategy::Waterline, Entry3D::Helix),
        ("auto_blind", PassStart::Auto, false, PocketStrategy::Waterline, Entry3D::Helix),
        ("inside", PassStart::Inside, true, PocketStrategy::Waterline, Entry3D::Helix),
        ("outside", PassStart::Outside, true, PocketStrategy::Waterline, Entry3D::Helix),
        ("inside_rings", PassStart::Inside, true, PocketStrategy::OffsetRings, Entry3D::Helix),
        ("outside_rings", PassStart::Outside, true, PocketStrategy::OffsetRings, Entry3D::Helix),
        ("inside_arc", PassStart::Inside, true, PocketStrategy::OffsetRings, Entry3D::Arc),
        ("outside_arc", PassStart::Outside, true, PocketStrategy::OffsetRings, Entry3D::Arc),
    ];
    for (name, start, known, strategy, entry) in cases {
        let side = StartSide { start, stock: known.then(|| stock.clone()), around_part: true };
        let p = offset_params(strategy, entry);
        let path = pocket_started_on(&band, &p, &[], &CornerRelief::default(), &side).expect("plan");
        golden(&format!("offset2d_dogbone_{name}.cl"), &dump(&path));
    }
}

/// #3750 pin: the 2D Pocket on the split band of #3316 (the ring set of
/// the area's own walls), for Auto, Inside and Outside, posts the same CL
/// as the head before the shared planner.
#[test]
fn pocket_rings_unchanged_by_the_shared_planner() {
    let (_, _, reach) = band();
    for (name, start) in [("auto", PassStart::Auto), ("inside", PassStart::Inside), ("outside", PassStart::Outside)] {
        let path = pocket_started(&reach, &params(), &[], &CornerRelief::default(), start).expect("plan");
        golden(&format!("pocket_band_{name}.cl"), &dump(&path));
    }
}

/// #3750 pin: the 3-D rough's level ring plan (`level_ring_plan`) on the
/// same band, for each start side, gives the same passes (1 um) and notes.
#[test]
fn level_rings_unchanged_by_the_shared_planner() {
    let (stock, _, reach) = band();
    let mut s = String::new();
    for start in [PassStart::Auto, PassStart::Inside, PassStart::Outside] {
        let side = StartSide { start, stock: Some(stock.clone()), around_part: false };
        let (passes, notes) =
            level_ring_plan(&reach, super::R, super::STEP, true, PocketStrategy::Waterline, &side).expect("level plan");
        s.push_str(&format!("{start:?} passes={}\n", passes.len()));
        for pass in &passes {
            let pts: Vec<String> = pass.iter().map(|q| format!("{:.3},{:.3}", q.x, q.y)).collect();
            s.push_str(&pts.join(" "));
            s.push('\n');
        }
        for n in notes {
            s.push_str(&format!("note: {}\n", n.replace('\n', " ")));
        }
    }
    golden("level_rings_band.txt", &s);
}
