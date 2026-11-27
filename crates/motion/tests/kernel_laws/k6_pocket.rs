//! #3752 (K6 of the path kernel): the 2D Pocket recipe on the kernel
//! (`machining2d::pocket_on_kernel`), with the pins of the issue:
//! - Inside, Outside and Auto (Waterline) cut the same rings;
//! - the rings of a square pocket do not lift, and each lift on a pocket
//!   with an island and round a part has a named reason;
//! - no ramp entry ends in a dead end: each ramp descends along its own
//!   first cut and ends at its start;
//! - the verifier finds no rapid in stock, no plunge into stock, no gouge;
//! - a closed pocket with no pre-drilled hole and no helix or ramp is
//!   refused by name;
//! - the old planner against the kernel with the law measures (#4154).

use super::common::law_measure::{EntryKind as MeasuredEntry, EntryStart, Readings, measure};
use super::fixture::{R, STEP, TOP, band, parts, planned_deg, pocket_params, region, side, spec_round, stock};
use camcraft_motion::cl::Toolpath;
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Point2, Region};
use camcraft_motion::machining2d::{
    CornerRelief, KERNEL_POCKET_NOTE, KERNEL_POCKET_ZIGZAG, PassStart, PocketParams, PocketStrategy, StartSide, pocket_kernel_plans, pocket_on_kernel,
    pocket_started_on,
};
use camcraft_motion::pathkernel::entry::{EntryKind, NO_SAFE_ENTRY};
use camcraft_motion::pathkernel::guide::Curve;
use camcraft_motion::pathkernel::plan::{Plan, Step};
use camcraft_motion::verify::{Mode, ToolGeom, VerifyContext, Warning, verify};

const SIDES: [(&str, PassStart); 3] = [("Auto", PassStart::Auto), ("Inside", PassStart::Inside), ("Outside", PassStart::Outside)];

