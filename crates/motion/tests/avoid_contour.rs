//! Integration tests for 2D Contour "Do not touch" protection (#3344).
//! Provenance: independent `camcraft-motion` tests checking `camcraft-sim::verify::avoid`.

use camcraft_motion::path_picks::{PathPick, PathPicks};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::geometry::{Point3, Triangle};
use camcraft_motion::links::{Compensation, Link, Links};
use camcraft_motion::machining2d::{
    ContourParams, Lead, SectionAt, Side, contour, contour_along_chain, contour_along_chain_avoid,
    contour_avoid, contour_on_picked, contour_on_picked_avoid, contour_open_run,
    contour_open_run_avoid, contour_open_run_sided, contour_open_run_sided_avoid,
    contour_whole_bodies_with, contour_whole_bodies_with_avoid,
};
use camcraft_motion::tools::ToolGeom;
use camcraft_motion::verify::{AVOID_VERIFY_TOL, AvoidItem, AvoidSpec, Cutter, Warning, avoid_hits};

fn rect(x0: f64, y0: f64, x1: f64, y1: f64) -> Region {
    Region::new(vec![Loop::new(vec![
        Point2::new(x0, y0),
        Point2::new(x1, y0),
        Point2::new(x1, y1),
        Point2::new(x0, y1),
    ])
    .unwrap()])
    .unwrap()
}

fn box_facets(min: [f64; 3], max: [f64; 3]) -> Vec<Triangle> {
    let p = |x, y, z| Point3::new(x, y, z);
    let v = |i: usize| p([min[0], max[0]][i & 1], [min[1], max[1]][(i >> 1) & 1], [min[2], max[2]][(i >> 2) & 1]);
    let quad = |a: usize, b: usize, c: usize, d: usize| -> [Triangle; 2] {
        [Triangle::new(v(a), v(b), v(c)), Triangle::new(v(a), v(c), v(d))]
    };
    let mut t = Vec::with_capacity(12);
    t.extend(quad(4, 5, 7, 6));
    t.extend(quad(0, 2, 3, 1));
    t.extend(quad(0, 1, 5, 4));
    t.extend(quad(2, 6, 7, 3));
    t.extend(quad(1, 3, 7, 5));
    t.extend(quad(2, 0, 4, 6));
    t
}

fn stepped_cutter() -> Cutter {
    Cutter::flat(&ToolGeom {
        radius: 3.0,
        nose_radius: 0.0,
        flute_len: 8.0,
        shank_radius: 5.5,
        shank_len: 10.0,
        holder_radius: 14.0,
        holder_len: 20.0,
    })
}

fn base_contour(side: Side, entry: Entry3D, lead: Lead, links: Links) -> ContourParams {
    ContourParams {
        tool_diameter: 6.0,
        stock_to_leave: 0.0,
        clearance_z: 25.0,
        retract_z: 10.0,
        top_z: 0.0,
        bottom_z: -5.0,
        stepdown: 2.5,
        feed: 600.0,
        plunge_feed: 150.0,
        climb: true,
        side,
        lead,
        tolerance: 0.01,
        tab_count: 0,
        tab_width: 4.0,
        tab_height: 1.5,
        entry,
        helix_radius: 1.5,
        blend_turns: false,
        blend_radius: None,
        links,
        picks: PathPicks::default(),
    }
}

fn assert_clean(path: &camcraft_motion::cl::Toolpath, spec: &AvoidSpec, label: &str) {
    let hits = avoid_hits(path, spec, AVOID_VERIFY_TOL).expect("verify");
    let details: Vec<_> = hits
        .iter()
        .filter_map(|w| match w {
            Warning::AvoidHit { at, part, short_mm, .. } => path.moves.get(*at).map(|m| {
                format!("#{at} {part:?} short={short_mm:.4} {:?} {:?}->{:?}", m.kind, m.from, m.to)
            }),
            _ => None,
        })
        .collect();
    assert!(hits.is_empty(), "{label} violated Do not touch: {details:#?}");
}

