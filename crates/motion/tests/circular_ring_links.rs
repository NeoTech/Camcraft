//! #2898: a Circular with several radial passes (rings) linked each
//! ring's floor to the next ring's start with one diagonal rapid - up
//! from the floor and inward at once - through the core the next ring
//! has still to cut. The verifier convicted it and held the post. These
//! pin the link between rings: every lateral rapid rides at or above the
//! stock top, the verifier finds nothing, and a spliced diagonal link of
//! the old shape is still convicted.
//!
//! Provenance: our own fixture (the issue's hole-work numbers: R10,
//! Ø6 cutter, 2 mm steps, Z20 to Z0 in a 90 x 90 x 20 block), specced
//! from the Prime Directive and the shop rule "never traverse through
//! uncut stock"; no vendor material.
//!
//! Gate map: covers camcraft_motion::machining2d (circular),
//! camcraft_sim::verify (rapids).

use std::f64::consts::PI;

use camcraft_motion::cl::{Move, MoveKind, Toolpath};
use camcraft_motion::entry::Entry3D;
use camcraft_motion::geo2d::{Loop, Point2, Region};
use camcraft_motion::links::Links;
use camcraft_motion::machining2d::{CircularParams, circular};
use camcraft_motion::verify::{Mode, ToolGeom, VerifyContext, Warning, verify};

const C: Point2 = Point2 { x: 30.0, y: 30.0 };
const TOP: f64 = 20.0;
const CLEAR: f64 = 30.0;
const R_TOOL: f64 = 3.0;

fn params(entry: Entry3D, links: Links, vault: bool) -> CircularParams {
    CircularParams {
        links,
        center: C,
        radius: 10.0,
        vault,
        bottom_z: 0.0,
        top_z: TOP,
        tool_diameter: 2.0 * R_TOOL,
        stock_to_leave: 0.0,
        stepover: 2.0,
        stepdown: 2.0,
        clearance_z: CLEAR,
        retract_z: None,
        feed: 800.0,
        plunge_feed: 200.0,
        climb: true,
        entry,
        helix_radius: 1.0,
    }
}

fn disk(r: f64) -> Region {
    let pts: Vec<Point2> = (0..256)
        .map(|k| {
            let t = 2.0 * PI * f64::from(k) / 256.0;
            Point2::new(C.x + r * t.cos(), C.y + r * t.sin())
        })
        .collect();
    Region::new(vec![Loop::new(pts).unwrap()]).unwrap()
}

/// The 90 x 90 block about the hole.
fn block() -> Region {
    let sq = |x: f64, y: f64| Point2::new(C.x + x, C.y + y);
    Region::new(vec![Loop::new(vec![sq(-45.0, -45.0), sq(45.0, -45.0), sq(45.0, 45.0), sq(-45.0, 45.0)]).unwrap()]).unwrap()
}

fn lateral(m: &Move) -> bool {
    (m.to.x - m.from.x).hypot(m.to.y - m.from.y) > 1e-6
}

/// The rapids that move in XY under the stock top.
fn low_traverses(t: &Toolpath) -> Vec<usize> {
    t.moves
        .iter()
        .enumerate()
        .filter(|(_, m)| m.kind == MoveKind::Rapid && lateral(m) && m.from.z.min(m.to.z) < TOP - 1e-9)
        .map(|(i, _)| i)
        .collect()
}

fn judge(t: &Toolpath) -> Vec<Warning> {
    let region = disk(10.0);
    let stock = block();
    let tool = ToolGeom {
        radius: R_TOOL,
        nose_radius: 0.0,
        flute_len: 25.0,
        shank_radius: R_TOOL,
        shank_len: 40.0,
        holder_radius: 20.0,
        holder_len: 60.0,
    };
    verify(
        t,
        &VerifyContext {
            region: &region,
            stock: Some(&stock),
            model: None,
            tool: &tool,
            cutter: None,
            mode: Mode::Area,
            stock_top_z: TOP,
            clearance_z: CLEAR,
            stock_to_leave: 0.0,
            gouge_tol: 0.01,
            z_tol: 1e-6,
            finish_tol: None,
            fixtures: &[],
            spindle: None,
            ..Default::default()
        },
    )
}

