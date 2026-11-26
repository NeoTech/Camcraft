//! #3237 pins for explicit 2D Chamfer lead geometry and hard fit.

use camcraft_motion::cl::MoveKind;
use camcraft_motion::geometry::{Point3, cube};
use camcraft_motion::machining2d::{
    ChamferLeadSafety, Lead, chamfer_open_runs, chamfer_open_runs_with_lead_safety,
    chamfer_with_lead_safety,
};
use camcraft_motion::verify::{Cutter, SolidLaw};

use super::*;

fn cut(p: &ChamferParams) -> camcraft_motion::cl::Toolpath {
    let run = vec![Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)];
    chamfer_open_runs(&[run], p, &|_| Ok(Some(rect(-20.0, -20.0, 20.0, 20.0))))
        .expect("open Chamfer")
}

fn checked_cut(
    p: &ChamferParams,
    facets: &[camcraft_motion::geometry::Triangle],
    cutter: &Cutter,
) -> Result<camcraft_motion::cl::Toolpath, GenError> {
    let safety = ChamferLeadSafety::new(
        facets,
        cutter,
        SolidLaw {
            step: 0.25,
            allow: 0.01,
            hole_walls: false,
        },
    );
    let run = vec![Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)];
    chamfer_open_runs_with_lead_safety(
        &[run],
        p,
        &|_| Ok(Some(rect(-20.0, -20.0, 20.0, 20.0))),
        &safety,
    )
}

fn checked_loop(
    p: &ChamferParams,
) -> Result<camcraft_motion::cl::Toolpath, GenError> {
    let cutter = cone();
    let safety = ChamferLeadSafety::new(
        &[],
        &cutter,
        SolidLaw {
            step: 0.25,
            allow: 0.01,
            hole_walls: false,
        },
    );
    chamfer_with_lead_safety(
        &rect(-20.0, -20.0, 20.0, 20.0),
        p,
        &safety,
    )
}

fn cone() -> Cutter {
    Cutter {
        radius: 4.0,
        corner_r: 0.0,
        tip_angle: 90.0,
        flute_len: 10.0,
        shank_radius: 6.0,
        shank_len: 30.0,
        holder_radius: 10.0,
        holder_len: 20.0,
        body: None,
        neck: None,
        undercut: None,
    }
}

fn wall_and_blocker(y0: f64) -> Vec<camcraft_motion::geometry::Triangle> {
    let wall = cube(
        Point3::new(-20.0, -20.0, 0.0),
        Point3::new(20.0, 20.0, 20.0),
    );
    let blocker = cube(
        Point3::new(-30.0, y0, 0.0),
        Point3::new(30.0, 40.0, 24.0),
    );
    let mut facets = wall.facets.to_vec();
    facets.extend(blocker.facets.iter().copied());
    facets
}

#[test]
fn explicit_none_emits_no_added_open_chamfer_xy_lead() {
    let mut p = chamfer_params();
    p.lead = Some(Lead::None);
    p.lead_out = None;
    let path = cut(&p);
    let feeds: Vec<_> = path
        .moves
        .iter()
        .filter(|move_| move_.kind == MoveKind::Feed)
        .collect();
    assert_eq!(feeds.len(), 1, "only the selected open run is fed: {feeds:?}");
    assert_eq!(feeds[0].from.y, 20.0);
    assert_eq!(feeds[0].to.y, 20.0);
}

#[test]
fn perpendicular_leads_reuse_the_requested_air_normal() {
    let mut p = chamfer_params();
    p.lead = Some(Lead::Perpendicular { length: 3.0 });
    p.lead_out = Some(Lead::Perpendicular { length: 1.5 });
    let path = checked_cut(&p, &[], &cone()).expect("air-only lead");
    let feeds: Vec<_> = path
        .moves
        .iter()
        .filter(|move_| move_.kind == MoveKind::Feed)
        .collect();
    assert_eq!(feeds.len(), 3, "lead-in, selected run, lead-out: {feeds:?}");
    assert!((feeds[0].from.y - 23.0).abs() < 1e-9);
    assert!((feeds[0].to.y - 20.0).abs() < 1e-9);
    assert!((feeds[2].from.y - 20.0).abs() < 1e-9);
    assert!((feeds[2].to.y - 21.5).abs() < 1e-9);
}

