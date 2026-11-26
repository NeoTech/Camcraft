//! #4153 2D Contour baseline: a 40 x 40 square, Outside and Inside, climb
//! and conventional, each entry kind, an arc lead, a start pick and tabs.

use super::{pin, poly, region};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{ContourParams, Lead, Side, contour};
use camcraft_motion::path_picks::{PathPick, PathPicks};

fn params(side: Side) -> ContourParams {
    ContourParams {
        tool_diameter: 6.0,
        stock_to_leave: 0.0,
        clearance_z: 5.0,
        retract_z: 2.0,
        top_z: 0.0,
        bottom_z: -4.0,
        stepdown: 2.0,
        feed: 500.0,
        plunge_feed: 100.0,
        climb: true,
        side,
        lead: Lead::Perpendicular { length: 5.0 },
        tolerance: 0.01,
        tab_count: 0,
        tab_width: 0.0,
        tab_height: 0.0,
        entry: Entry3D::Plunge,
        helix_radius: 1.0,
        blend_turns: false,
        blend_radius: None,
        links: Links::PLANAR,
        picks: PathPicks::default(),
    }
}

#[test]
fn kernel_baseline_contour() {
    let square = region(vec![poly(&[(0.0, 0.0), (40.0, 0.0), (40.0, 40.0), (0.0, 40.0)])]);
    let out = || params(Side::Outside);
    let inn = || params(Side::Inside);
    let cases: Vec<(&str, ContourParams)> = vec![
        ("outside", out()),
        ("outside_conventional", ContourParams { climb: false, ..out() }),
        ("outside_ramp", ContourParams { entry: Entry3D::Ramp, ..out() }),
        ("outside_helix", ContourParams { entry: Entry3D::Helix, ..out() }),
        ("outside_arc_lead", ContourParams { lead: Lead::Arc { radius: 4.0 }, entry: Entry3D::Arc, ..out() }),
        (
            "outside_picked",
            ContourParams {
                picks: PathPicks { start: vec![PathPick::xy(Point2::new(20.0, -3.0))], exit: Vec::new() },
                ..out()
            },
        ),
        ("outside_tabs", ContourParams { tab_count: 4, tab_width: 4.0, tab_height: 1.0, ..out() }),
        ("inside", inn()),
        ("inside_conventional", ContourParams { climb: false, ..inn() }),
        ("inside_helix", ContourParams { entry: Entry3D::Helix, ..inn() }),
    ];
    for (case, p) in cases {
        pin("contour", case, &contour(&square, &p).unwrap_or_else(|e| panic!("{case}: {e}")));
    }
}