#[test]
fn empty_avoid_matches_plain_contour_paths_exactly() {
    let reg = rect(-20.0, -20.0, 20.0, 20.0);
    let part = box_facets([-20.0, -20.0, -10.0], [20.0, 20.0, 0.0]);
    let empty = AvoidSpec { items: vec![], cutter: stepped_cutter(), clearance: 0.0 };

    let cp = base_contour(Side::Outside, Entry3D::Arc, Lead::Arc { radius: 3.0 }, Links::ROUGH3D);
    assert_eq!(contour_avoid(&reg, &cp, &empty).unwrap(), contour(&reg, &cp).unwrap());
    assert_eq!(contour_on_picked_avoid(&reg, &part, &[], &cp, &empty).unwrap(), contour_on_picked(&reg, &part, &[], &cp).unwrap());
    assert_eq!(
        contour_whole_bodies_with_avoid(&reg, &part, &[], &cp, &empty, &mut Ok).unwrap(),
        contour_whole_bodies_with(&reg, &part, &[], &cp, &mut Ok).unwrap()
    );

    let chain = vec![Point2::new(-15.0, -25.0), Point2::new(15.0, -25.0)];
    let cp_open = base_contour(Side::Auto, Entry3D::Plunge, Lead::Linear { length: 3.0 }, Links::PLANAR);
    assert_eq!(contour_along_chain_avoid(&chain, &cp_open, &empty).unwrap(), contour_along_chain(&chain, &cp_open).unwrap());

    let sec: &SectionAt = &|_z| Ok(Some(reg.clone()));
    // Stock on one side of the run only: main refuses a run with stock on both sides.
    let stock_sec: &SectionAt = &|_z| Ok(Some(rect(-30.0, -30.0, 30.0, 20.0)));
    let no_part: &SectionAt = &|_z| Ok(None);
    let run = vec![Point2::new(-20.0, 20.0), Point2::new(20.0, 20.0)];
    assert_eq!(contour_open_run_avoid(&run, &cp_open, sec, &empty).unwrap(), contour_open_run(&run, &cp_open, sec).unwrap());
    assert_eq!(
        contour_open_run_sided_avoid(&run, &cp_open, Side::Outside, stock_sec, no_part, &empty).unwrap(),
        contour_open_run_sided(&run, &cp_open, Side::Outside, stock_sec, no_part).unwrap()
    );
}

#[test]
fn contour_avoid_closed_partial_tabs_inside_whole_and_open_stay_clear() {
    let reg = rect(-30.0, -20.0, 30.0, 20.0);
    // Protected block straddling the east wall (triggers partial trim + shank/holder clearance),
    // elevated holder ledge on north edge, and bare 3D edge near south lead-out.
    let mut tris = box_facets([27.0, -6.0, -10.0], [38.0, 6.0, 12.0]);
    tris.extend(box_facets([-4.0, 25.0, 6.0], [4.0, 34.0, 22.0]));
    let spec = AvoidSpec {
        items: vec![AvoidItem {
            label: "protected block".into(),
            tris,
            edges: vec![([-15.0, -25.0, -2.5], [15.0, -25.0, -2.5])],
        }],
        cutter: stepped_cutter(),
        clearance: 1.0,
    };

    let keep_down = Links { pass: Link::KeepDown, level: Link::KeepDown, ..Links::ROUGH3D };
    for (entry, lead, links, blend, tabs, label) in [
        (Entry3D::Plunge, Lead::Arc { radius: 4.0 }, Links::PLANAR, false, 0, "plunge+arc"),
        (Entry3D::Arc, Lead::Tangential { radius: 3.5 }, Links::ROUGH3D, true, 0, "arc+tan+blend"),
        (Entry3D::Plunge, Lead::Linear { length: 4.0 }, keep_down, false, 2, "keepdown+tabs"),
    ] {
        let mut p = base_contour(Side::Outside, entry, lead, links);
        p.blend_turns = blend;
        p.tab_count = tabs;
        let path = contour_avoid(&reg, &p, &spec).unwrap_or_else(|e| panic!("{label}: {e:?}"));
        assert_clean(&path, &spec, label);
    }

    // Whole-model stepped part with protected block on east step.
    let mut stepped_part = box_facets([-30.0, -20.0, -5.0], [30.0, 20.0, 0.0]);
    stepped_part.extend(box_facets([-20.0, -12.0, -5.0], [20.0, 12.0, -2.5]));
    let p_whole = base_contour(Side::Outside, Entry3D::Arc, Lead::Linear { length: 3.0 }, Links::ROUGH3D);
    let (whole_path, _) = contour_whole_bodies_with_avoid(&reg, &stepped_part, &[], &p_whole, &spec, &mut Ok).unwrap();
    assert_clean(&whole_path, &spec, "whole_bodies");

    // Bore contour (Side::Inside) with protected island off-center inside bore.
    let bore_reg = rect(-25.0, -25.0, 25.0, 25.0);
    let bore_spec = AvoidSpec {
        items: vec![AvoidItem {
            label: "bore pin".into(),
            tris: box_facets([12.0, -4.0, -10.0], [18.0, 4.0, 8.0]),
            edges: vec![],
        }],
        cutter: stepped_cutter(),
        clearance: 1.0,
    };
    for entry in [Entry3D::Helix, Entry3D::Ramp, Entry3D::Plunge] {
        let p_in = base_contour(Side::Inside, entry, Lead::Linear { length: 3.0 }, Links::ROUGH3D);
        let path = contour_avoid(&bore_reg, &p_in, &bore_spec).unwrap_or_else(|e| panic!("inside {entry:?}: {e:?}"));
        assert_clean(&path, &bore_spec, "inside_bore");
    }

    // Open chain & open run trimmed across protected pin mid-chain.
    let open_chain = vec![Point2::new(-30.0, 0.0), Point2::new(30.0, 0.0)];
    let mid_spec = AvoidSpec {
        items: vec![AvoidItem {
            label: "center post".into(),
            tris: box_facets([-4.0, -4.0, -10.0], [4.0, 4.0, 15.0]),
            edges: vec![],
        }],
        cutter: stepped_cutter(),
        clearance: 1.0,
    };
    let mut p_chain = base_contour(Side::Auto, Entry3D::Plunge, Lead::Linear { length: 4.0 }, Links::ROUGH3D);
    p_chain.retract_z = 8.0; // below post top 15 -> elevate_rapid_links must raise hop!
    let chain_path = contour_along_chain_avoid(&open_chain, &p_chain, &mid_spec).unwrap();
    assert_clean(&chain_path, &mid_spec, "open_chain");
    let stock_sec: &SectionAt = &|_z| Ok(Some(rect(-40.0, -40.0, 40.0, 0.0)));
    let no_part: &SectionAt = &|_z| Ok(None);
    let sided_path = contour_open_run_sided_avoid(&open_chain, &p_chain, Side::Outside, stock_sec, no_part, &mid_spec).unwrap();
    assert_clean(&sided_path, &mid_spec, "open_run_sided");
}