fn rapids_in_stock(w: &[Warning]) -> Vec<usize> {
    w.iter()
        .filter_map(|w| match *w {
            Warning::RapidInStock { at } => Some(at),
            _ => None,
        })
        .collect()
}

/// The issue's case and its neighbours: every entry, the planar and the
/// default links, flat and vault. Each ring after the first is reached
/// over the stock, and the verifier convicts no rapid.
#[test]
fn the_link_between_circular_rings_clears_the_stock() {
    for entry in [Entry3D::Plunge, Entry3D::Helix, Entry3D::Ramp] {
        for (tag, links) in [("planar", Links::PLANAR), ("default", Links::default())] {
            for vault in [false, true] {
                let t = circular(&disk(10.0), &params(entry, links, vault)).expect("a legal hole");
                let low = low_traverses(&t);
                assert!(low.is_empty(), "{entry:?}/{tag}/vault {vault}: rapids under the stock top at {low:?}: {:?}", low.first().map(|&i| &t.moves[i]));
                let convicted = rapids_in_stock(&judge(&t));
                assert!(convicted.is_empty(), "{entry:?}/{tag}/vault {vault}: verify convicts rapids {convicted:?}");
            }
        }
    }
}

/// The issue's plunge case cuts four rings (R 7, 5, 3, 1), so it has
/// three links between rings (onto R 5, 3 and 1; ring 0 enters at its
/// own dive point), each a climb to clearance before the traverse - not
/// a single ring that hides the law.
#[test]
fn the_issue_case_climbs_to_clearance_between_its_rings() {
    let t = circular(&disk(10.0), &params(Entry3D::Plunge, Links::PLANAR, false)).expect("a legal hole");
    let traverses: Vec<&Move> = t.moves.iter().filter(|m| m.kind == MoveKind::Rapid && lateral(m)).collect();
    let radii: Vec<f64> = traverses.iter().map(|m| ((m.to.x - C.x).hypot(m.to.y - C.y) * 1e6).round() / 1e6).collect();
    for rho in [5.0, 3.0, 1.0] {
        assert!(radii.contains(&rho), "a traverse onto ring {rho}: {radii:?}");
    }
    assert!(traverses.iter().all(|m| m.from.z >= CLEAR - 1e-9 && m.to.z >= CLEAR - 1e-9), "{traverses:?}");
}

/// The verifier still holds a link of the old shape: one diagonal rapid
/// from a ring's floor up to clearance over the next ring's start.
#[test]
fn a_spliced_diagonal_link_between_rings_is_still_convicted() {
    let t = circular(&disk(10.0), &params(Entry3D::Plunge, Links::PLANAR, false)).expect("a legal hole");
    // The first climb off the floor with a traverse after it (ring 0 to
    // ring 1): the two become the one diagonal rapid the old code posted.
    let lift = (0..t.moves.len().saturating_sub(1))
        .find(|&i| {
            let (m, n) = (&t.moves[i], &t.moves[i + 1]);
            m.kind == MoveKind::Rapid && !lateral(m) && m.from.z < 1e-9 && m.to.z >= CLEAR - 1e-9 && n.kind == MoveKind::Rapid && lateral(n)
        })
        .expect("a climb from the first ring's floor to the next ring");
    let (m, n) = (&t.moves[lift], &t.moves[lift + 1]);
    let diag = Move { from: m.from, to: n.to, ..*m };
    let mut bad = t.clone();
    bad.moves.splice(lift..lift + 2, [diag]);
    assert_eq!(rapids_in_stock(&judge(&t)), Vec::<usize>::new(), "the real path is clean");
    let convicted = rapids_in_stock(&judge(&bad));
    assert!(convicted.contains(&lift), "the diagonal link at move {lift} must be convicted: {convicted:?}");
}
