//! #1665: a Bore with `entry: ramp` and no ramp angle descends at the
//! ramp angle, or substitutes by name. Every descending feed move of the
//! path is either a ramp piece no steeper than the ramp angle, or an
//! axial plunge at the plunge feed that a note names - never the steep
//! chord from the last ring start to the centre at the cutting feed.
//!
//! Provenance: our own tests over the kernel's ramp law (standard linear
//! ramp entry, RS-274 G1/G2/G3); no vendor material consulted.
//!
//! Gate map: covers camcraft_motion::machining2d (bore, descent, ladder_links).

use camcraft_motion::cl::{Move, MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::Point2;
use camcraft_motion::links::{DEFAULT_RAMP_ANGLE, Links};
use camcraft_motion::machining2d::{BoreHole, BoreParams, bore};

const FEED: f64 = 500.0;
const PLUNGE: f64 = 250.0;

/// The issue's repro: one blind hole at (0,0), top 0, floor -6, a Ø6
/// square end mill, 2 mm stepdown, entry Ramp, planar links.
fn params(diameter: f64, climb: bool) -> BoreParams {
    BoreParams {
        holes: vec![BoreHole {
            xy: Point2::new(0.0, 0.0),
            radius: diameter * 0.5,
            top_z: 0.0,
            floor_z: -6.0,
            through: false,
            stepped: false,
        }],
        tool_diameter: 6.0,
        corner_radius: 0.0,
        stock_to_leave: 0.0,
        stepdown: 2.0,
        through_allowance: 0.0,
        clearance_z: 5.0,
        retract_z: None,
        feed: FEED,
        plunge_feed: PLUNGE,
        climb,
        tolerance: 0.01,
        entry: Entry3D::Ramp,
        helix_radius: 1.0,
        links: Links::PLANAR,
    }
}

fn path(p: &BoreParams) -> Toolpath {
    bore(p).expect("a legal bore")
}

/// The XY length a move flies: the chord of a line, radius times the
/// sweep (in its own sense) of an arc.
fn xy_len(m: &Move) -> f64 {
    match m.arc {
        None => (m.to.x - m.from.x).hypot(m.to.y - m.from.y),
        Some(a) => {
            let ang = |p: camcraft_motion::geometry::Point3| (p.y - a.center.y).atan2(p.x - a.center.x);
            let d = if a.ccw { ang(m.to) - ang(m.from) } else { ang(m.from) - ang(m.to) };
            let d = d.rem_euclid(2.0 * std::f64::consts::PI);
            let d = if d <= 0.0 { 2.0 * std::f64::consts::PI } else { d };
            (m.from.x - a.center.x).hypot(m.from.y - a.center.y) * d
        }
    }
}

/// The issue's "Done means" on the Ø7 repro and on Ø12: every
/// descending feed move is at most the ramp angle, or an axial move at
/// the plunge feed that an `entry:` note names. Today's code posts an
/// 84 deg (Ø7) and a 34 deg (Ø12) chord at the cutting feed.
#[test]
fn a_ramp_entry_bore_never_descends_steeper_than_its_ramp_angle() {
    let tan = DEFAULT_RAMP_ANGLE.to_radians().tan();
    for diameter in [7.0, 12.0, 16.0] {
        for climb in [true, false] {
            let t = path(&params(diameter, climb));
            let named = t.notes.iter().any(|n| n.starts_with("entry:"));
            let mut ramps = 0;
            for m in &t.moves {
                let dz = m.from.z - m.to.z;
                if m.kind == MoveKind::Rapid || dz <= 0.0 {
                    continue;
                }
                let len = xy_len(m);
                if len > 1e-9 {
                    // 1e-5 mm of XY slack: the endpoint lattice.
                    assert!(
                        dz <= tan * (len + 1e-5),
                        "Ø{diameter} climb {climb}: a descent at {:.3} deg ({:?} -> {:?}) at feed {:?} - steeper than the ramp angle",
                        (dz / len).atan().to_degrees(),
                        m.from,
                        m.to,
                        m.feed
                    );
                    ramps += 1;
                } else {
                    assert_eq!(m.feed, Some(PLUNGE), "Ø{diameter} climb {climb}: an axial descent at {:?} not at the plunge feed", m.to);
                    assert!(named, "Ø{diameter} climb {climb}: an axial descent at {:?} with no note naming it: {:?}", m.to, t.notes);
                }
            }
            assert!(ramps > 0, "Ø{diameter} climb {climb}: the ramp entry ramps somewhere");
        }
    }
}

/// The plunge that starts each ring again (the pass link back down the
/// column the radial sweep just cut) is named: the user selected Ramp,
/// so an axial move at the plunge feed never goes without a note.
#[test]
fn the_ring_restart_plunge_of_a_ramp_entry_bore_is_named() {
    let t = path(&params(12.0, true));
    let axial = t
        .moves
        .iter()
        .filter(|m| m.kind != MoveKind::Rapid && m.from.z > m.to.z && xy_len(m) <= 1e-9)
        .count();
    assert!(axial > 0, "the ring restart dives axially: {:?}", t.notes);
    assert!(
        t.notes.iter().any(|n| n.starts_with("entry:") && n.contains("column")),
        "the ring restart is named: {:?}",
        t.notes
    );
}

/// A hole barely wider than the cutter: Ø3.175 tool, Ø3.2 hole, so the
/// cutter-centre ring is 0.0125 mm in radius. Entry `entry`, the links'
/// ramp and helix angle both `deg`, 1 mm stepdown, 3 mm deep.
fn tight(entry: Entry3D, hole: f64, deg: f64) -> BoreParams {
    let mut links = Links::PLANAR;
    links.approach.ramp_angle = deg;
    links.approach.helix_angle = deg;
    BoreParams {
        holes: vec![BoreHole { xy: Point2::new(0.0, 0.0), radius: hole * 0.5, top_z: 0.0, floor_z: -3.0, through: false, stepped: false }],
        tool_diameter: 3.175,
        corner_radius: 0.0,
        stock_to_leave: 0.0,
        stepdown: 1.0,
        through_allowance: 0.0,
        clearance_z: 5.0,
        retract_z: None,
        feed: FEED,
        plunge_feed: PLUNGE,
        climb: true,
        tolerance: 0.01,
        entry,
        helix_radius: 1.0,
        links,
    }
}

/// Every descending feed move at the cutting feed is at most the angle;
/// the only axial descent is the named re-plunge down the column a ring
/// just cut (one per level at most), never a dive into solid. A hole from
/// 3.2 mm (ring radius 0.0125 mm) to 6 mm (under 2 x the tool).
#[test]
fn a_bore_barely_wider_than_its_cutter_descends_within_the_angle_cap() {
    let deg: f64 = 5.0;
    let tan = deg.to_radians().tan();
    for entry in [Entry3D::Ramp, Entry3D::Helix] {
        for hole in [3.2, 3.5, 4.5, 6.0] {
            let t = path(&tight(entry, hole, deg));
            let mut axial = 0;
            let mut ramps = 0;
            for m in &t.moves {
                let dz = m.from.z - m.to.z;
                if m.kind == MoveKind::Rapid || dz <= 1e-9 {
                    continue;
                }
                let len = xy_len(m);
                if len > 1e-9 {
                    assert!(dz <= tan * (len + 1e-5), "{entry:?} Ø{hole}: a descent steeper than {deg} deg at {:?} -> {:?}", m.from, m.to);
                    ramps += 1;
                } else {
                    axial += 1;
                }
            }
            assert!(ramps > 0, "{entry:?} Ø{hole}: it ramps");
            assert!(axial <= 3, "{entry:?} Ø{hole}: {axial} axial descents, only the three ring re-plunges may stand: {:?}", t.notes);
            assert!(
                t.notes.iter().filter(|n| n.contains("axial dive")).count() == 0,
                "{entry:?} Ø{hole}: an axial dive on the floor remains: {:?}",
                t.notes
            );
            let low = t.moves.iter().filter(|m| m.kind != MoveKind::Rapid).map(|m| m.to.z).fold(f64::INFINITY, f64::min);
            assert!((low + 3.0).abs() < 1e-6, "{entry:?} Ø{hole}: bottoms at the floor, not {low}");
        }
    }
}

/// The default stays the plunge: a Plunge bore posts the same path with
/// or without angles on its links, and a tight hole still plunges.
#[test]
fn a_plunge_bore_ignores_the_ramp_and_helix_angles() {
    let with = path(&tight(Entry3D::Plunge, 3.2, 5.0));
    let without = path(&tight(Entry3D::Plunge, 3.2, 0.0));
    assert_eq!(with.moves.len(), without.moves.len());
    let axial = with.moves.iter().filter(|m| m.kind != MoveKind::Rapid && m.from.z - m.to.z > 1e-9 && xy_len(m) <= 1e-9).count();
    assert!(axial >= 3, "plunge dives straight down at each level");
}
