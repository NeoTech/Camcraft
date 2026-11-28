//! Finding 2 of the #4123 review: the step law of a plan is a transition
//! table. A cut consumes the readiness, so each cut needs its own entry or
//! stay-down link. Every cell of the table is tested.

use super::common::{pts, square_region, tool};
use camcraft_pathkernel::entry::{Area, Entry, EntryChoice, EntryKind, Exit, Proof};
use camcraft_pathkernel::guide::{Curve, Family};
use camcraft_pathkernel::link::Hop;
use camcraft_pathkernel::plan::{BAD_AXIS, CUT_WITHOUT_ENTRY, Cut, DANGLING, NO_EXIT, Plan, STRAY_ENTRY, STRAY_EXIT, STRAY_LINK, Step};
use camcraft_pathkernel::Point2;

fn cut(y: f64) -> Step {
    let c = Curve::new(pts(&[(0.0, y), (5.0, y)]), None, false, 0, Family::Row).unwrap();
    Step::Cut(Cut::new(c, None).unwrap())
}
fn entry() -> Step {
    let c = EntryChoice::new(EntryKind::Air, 2.0, vec![]).unwrap();
    let proof = Proof::InAir(Area::new(square_region(-1.0, -1.0, 20.0)).unwrap());
    Step::Entry(Entry::new(EntryKind::Air, vec![], pts(&[(0.0, 0.0)]), proof, &c, tool(3.0, false)).unwrap())
}
fn exit() -> Step {
    Step::Exit(Exit::new(None, 5.0, 0.0).unwrap())
}
fn stay() -> Step {
    Step::Link(Hop::stay(vec![Point2::new(5.0, 1.0)]).unwrap())
}
fn lift() -> Step {
    Step::Link(Hop::lift("crosses a planned cut").unwrap())
}

fn law(steps: Vec<Step>) -> Result<(), &'static str> {
    Plan::new(steps).map(|_| ()).map_err(|e| e.reason())
}

#[test]
fn the_legal_sequences_pass() {
    assert!(law(vec![]).is_ok());
    assert!(law(vec![entry(), cut(0.0), exit()]).is_ok());
    assert!(law(vec![entry(), cut(0.0), stay(), cut(1.0), exit()]).is_ok());
    assert!(law(vec![entry(), cut(0.0), lift(), entry(), cut(9.0), exit()]).is_ok());
    assert!(law(vec![entry(), cut(0.0), exit(), entry(), cut(9.0), exit()]).is_ok());
}

#[test]
fn a_cut_consumes_the_readiness() {
    // Two disconnected cuts in a row: the second has no entry or stay link.
    assert_eq!(law(vec![entry(), cut(0.0), cut(9.0), exit()]).unwrap_err(), CUT_WITHOUT_ENTRY);
    assert_eq!(law(vec![cut(0.0), exit()]).unwrap_err(), CUT_WITHOUT_ENTRY);
    assert_eq!(law(vec![entry(), cut(0.0), lift(), cut(9.0), exit()]).unwrap_err(), CUT_WITHOUT_ENTRY);
    assert_eq!(law(vec![entry(), cut(0.0), exit(), cut(9.0), exit()]).unwrap_err(), CUT_WITHOUT_ENTRY);
}

#[test]
fn each_illegal_cell_of_the_table_is_refused() {
    // clear: link, exit.
    assert_eq!(law(vec![stay()]).unwrap_err(), STRAY_LINK);
    assert_eq!(law(vec![lift()]).unwrap_err(), STRAY_LINK);
    assert_eq!(law(vec![exit()]).unwrap_err(), STRAY_EXIT);
    // ready: entry, link, exit.
    assert_eq!(law(vec![entry(), entry()]).unwrap_err(), STRAY_ENTRY);
    assert_eq!(law(vec![entry(), stay(), cut(0.0), exit()]).unwrap_err(), STRAY_LINK);
    assert_eq!(law(vec![entry(), exit()]).unwrap_err(), STRAY_EXIT);
    // cut end: entry.
    assert_eq!(law(vec![entry(), cut(0.0), entry(), cut(9.0), exit()]).unwrap_err(), STRAY_ENTRY);
    // lifted: cut, link, exit.
    assert_eq!(law(vec![entry(), cut(0.0), lift(), stay()]).unwrap_err(), STRAY_LINK);
    assert_eq!(law(vec![entry(), cut(0.0), lift(), exit()]).unwrap_err(), STRAY_EXIT);
}

#[test]
fn a_plan_ends_clear() {
    assert_eq!(law(vec![entry(), cut(0.0)]).unwrap_err(), NO_EXIT);
    assert_eq!(law(vec![entry(), cut(0.0), stay(), cut(1.0)]).unwrap_err(), NO_EXIT);
    assert_eq!(law(vec![entry()]).unwrap_err(), DANGLING);
    assert_eq!(law(vec![entry(), cut(0.0), stay()]).unwrap_err(), DANGLING);
    assert_eq!(law(vec![entry(), cut(0.0), lift()]).unwrap_err(), DANGLING);
}

#[test]
fn a_cut_axis_is_a_unit_vector() {
    let c = Curve::new(pts(&[(0.0, 0.0), (1.0, 0.0)]), None, false, 0, Family::Trace).unwrap();
    assert!(Cut::new(c.clone(), Some([0.0, 0.0, 1.0])).is_ok());
    assert_eq!(Cut::new(c.clone(), Some([0.0, 0.0, 2.0])).unwrap_err().reason(), BAD_AXIS);
    assert_eq!(Cut::new(c, Some([f64::NAN, 0.0, 1.0])).unwrap_err().reason(), BAD_AXIS);
}
