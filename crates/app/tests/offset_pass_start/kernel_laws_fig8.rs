//! #4154 (K7a of the path kernel, #4027): the readings of the OLD planners
//! on `fig8.step` with the law measures of `camcraft-motion`'s test
//! helpers (`motion/tests/common/law_measure/`, included by `#[path]`).
//! The kinds are the area kinds that apply to the part: 2D Pocket, 2D
//! Adaptive, 2D Offset and 3D Pocket (Inside and Outside), and the 2D
//! Contour of the outer wall. One level 5 mm over the floor, EM-0800-4FL,
//! a step of 3 mm. The tables pin today's readings; the third column
//! names each law that a reading breaks today.
//! Provenance: our own design record (#4024, the switch-over proof); no
//! vendor material consulted.

use super::area_support::*;
use super::motion_common::law_measure::{EntryKind, Spec, measure};
use super::{EM8, R8, fig8_world, horizontal_face};
use camcraft_app::geo2d::{Loop, Point2, Region};
use camcraft_app::machining2d::PassStart;
use camcraft_app::operation::HeightReference;

const STEP: f64 = 3.0;
const WALL: f64 = 1.0;

type Row = (&'static str, &'static str, &'static str);

fn planned_deg(s: f64) -> f64 {
    (1.0 - s / R8).clamp(-1.0, 1.0).acos().to_degrees()
}

fn rect(lo: Point3, hi: Point3) -> Region {
    let pts = vec![
        Point2::new(lo.x, lo.y),
        Point2::new(hi.x, lo.y),
        Point2::new(hi.x, hi.y),
        Point2::new(lo.x, hi.y),
    ];
    Region::new(vec![Loop::new(pts).expect("loop")]).expect("stock")
}

/// The fig8 world, the part's section at the cut level, and the stock.
struct Fig8 {
    world: World,
    name: String,
    lo: Point3,
    hi: Point3,
    part: Region,
}

fn fig8() -> Fig8 {
    let (world, _h, name, (lo, hi)) = fig8_world();
    let e = model::find_by_name(&world, &name).expect("part");
    let facets = model::world_facets(&world, &e).expect("facets");
    let part = camcraft_motion::geometry::slice::slice(&facets, lo.z + 5.1).expect("section");
    Fig8 { world, name, lo, hi, part }
}

impl Fig8 {
    fn level(&self) -> f64 {
        self.lo.z + 5.0
    }

    /// The spec of a clearing round the part in the stock box.
    fn spec(&self) -> Spec {
        let stock = rect(self.lo, self.hi);
        let mut s = Spec::new(R8, stock.clone(), self.hi.z, planned_deg(STEP));
        s.reach = Some(
            stock
                .offset(R8)
                .and_then(|o| o.difference(&self.part.offset(R8).expect("grown part")))
                .expect("reach"),
        );
        s.part = Some(self.part.clone());
        s.asked_entry = Some(EntryKind::Helix);
        s
    }

    /// Generate the operation `ty` on the floor face, with the heights of
    /// one level, a helix entry and `start`.
    fn run(&self, ty: OpType, start: PassStart, area: AreaControls) -> Toolpath {
        let (top, bottom) = (
            Height { reference: HeightReference::TopOfStock, offset: 0.0 },
            Height::fixed(self.level()),
        );
        let mut p = operation::retype_params(&pocket(self.hi.z, self.level()), ty.ordinal());
        match &mut p {
            OpParams::Pocket { top_z, bottom_z, stepdown, stepover, entry, pass_start, .. }
            | OpParams::Offset2D { top_z, bottom_z, stepdown, stepover, entry, pass_start, .. }
            | OpParams::Adaptive2D { top_z, bottom_z, stepdown, stepover, entry, pass_start, .. } => {
                (*top_z, *bottom_z, *stepdown, *stepover, *entry, *pass_start) =
                    (top, bottom, 20.0, STEP, Entry3D::Helix, start);
            }
            OpParams::Pocket3D { top_z, bottom_z, stepdown, stepover, entry, pass_start, .. } => {
                (*top_z, *bottom_z, *stepdown, *stepover, *entry, *pass_start) =
                    (top, bottom, 20.0, STEP, Entry3D::Helix, start);
            }
            OpParams::Contour { top_z, bottom_z, stepdown, entry, side, .. } => {
                (*top_z, *bottom_z, *stepdown, *entry, *side) = (top, bottom, 20.0, Entry3D::Helix, Side::Outside);
            }
            other => panic!("not an area kind: {other:?}"),
        }
        let src = if ty == OpType::Pocket3D {
            SrcRef::Entity(self.name.clone())
        } else {
            sel(&self.name, horizontal_face(&self.world, &self.name, self.lo.z))
        };
        let mut o = op(src, p, area);
        o.tool = EM8.into();
        operation::generate(&self.world, &o, "").map(|(path, _)| path).unwrap_or_else(|e| panic!("{ty:?} {start:?}: {e}"))
    }
}

const SIDES: [(&str, PassStart); 2] = [("Inside", PassStart::Inside), ("Outside", PassStart::Outside)];

/// Measure each case and compare it with the pinned table; on a
/// difference, list the whole table as read.
fn pin(kind: &str, table: &[Row], cases: Vec<(String, Toolpath, Spec)>) {
    let got: Vec<(String, String, String)> = cases
        .iter()
        .map(|(n, path, spec)| {
            let r = measure(path, spec).unwrap_or_else(|e| panic!("{kind} {n}: {e}"));
            (n.clone(), r.row(), r.breaks(false))
        })
        .collect();
    let want: Vec<(String, String, String)> =
        table.iter().map(|(n, r, b)| ((*n).to_string(), (*r).to_string(), (*b).to_string())).collect();
    if got != want {
        let text: String = got.iter().map(|(n, r, b)| format!("    (\"{n}\",\n     \"{r}\",\n     \"{b}\"),\n")).collect();
        panic!("{kind}: the readings of the old planner differ from the pinned table:\n{text}");
    }
}

/// The cases of `ty` on the first `n` start sides, against the first `n`
/// rows of `table` (a light twin reads a reduced set).
fn sided(kind: &str, ty: OpType, table: &[Row], n: usize) {
    let f = fig8();
    let area = AreaControls { containment: Some(Containment::Outside), ..Default::default() };
    let cases = SIDES
        .iter()
        .take(n)
        .map(|(s, start)| (format!("fig8 {s}"), f.run(ty, *start, area.clone()), f.spec()))
        .collect();
    pin(kind, table.get(..n).unwrap_or(table), cases);
}

#[test]
fn fig8_pocket_2d_readings_of_the_old_planner() {
    const TABLE: &[Row] = &[
        ("fig8 Inside",
         "lifts 70 named 0 re 70 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 70 (plunge 1 ramp 0 helix 69 rapid 0) | over 82 worst 180 full 2 | fallback 0",
         "link L4 L5 L6"),
        ("fig8 Outside",
         "lifts 70 named 0 re 70 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 70 (plunge 1 ramp 0 helix 69 rapid 0) | over 82 worst 180 full 2 | fallback 0",
         "link L4 L5 L6"),
    ];
    sided("2D Pocket", OpType::Pocket, TABLE, 2);
}

const FIG8_ADAPTIVE_2D_READINGS_OF_THE_OLD_PLANNER: &[Row] = &[
    ("fig8 Inside",
     "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 148 worst 173 full 0 | fallback 1",
     "L6"),
    ("fig8 Outside",
     "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 1 hole 0 col 0 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 188 worst 175 full 0 | fallback 1",
     "L6"),
];

#[test]
#[ignore = "heavy lane (#4154 fig8 Inside and Outside): two generations run past the 60 s debug budget (about 218 s) - run with --release -- --ignored"]
fn fig8_adaptive_2d_readings_of_the_old_planner() {
    sided("2D Adaptive", OpType::Adaptive2D, FIG8_ADAPTIVE_2D_READINGS_OF_THE_OLD_PLANNER, 2);
}

/// Light twin of `fig8_adaptive_2d_readings_of_the_old_planner`: the Inside case only (one generation).
#[test]
#[ignore = "light lane: the Inside case of the full test only (#4154); the full test is fig8_adaptive_2d_readings_of_the_old_planner"]
fn fig8_adaptive_2d_readings_of_the_old_planner_light() {
    sided("2D Adaptive", OpType::Adaptive2D, FIG8_ADAPTIVE_2D_READINGS_OF_THE_OLD_PLANNER, 1);
}

const FIG8_OFFSET_2D_READINGS_OF_THE_OLD_PLANNER: &[Row] = &[
    ("fig8 Inside",
     "lifts 3 named 3 re 3 | cross 0 leave 0 | entry air 0 hole 0 col 3 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 413 worst 173 full 0 | fallback 0",
     "L6"),
    ("fig8 Outside",
     "lifts 7 named 7 re 7 | cross 0 leave 0 | entry air 5 hole 0 col 3 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 875 worst 180 full 2 | fallback 0",
     "L6"),
];

#[test]
#[ignore = "heavy lane (#4154 fig8 Inside and Outside): two generations run past the 60 s debug budget (about 86 s) - run with --release -- --ignored"]
fn fig8_offset_2d_readings_of_the_old_planner() {
    sided("2D Offset", OpType::Offset2D, FIG8_OFFSET_2D_READINGS_OF_THE_OLD_PLANNER, 2);
}

/// Light twin of `fig8_offset_2d_readings_of_the_old_planner`: the Inside case only (one generation).
#[test]
#[ignore = "light lane: the Inside case of the full test only (#4154); the full test is fig8_offset_2d_readings_of_the_old_planner"]
fn fig8_offset_2d_readings_of_the_old_planner_light() {
    sided("2D Offset", OpType::Offset2D, FIG8_OFFSET_2D_READINGS_OF_THE_OLD_PLANNER, 1);
}

const FIG8_POCKET_3D: &[Row] = &[
    ("fig8 Inside",
     "lifts 98 named 0 re 98 | cross 0 leave 68 | entry air 0 hole 0 col 2 solid 98 (plunge 0 ramp 7 helix 91 rapid 0) | over 123 worst 180 full 5 | fallback 1",
     "link L6"),
    ("fig8 Outside",
     "lifts 98 named 0 re 98 | cross 0 leave 68 | entry air 0 hole 0 col 2 solid 98 (plunge 0 ramp 7 helix 91 rapid 0) | over 123 worst 180 full 5 | fallback 1",
     "link L6"),
];

/// The 3D Pocket of the whole part on the first `n` start sides.
fn pocket_3d(n: usize) {
    let f = fig8();
    let cases = SIDES
        .iter()
        .take(n)
        .map(|(s, start)| (format!("fig8 {s}"), f.run(OpType::Pocket3D, *start, AreaControls::default()), f.spec()))
        .collect();
    pin("3D Pocket", FIG8_POCKET_3D.get(..n).unwrap_or(FIG8_POCKET_3D), cases);
}

#[test]
#[ignore = "heavy lane (#4154 fig8 Inside and Outside): two generations run past the 60 s debug budget (about 211 s) - run with --release -- --ignored"]
fn fig8_pocket_3d_readings_of_the_old_planner() {
    pocket_3d(2);
}

/// Light twin of `fig8_pocket_3d_readings_of_the_old_planner`: the Inside case only (one generation).
#[test]
#[ignore = "light lane: the Inside case of the full test only (#4154); the full test is fig8_pocket_3d_readings_of_the_old_planner"]
fn fig8_pocket_3d_readings_of_the_old_planner_light() {
    pocket_3d(1);
}

const FIG8_CONTOUR: &[Row] = &[
    ("fig8 Outside wall",
     "lifts 2 named 0 re 2 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 3 (plunge 0 ramp 3 helix 0 rapid 0) | over 1 worst 45 full 0 | fallback 2",
     "link L6"),
];

/// The 2D Contour of the outer wall. It has one case, so it has no
/// reduced set for a light twin (the motion suite `kernel_laws` reads the
/// same planner on the disc and the dog bone in the default lane).
#[test]
#[ignore = "heavy lane (#4154 fig8 Outside wall): one generation runs past the 60 s debug budget (about 460 s) - run with --release -- --ignored"]
fn fig8_contour_2d_readings_of_the_old_planner() {
    let f = fig8();
    let path = f.run(OpType::Contour, PassStart::Auto, AreaControls::default());
    // The wall stock: the part's section grown by the finish stock.
    let mut spec = Spec::new(R8, f.part.offset(WALL).expect("wall"), f.hi.z, planned_deg(WALL));
    spec.part = Some(f.part.clone());
    spec.asked_entry = Some(EntryKind::Helix);
    pin("2D Contour", FIG8_CONTOUR, vec![("fig8 Outside wall".into(), path, spec)]);
}
