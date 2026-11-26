//! #4154: the readings of the OLD planners of Slot, Trochoidal slot and
//! Slot Undercut along the neck of the dog bone (from X -15 to X 15).
//! These kinds have no start side. A plain slot and an undercut cut at the
//! full width by plan (a named slot for law L6); a trochoidal slot plans a
//! limited engagement, so a full-width cut there is a reading of L6.

use super::common::law_measure::{EntryKind, Spec};
use super::fixture::{R, TOP, Z, pin, rect, region, stock};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{
    DEFAULT_MICROLIFT, SlotParams, TrochoidParams, TrochoidReturn, UndercutParams, slot,
    trochoidal_slot, undercut,
};
use camcraft_motion::tools::ToolKind;
use camcraft_tools::undercut::{Undercut, cutter};

fn ends() -> (Point2, Point2) {
    (Point2::new(-15.0, 0.0), Point2::new(15.0, 0.0))
}

#[test]
fn slot_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("dog bone neck",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 0 col 0 solid 1 (plunge 1 ramp 0 helix 0 rapid 0) | over 0 worst 180 full 1 | fallback 0",
         "L4 L5"),
    ];
    let p = SlotParams {
        links: Links::PLANAR,
        tool_diameter: 2.0 * R,
        width: 2.0 * R,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        retract_z: Some(2.0),
        top_z: TOP,
        bottom_z: Z,
        stepdown: 2.0,
        stepover: 2.0,
        feed: 600.0,
        plunge_feed: 200.0,
        climb: true,
        entry: Entry3D::Helix,
        helix_radius: 1.5,
        blend_turns: false,
        blend_radius: None,
    };
    let (a, b) = ends();
    let path = slot(a, b, &p).expect("slot");
    let mut spec = Spec::new(R, stock(), TOP, 180.0);
    spec.asked_entry = Some(EntryKind::Helix);
    pin("Slot", true, TABLE, vec![("dog bone neck".into(), path, spec)]);
}

#[test]
fn trochoidal_slot_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("dog bone neck",
         "lifts 30 named 0 re 59 | cross 0 leave 0 | entry air 0 hole 0 col 30 solid 1 (plunge 0 ramp 0 helix 1 rapid 0) | over 2 worst 180 full 1 | fallback 0",
         "link L6"),
    ];
    let p = TrochoidParams {
        tool_diameter: 2.0 * R,
        width: 12.0,
        loop_width: None,
        step: 1.0,
        max_engagement: 90.0,
        ret: TrochoidReturn::Microlift {
            height: DEFAULT_MICROLIFT,
        },
        smoothing: 1.0,
        stock_to_leave: 0.0,
        clearance_z: 10.0,
        retract_z: Some(2.0),
        top_z: TOP,
        bottom_z: Z,
        stepdown: 2.0,
        feed: 1200.0,
        return_feed: 3000.0,
        plunge_feed: 300.0,
        climb: true,
        stock_top: Some(TOP),
    };
    let (a, b) = ends();
    let path = trochoidal_slot(&[a, b], &p).expect("trochoidal slot");
    let spec = Spec::new(R, stock(), TOP, 90.0);
    pin("Trochoidal slot", false, TABLE, vec![("dog bone neck".into(), path, spec)]);
}

#[test]
fn slot_undercut_readings_of_the_old_planner() {
    const TABLE: &[super::fixture::Row] = &[
        ("dog bone neck",
         "lifts 0 named 0 re 0 | cross 0 leave 0 | entry air 0 hole 1 col 0 solid 0 (plunge 0 ramp 0 helix 0 rapid 0) | over 0 worst 180 full 1 | fallback 0",
         "-"),
    ];
    let head = Undercut::of(&cutter(ToolKind::TSlot, 20.0, 6.0, 8.0, 30.0, 0.0)).expect("head");
    let p = UndercutParams {
        cutter: head,
        width: 26.0,
        bulge: 0.0,
        run_in: 40.0,
        neck_slot: 14.0,
        clearance_z: 10.0,
        retract_z: Some(2.0),
        bottom_z: -12.0,
        stepover: 2.0,
        feed: 120.0,
        plunge_feed: 60.0,
        climb: true,
    };
    let (a, b) = ends();
    let path = undercut(a, b, &p).expect("undercut");
    // The neck slot (14 wide) is cut before, with its run-in channel to
    // the station 40 mm before the start, past the stock edge: a hole for
    // the grid. The grid reads columns, so the material over the head is
    // not modelled; the engagement column is a reading only.
    let mut spec = Spec::new(10.0, stock(), TOP, 180.0);
    spec.holes = Some(region(vec![rect(-60.0, -7.0, 22.0, 7.0)]));
    pin("Slot Undercut", true, TABLE, vec![("dog bone neck".into(), path, spec)]);
}