fn circle(cx: f64, cy: f64, r: f64, n: u32) -> Vec<Point2> {
    (0..n)
        .map(|k| {
            let a = f64::from(k) * std::f64::consts::TAU / f64::from(n);
            Point2::new(cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

fn square(s: f64) -> Vec<Point2> {
    let h = 0.5 * s;
    vec![Point2::new(-h, -h), Point2::new(h, -h), Point2::new(h, h), Point2::new(-h, h)]
}

/// A closed 40 mm square pocket.
fn square_pocket() -> Region {
    region(vec![square(40.0)])
}

/// The same pocket with a round island of radius 6 in its middle.
fn island_pocket() -> Region {
    let mut hole = circle(0.0, 0.0, 6.0, 96);
    hole.reverse();
    region(vec![square(40.0), hole])
}

/// A closed pocket: no stock is known round it (the walls are the part).
fn closed(start: PassStart) -> StartSide {
    StartSide { start, stock: None, around_part: false }
}

fn waterline() -> PocketParams {
    pocket_params(PocketStrategy::Waterline)
}

/// The cut curves of the plans, in order.
fn cuts(plans: &[&Plan]) -> Vec<Curve> {
    plans
        .iter()
        .flat_map(|p| p.steps().iter())
        .filter_map(|s| if let Step::Cut(c) = s { Some(c.curve().clone()) } else { None })
        .collect()
}

/// The plans of a kernel pocket, its notes, its lift count and the lifts by reason.
type Planned = (Vec<Plan>, Vec<String>, u32, Vec<(&'static str, u32)>);

fn plans_of(area: &Region, p: &PocketParams, s: &StartSide) -> Planned {
    let k = pocket_kernel_plans(area, p, &[], s).unwrap_or_else(|e| panic!("the kernel plans the pocket: {e}"));
    let lifts = k.levels.iter().map(|(l, _)| l.tally.lifts()).sum();
    let mut why: Vec<(&'static str, u32)> = Vec::new();
    for (l, _) in &k.levels {
        for (r, n) in l.tally.by_reason() {
            why.push((r, *n));
        }
    }
    (k.levels.iter().map(|(l, _)| l.plan.clone()).collect(), k.notes, lifts, why)
}

/// True when the two multisets of curves are the same cuts: each curve of
/// `a` has its own partner in `b` that is the same ring, restarted or
/// reversed (`Curve::same_cut`, 1 um).
fn same_rings(a: &[Curve], b: &[Curve]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut used = vec![false; b.len()];
    a.iter().all(|ca| match b.iter().enumerate().find(|(j, cb)| !used[*j] && ca.same_cut(cb)) {
        Some((j, _)) => {
            used[j] = true;
            true
        }
        None => false,
    })
}

#[test]
fn pocket_inside_and_outside_cut_the_same_rings() {
    for (name, area) in [("square pocket", square_pocket()), ("pocket with an island", island_pocket())] {
        let mut sets = Vec::new();
        for (sname, start) in SIDES {
            let (plans, _, _, _) = plans_of(&area, &waterline(), &closed(start));
            let c = cuts(&plans.iter().collect::<Vec<_>>());
            eprintln!("{name} {sname}: {} rings", c.len());
            sets.push((sname, c));
        }
        let (_, auto) = &sets[0];
        assert!(!auto.is_empty(), "{name}: the kernel cuts rings");
        for (sname, c) in &sets[1..] {
            assert!(same_rings(auto, c), "{name}: {sname} cuts other rings than Auto");
        }
    }
}

#[test]
fn pocket_rings_do_not_lift() {
    let (_, _, lifts, why) = plans_of(&square_pocket(), &waterline(), &closed(PassStart::Auto));
    assert_eq!(lifts, 0, "the square pocket links each ring at depth: {why:?}");
    for (name, area, s) in [
        ("pocket with an island", island_pocket(), closed(PassStart::Auto)),
        ("pocket with an island, Outside", island_pocket(), closed(PassStart::Outside)),
    ] {
        let (_, _, lifts, why) = plans_of(&area, &waterline(), &s);
        let named: u32 = why.iter().map(|(_, n)| n).sum();
        eprintln!("{name}: {lifts} lift(s) {why:?}");
        assert_eq!(named, lifts, "{name}: each lift has a named reason");
        assert!(why.iter().all(|(r, _)| !r.trim().is_empty()), "{name}: each reason is named");
    }
}

#[test]
fn pocket_has_no_dead_end_ramp() {
    let mut p = waterline();
    p.entry = Entry3D::Ramp;
    let mut ramps = 0;
    for (name, area) in [("square pocket", square_pocket()), ("pocket with an island", island_pocket())] {
        for (_, start) in SIDES {
            let (plans, _, _, _) = plans_of(&area, &p, &closed(start));
            for plan in &plans {
                let steps = plan.steps();
                for (i, s) in steps.iter().enumerate() {
                    let Step::Entry(e) = s else { continue };
                    if e.kind() != EntryKind::Ramp {
                        continue;
                    }
                    ramps += 1;
                    let Some(Step::Cut(c)) = steps.get(i + 1) else { panic!("{name}: a ramp entry is not followed by its cut") };
                    let mut line: Vec<Point2> = c.curve().pts().iter().map(|q| q.at()).collect();
                    if c.curve().closed() {
                        line.push(line[0]);
                    }
                    let on = |q: Point2| line.windows(2).any(|w| seg(q, w[0], w[1]) <= 1e-3);
                    assert!(e.descent().iter().all(|q| on(q.at())), "{name}: the ramp leaves its first cut (a dead end)");
                    let end = e.descent().last().map(|q| q.at()).expect("descent");
                    assert!(seg(end, line[0], line[0]) <= 1e-6, "{name}: the ramp ends at the start of its cut");
                }
            }
        }
    }
    assert!(ramps > 0, "the closed pockets enter by the ramp that the user selected");
}

fn seg(p: Point2, a: Point2, b: Point2) -> f64 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let len2 = dx * dx + dy * dy;
    let t = if len2 > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / len2).clamp(0.0, 1.0) } else { 0.0 };
    (p.x - (a.x + t * dx)).hypot(p.y - (a.y + t * dy))
}

#[test]
fn a_closed_pocket_without_a_safe_entry_is_refused_by_name() {
    let mut p = waterline();
    p.entry = Entry3D::Plunge;
    let e = pocket_on_kernel(&square_pocket(), &p, &[], &closed(PassStart::Auto)).expect_err("refused, never a plunge into solid");
    assert!(e.to_string().contains(NO_SAFE_ENTRY), "{e}");
    assert!(e.to_string().contains("pre-drill a hole"), "the refusal says how to fix it: {e}");
}

#[test]
fn the_zigzag_pocket_is_refused_by_name_on_the_kernel() {
    for strategy in [PocketStrategy::Zigzag, PocketStrategy::Auto] {
        let e = pocket_on_kernel(&square_pocket(), &pocket_params(strategy), &[], &closed(PassStart::Auto)).expect_err("refused by name");
        assert!(e.to_string().contains(KERNEL_POCKET_ZIGZAG), "{strategy:?}: {e}");
    }
}

/// The verifier on a kernel pocket: no rapid in stock or in the part, no
/// plunge into stock, no gouge; each finding is printed, not silenced.
fn verified(name: &str, path: &Toolpath, area: &Region, st: &Region) {
    let tool = ToolGeom { radius: R, nose_radius: 0.0, flute_len: 30.0, shank_radius: R, shank_len: 10.0, holder_radius: 15.0, holder_len: 30.0 };
    let cx = VerifyContext {
        region: area,
        stock: Some(st),
        tool: &tool,
        mode: Mode::Area,
        stock_top_z: TOP,
        clearance_z: 10.0,
        gouge_tol: 0.01,
        z_tol: 0.05,
        ..VerifyContext::default()
    };
    let warns = verify(path, &cx);
    eprintln!("{name}: {} moves, verifier {warns:?}", path.moves.len());
    let bad: Vec<&Warning> = warns
        .iter()
        .filter(|w| matches!(w, Warning::RapidInStock { .. } | Warning::RapidInPart { .. } | Warning::PlungeIntoStock { .. } | Warning::Gouge { .. }))
        .collect();
    assert!(bad.is_empty(), "{name}: the verifier finds {bad:?}");
}

/// The count of the verifier's `Crossing` findings on `path`.
fn crossings(path: &Toolpath, area: &Region, st: &Region) -> usize {
    let tool = ToolGeom { radius: R, nose_radius: 0.0, flute_len: 30.0, shank_radius: R, shank_len: 10.0, holder_radius: 15.0, holder_len: 30.0 };
    let cx = VerifyContext { region: area, stock: Some(st), tool: &tool, mode: Mode::Area, stock_top_z: TOP, clearance_z: 10.0, gouge_tol: 0.01, z_tol: 0.05, ..VerifyContext::default() };
    verify(path, &cx).iter().filter(|w| matches!(w, Warning::Crossing { .. })).count()
}

#[test]
fn the_verifier_passes_each_kernel_pocket() {
    let strategies = [PocketStrategy::Waterline, PocketStrategy::OffsetRings];
    for (name, area) in [("square pocket", square_pocket()), ("pocket with an island", island_pocket())] {
        for strategy in strategies {
            for (sname, start) in SIDES {
                let path = pocket_on_kernel(&area, &pocket_params(strategy), &[], &closed(start))
                    .unwrap_or_else(|e| panic!("{name} {strategy:?} {sname}: {e}"));
                assert!(path.notes.iter().any(|n| n == KERNEL_POCKET_NOTE), "the status line names the kernel");
                let st = region(vec![square(40.0)]);
                verified(&format!("{name} {strategy:?} {sname}"), &path, &area, &st);
                let old = pocket_started_on(&area, &pocket_params(strategy), &[], &CornerRelief::default(), &closed(start));
                if let Ok(old) = old {
                    let n = |t: &Toolpath| crossings(t, &area, &st);
                    let (o, k) = (n(&old), n(&path));
                    eprintln!("  crossings: old {o} kernel {k}");
                    assert!(k <= o, "{name} {strategy:?} {sname}: the kernel crosses more feeds than the old planner");
                }
            }
        }
    }
    for (pname, part) in parts() {
        for (sname, start) in SIDES {
            let path = pocket_on_kernel(&band(&part), &waterline(), &[], &side(start)).expect("planned");
            let area = stock().offset(4.0 * R).unwrap().difference(&part).unwrap();
            verified(&format!("{pname} {sname}"), &path, &area, &stock());
        }
    }
}

/// The law readings of the kernel's 2D Pocket (Waterline) round the disc
/// and the dog bone, the cases of `laws_pockets::pocket_2d_readings_of_the_old_planner`
/// (old: lifts 18 / 26, none named, 18 / 26 entries into solid). Each
/// reading is pinned as measured; the open items are named in the PR and
/// in `docs/decisions/3752-kernel-pocket-recipe.md`.
#[test]
fn pocket_2d_readings_of_the_kernel() {
    const TABLE: &[super::fixture::Row] = &[
        ("disc Inside",
         "lifts 4 named 4 re 4 | cross 1 leave 6 | entry air 2 hole 0 col 2 solid 1 (plunge 0 ramp 1 helix 0 rapid 0) | over 1205 worst 173 full 0 | fallback 0",
         "link L4 L6"),
        ("disc Outside",
         "lifts 5 named 5 re 5 | cross 0 leave 10 | entry air 2 hole 0 col 0 solid 4 (plunge 0 ramp 4 helix 0 rapid 0) | over 362 worst 180 full 172 | fallback 0",
         "link L4 L6"),
        ("dog bone Inside",
         "lifts 10 named 10 re 10 | cross 1 leave 6 | entry air 2 hole 0 col 8 solid 1 (plunge 0 ramp 1 helix 0 rapid 0) | over 812 worst 173 full 0 | fallback 0",
         "link L4 L6"),
        ("dog bone Outside",
         "lifts 9 named 9 re 9 | cross 0 leave 2 | entry air 2 hole 0 col 0 solid 8 (plunge 0 ramp 8 helix 0 rapid 0) | over 310 worst 180 full 103 | fallback 0",
         "link L4 L6"),
    ];
    let mut cases = Vec::new();
    for (pname, part) in parts() {
        for (sname, start) in [("Inside", PassStart::Inside), ("Outside", PassStart::Outside)] {
            let path = pocket_on_kernel(&band(&part), &waterline(), &[], &side(start)).expect("planned");
            cases.push((format!("{pname} {sname}"), path, spec_round(&part, planned_deg(STEP, R))));
        }
    }
    for (n, path, spec) in &cases {
        let r: Readings = measure(path, spec).unwrap();
        eprintln!("{n}: {} | breaks {}", r.row(), r.breaks(false));
        assert_eq!(r.named_lifts, r.lifts.count, "{n}: each lift is named");
        for e in &r.entries {
            let proved = e.start != EntryStart::Solid || matches!(e.kind, MeasuredEntry::Helix | MeasuredEntry::Ramp);
            assert!(proved, "{n}: a plunge into solid without a proved entry: {e:?}");
        }
    }
    super::fixture::pin("2D Pocket (kernel)", false, TABLE, cases);
}