#[test]
fn every_shared_lead_shape_is_posted_by_the_existing_arc_seam() {
    for (lead, arcs) in [
        (Lead::Linear { length: 2.0 }, 0),
        (Lead::Arc { radius: 2.0 }, 2),
        (Lead::Tangential { radius: 2.0 }, 2),
        (Lead::Perpendicular { length: 2.0 }, 0),
    ] {
        let mut p = chamfer_params();
        p.lead = Some(lead);
        p.lead_out = None;
        let path = checked_cut(&p, &[], &cone()).expect("shared lead shape");
        let feeds: Vec<_> = path
            .moves
            .iter()
            .filter(|move_| move_.kind == MoveKind::Feed)
            .collect();
        assert_eq!(feeds.len(), 3, "{lead:?}: {feeds:?}");
        assert_eq!(
            feeds.iter().filter(|move_| move_.arc.is_some()).count(),
            arcs,
            "{lead:?}"
        );
    }
}

#[test]
fn every_shared_lead_shape_closes_one_chamfer_loop() {
    for lead in [
        Lead::Linear { length: 2.0 },
        Lead::Arc { radius: 2.0 },
        Lead::Tangential { radius: 2.0 },
        Lead::Perpendicular { length: 2.0 },
    ] {
        let mut p = chamfer_params();
        p.lead = Some(lead);
        p.lead_out = None;
        let path = checked_loop(&p).expect("the closed Chamfer lead fits");
        assert_eq!(path.breaks.len(), 1, "{lead:?}");
        assert!(
            path.moves.iter().any(|move_| move_.kind == MoveKind::Feed),
            "{lead:?}"
        );
    }
}

#[test]
fn blocked_requested_leads_halve_separately_and_name_each_note() {
    let mut p = chamfer_params();
    p.lead = Some(Lead::Perpendicular { length: 8.0 });
    p.lead_out = None;
    let path = checked_cut(&p, &wall_and_blocker(29.0), &cone())
        .expect("half the perpendicular lead clears the blocker");
    assert!(
        path.notes
            .iter()
            .any(|note| note.contains("lead-in") && note.contains("halved"))
    );
    assert!(
        path.notes
            .iter()
            .any(|note| note.contains("lead-out") && note.contains("halved"))
    );
    let feeds: Vec<_> = path
        .moves
        .iter()
        .filter(|move_| move_.kind == MoveKind::Feed)
        .collect();
    assert!((feeds[0].from.y - 24.0).abs() < 1e-9, "{feeds:?}");
    assert!((feeds[2].to.y - 24.0).abs() < 1e-9, "{feeds:?}");
}

#[test]
fn blocked_requested_lead_refuses_instead_of_posting_the_bare_wall() {
    let mut p = chamfer_params();
    p.lead = Some(Lead::Perpendicular { length: 8.0 });
    p.lead_out = None;
    let error = checked_cut(&p, &wall_and_blocker(20.1), &cone())
        .expect_err("no positive half-step clears the blocked air");
    let why = error.to_string();
    assert!(why.contains("cannot fit after four halvings"), "{why}");
    assert!(why.contains("full cone collision"), "{why}");
}

#[test]
fn lead_refuses_a_modelled_body_collision_above_the_cone() {
    let wall = cube(
        Point3::new(-20.0, -20.0, 0.0),
        Point3::new(20.0, 20.0, 20.0),
    );
    let body_obstacle = cube(
        Point3::new(-30.0, 15.0, 36.0),
        Point3::new(30.0, 110.0, 45.0),
    );
    let mut facets = wall.facets.to_vec();
    facets.extend(body_obstacle.facets.iter().copied());
    let mut p = chamfer_params();
    p.lead = Some(Lead::Perpendicular { length: 80.0 });
    p.lead_out = Some(Lead::None);
    let why = checked_cut(&p, &facets, &cone())
        .expect_err("the shank/body cannot pass through the upper obstacle")
        .to_string();
    assert!(why.contains("full body collision"), "{why}");
}