#[test]
fn contour_avoid_refuses_unavoidable_control_and_blocked_pick_by_name() {
    let reg = rect(-20.0, -20.0, 20.0, 20.0);
    let full = AvoidSpec {
        items: vec![AvoidItem {
            label: "full shield".into(),
            tris: box_facets([-40.0, -40.0, -10.0], [40.0, 40.0, 20.0]),
            edges: vec![],
        }],
        cutter: stepped_cutter(),
        clearance: 0.5,
    };
    let p = base_contour(Side::Outside, Entry3D::Plunge, Lead::Linear { length: 4.0 }, Links::PLANAR);
    let err = contour_avoid(&reg, &p, &full).expect_err("fully covered contour must refuse");
    assert!(format!("{err:?}").contains("full shield"), "{err:?}");

    // Partial overlap with G41 Control compensation must refuse rather than breaking closed ring.
    let side_block = AvoidSpec {
        items: vec![AvoidItem {
            label: "side lug".into(),
            tris: box_facets([18.0, -5.0, -10.0], [30.0, 5.0, 10.0]),
            edges: vec![],
        }],
        cutter: stepped_cutter(),
        clearance: 1.0,
    };
    let mut p_ctrl = p.clone();
    p_ctrl.links.approach.compensation = Compensation::Control;
    // #3891: control compensation takes only the Perpendicular lead.
    p_ctrl.lead = Lead::Perpendicular { length: 4.0 };
    let ctrl_err = contour_avoid(&reg, &p_ctrl, &side_block).expect_err("control comp on clipped ring must refuse");
    assert!(format!("{ctrl_err:?}").contains("side lug"), "{ctrl_err:?}");

    // Picked start point inside keep-out must refuse naming the protected item.
    let mut p_pick = p;
    p_pick.picks.start = vec![PathPick::xy(Point2::new(23.0, 0.0))];
    let pick_err = contour_avoid(&reg, &p_pick, &side_block).expect_err("blocked start_point must refuse");
    assert!(format!("{pick_err:?}").contains("side lug"), "{pick_err:?}");
}
