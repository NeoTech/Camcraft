//! The multi-band and single-chain laws (#2829, the third #2804
//! slice): three ordered guide chains machine two bands whose shared
//! boundary chain is cut once; a single helical chain covers a full
//! revolution as its own next lap; a ring that crosses a corner chain
//! is refused by name, at the crossing's own spot; a middle pick
//! reversed holds the cutting direction of every band, for both climb
//! settings (PR #4018's set normalization); and a single ring guide -
//! closed or an open arc - is refused by name, its lap family sweeps
//! no band area; and a guide that crosses a guide that is not its
//! neighbour in the pick is refused by name at the crossing (#4032),
//! while a clean three-guide set on the same prism still machines.
//!
//! Gate map (M4.4): covers camcraft_motion::multiaxis (rotary_flow
//! and rotary_flow_set, the #2829 laws) on this crate's own
//! fixtures.

use camcraft_motion::cl::{MoveKind, Toolpath};
use camcraft_motion::geometry::Point3;
use camcraft_motion::multiaxis::{rotary_flow, AxisControl};

use crate::fixture::*;

/// The groove's three root-line helices at v-offsets 0, half and the
/// full floor width - all exact stations of the floor strip's own
/// vertex columns, so every point is on the model.
fn root_trio(n_circ: usize, turns: usize, phase: f64) -> Vec<Vec<Point3>> {
    let step = TAU / n_circ as f64;
    let mut out = Vec::with_capacity(3);
    for f in [0.0, 0.5, 1.0] {
        let mut c = Vec::with_capacity(n_circ * turns + 1);
        for j in 0..=n_circ * turns {
            let th = phase + step * j as f64;
            c.push(gp(R_ROOT, th, law(th) + W_V * f));
        }
        out.push(c);
    }
    out
}

/// The landing station of every pass: the ball centre of each plunge.
fn landings(path: &Toolpath, r: f64) -> Vec<Point3> {
    path.moves
        .iter()
        .filter(|m| m.kind == MoveKind::Plunge)
        .filter_map(|m| center_of(m, r))
        .collect()
}

/// Every pass's ball centres: a plunge opens a pass, and the
/// retract onto the hop cylinder closes it (the cut moves in
/// between stay under the link radius).
fn every_pass(path: &Toolpath, clear: f64, r: f64) -> Vec<Vec<Point3>> {
    let mut out: Vec<Vec<Point3>> = Vec::new();
    for m in path.moves.iter().filter(|m| m.kind != MoveKind::Rapid) {
        let Some(c) = center_of(m, r) else { continue };
        if cyl(c).1 > clear - 2.0 {
            continue; // the retract onto the link cylinder
        }
        if m.kind == MoveKind::Plunge {
            out.push(vec![c]);
        } else if let Some(p) = out.last_mut() {
            p.push(c);
        }
    }
    out
}

#[test]
fn three_ordered_chains_cut_two_bands_and_the_boundary_once() {
    // One turn of the groove fixture, three root-line helices picked
    // in band order. Each 2.0 mm band takes ceil(2.0 / 0.9) + 1 = 4
    // passes; the shared middle chain is band 1's own last pass
    // already, so band 2 starts at its first interior pass: 4 + 4 -
    // 1 = 7 passes, and no two of them land on one station.
    let facets = groove(N_CIRC, 1, 0.0);
    let chains = root_trio(N_CIRC, 1, 0.0);
    let tool = ball(2.0);
    let path = rotary_flow(
        &chains,
        &facets,
        &stated_axis(),
        &params(&tool, Some(0.9)),
        &AxisControl::default(),
    )
    .unwrap();
    no_gouge(&path, &facets, 1.0);
    no_swept_gouge(&path, &facets, 1.0);
    let ls = landings(&path, 1.0);
    assert_eq!(
        ls.len(),
        7,
        "the two bands did not divide into 7 passes: {:?}",
        path.notes
    );
    for (i, &x) in ls.iter().enumerate() {
        for &y in ls.iter().skip(i + 1) {
            let d = x - y;
            assert!(
                d.dot(d) > 1e-12,
                "two passes land on one station - the boundary is cut twice: {x:?} ~ {y:?}"
            );
        }
    }
    assert!(
        path.notes.iter().any(|n| n.contains("boundary chain shared with band 1")),
        "the notes do not state the shared boundary: {:?}",
        path.notes
    );
}

#[test]
fn one_helical_chain_covers_the_full_revolution() {
    // The single-chain case (#2829): the near root edge alone, one
    // full turn. The kernel morphs it to its own next lap, so the
    // developed band is exactly one revolution at the chain's own
    // radius: ceil(R_ROOT * TAU / 2.0) = 26 spans, one pass each -
    // the picked chain at the opening edge, the closing pass dropped
    // because the lap's two edges are one world curve. The first
    // pass is the picked chain itself: one full lap wound.
    let facets = groove(N_CIRC, 1, 0.0);
    let mut pair = root_pair(N_CIRC, 1, 0.0);
    let alone = pair.swap_remove(0);
    let tool = ball(6.0);
    let mut q = params(&tool, Some(2.0));
    q.clearance = 20.0; // the wall-adjacent pass pushes past the 14 mm helper's read
    let path = rotary_flow(&[alone], &facets, &stated_axis(), &q, &AxisControl::default())
        .unwrap();
    no_gouge(&path, &facets, 3.0);
    let ls = landings(&path, 3.0);
    assert_eq!(
        ls.len(),
        26,
        "the full revolution did not divide into 26 passes: {:?}",
        path.notes
    );
    let centers = first_pass_centers(&path, 20.0, 3.0);
    let span = wound(&centers);
    assert!(
        (span - TAU).abs() < 0.05,
        "the first pass winds {span:.3} rad, not one full revolution"
    );
    assert!(
        path.notes.iter().any(|n| n.contains("morphed a full revolution")),
        "the notes do not state the lap morph: {:?}",
        path.notes
    );
}

#[test]
fn a_ring_that_crosses_a_corner_chain_is_refused_by_name() {
    // A closed hoop on the prism crosses the corner chain at its own
    // middle: the set law refuses the pair at the crossing, never
    // blends a band that folds through it.
    let facets = prism(12, R_ROOT, 0.0, 4.0);
    let corner: Vec<Point3> = (0..=4).map(|k| gp(R_ROOT, 0.0, k as f64)).collect();
    let hoop = ring(12, R_ROOT, 2.0);
    let tool = ball(2.0);
    let e = rotary_flow(
        &[corner, hoop],
        &facets,
        &stated_axis(),
        &params(&tool, Some(0.9)),
        &AxisControl::default(),
    )
    .unwrap_err();
    let msg = format!("{e}");
    assert!(msg.contains("meet or cross each other"), "{msg}");
    let at = e.location().expect("the refusal names its spot");
    assert!(
        (at.x - R_ROOT).abs() < 1e-6 && at.y.abs() < 1e-6 && (at.z - 2.0).abs() < 1e-6,
        "the refusal stands at {at:?}, not at the crossing"
    );
}

#[test]
fn a_crossing_between_non_adjacent_guides_is_refused_by_name() {
    // The #4032 reproduction (safety-tier review of PR #4018): an R8
    // prism of 72 sides, Z0..Z32. Guide 3 runs down to (0, 2) and
    // back up, so it meets guide 1 on its corner line at (8, 0, 2).
    // The adjacent pairs 1-2 and 2-3 never meet; only the complete
    // set law reads the crossing, and refuses it there.
    let n = 72;
    let d = TAU / n as f64;
    let facets = prism(n, R_ROOT, 0.0, 32.0);
    let g1 = vec![gp(R_ROOT, 0.0, 0.0), gp(R_ROOT, 0.0, 4.0)];
    let g2 = vec![gp(R_ROOT, 3.0 * d, 10.0), gp(R_ROOT, 3.0 * d, 14.0)];
    let g3 = vec![gp(R_ROOT, -d, 30.0), gp(R_ROOT, 0.0, 2.0), gp(R_ROOT, d, 30.0)];
    let tool = ball(2.0);
    let mut q = params(&tool, Some(0.9));
    q.clearance = 20.0;
    let e = rotary_flow(&[g1, g2, g3], &facets, &stated_axis(), &q, &AxisControl::default())
        .unwrap_err();
    let msg = format!("{e}");
    assert!(
        msg.contains("guide chains 1 and 3 meet or cross each other")
            && msg.contains("(8.000, 0.000, 2.000)"),
        "{msg}"
    );
    let at = e.location().expect("the refusal names its spot");
    assert!(
        (at.x - R_ROOT).abs() < 1e-6 && at.y.abs() < 1e-6 && (at.z - 2.0).abs() < 1e-6,
        "the refusal stands at {at:?}, not at the crossing"
    );
}

#[test]
fn a_clean_three_guide_set_on_the_prism_still_machines() {
    // Three corner-line guides that never meet, on the same prism:
    // guides 1 and 3 share no ground, and the complete set law lets
    // the two adjacent bands machine as before.
    let n = 72;
    let d = TAU / n as f64;
    let facets = prism(n, R_ROOT, 0.0, 32.0);
    let line = |th: f64| vec![gp(R_ROOT, th, 2.0), gp(R_ROOT, th, 30.0)];
    let tool = ball(2.0);
    let mut q = params(&tool, Some(0.9));
    q.clearance = 20.0;
    let path = rotary_flow(
        &[line(0.0), line(3.0 * d), line(6.0 * d)],
        &facets,
        &stated_axis(),
        &q,
        &AxisControl::default(),
    )
    .unwrap();
    no_gouge(&path, &facets, 1.0);
    assert!(!landings(&path, 1.0).is_empty(), "no pass landed: {:?}", path.notes);
    assert!(
        path.notes.iter().any(|n| n.contains("boundary chain shared with band 1")),
        "the notes do not state the shared boundary: {:?}",
        path.notes
    );
}

#[test]
fn reversing_only_the_middle_pick_preserves_the_cut_directions() {
    // The review's reproduction (PR #4018, first blocking item): the
    // one-turn groove, three root helices at axial offsets 0, 2 and
    // 4 mm, the middle one picked backwards. The set is ONE ordered
    // set: the winding of each guide is resolved once against its
    // neighbour, and the SAME normalized guide bounds both adjacent
    // bands - so every pass of every band holds the direction the
    // climb states, for BOTH climb settings, and the second band
    // never flips its cutting direction without a change to climb.
    let facets = groove(N_CIRC, 1, 0.0);
    let tool = ball(2.0);
    for climb in [true, false] {
        let mut q = params(&tool, Some(0.9));
        q.climb = climb;
        // The control: the aligned trio, every pass one direction.
        let aligned = rotary_flow(
            &root_trio(N_CIRC, 1, 0.0),
            &facets,
            &stated_axis(),
            &q,
            &AxisControl::default(),
        )
        .unwrap();
        // The mixed winding: only the middle pick is reversed.
        let mut picks = root_trio(N_CIRC, 1, 0.0);
        picks[1].reverse();
        let mixed = rotary_flow(&picks, &facets, &stated_axis(), &q, &AxisControl::default())
            .unwrap();
        let want = if climb { TAU } else { -TAU };
        for (name, path) in [("aligned", &aligned), ("mixed", &mixed)] {
            no_gouge(path, &facets, 1.0);
            if name == "mixed" {
                no_swept_gouge(path, &facets, 1.0);
            }
            let passes = every_pass(path, 14.0, 1.0);
            assert_eq!(
                passes.len(),
                7,
                "the {name} bands did not divide into 7 passes: {:?}",
                path.notes
            );
            for (i, p) in passes.iter().enumerate() {
                let span = wound(p);
                assert!(
                    (span - want).abs() < 0.05,
                    "pass {} of the {name} set winds {span:.3} rad at climb {climb}, not \
                     {want:.3} - a band changed its cutting direction",
                    i + 1
                );
            }
        }
        assert!(
            mixed
                .notes
                .iter()
                .any(|n| n.contains("winds against guide chain 1 - walked backwards")),
            "the notes do not state the backward-wound middle guide: {:?}",
            mixed.notes
        );
        assert!(
            aligned.notes.iter().all(|n| !n.contains("walked backwards")),
            "an aligned set states a reversal: {:?}",
            aligned.notes
        );
    }
}

#[test]
fn a_single_closed_ring_refuses_the_zero_area_lap_family() {
    // The review's reproduction (PR #4018, second blocking item): a
    // 72-sided R8 prism from Z0 to Z4, ONE closed ring at Z2, a 2 mm
    // ball, a 0.9 mm stepover. The lap morph reads a positive angular
    // width - one whole revolution - but a ring turned a lap sweeps
    // the same circle again at every pass, and the family repeats
    // one closed cut. The kernel refuses the degenerate guide by
    // name, before a pass is generated.
    let facets = prism(72, R_ROOT, 0.0, 4.0);
    let hoop = ring(72, R_ROOT, 2.0);
    let tool = ball(2.0);
    let e = rotary_flow(
        &[hoop],
        &facets,
        &stated_axis(),
        &params(&tool, Some(0.9)),
        &AxisControl::default(),
    )
    .unwrap_err();
    let msg = format!("{e}");
    assert!(msg.contains("guide chain 1"), "the refusal does not name the guide: {msg}");
    assert!(msg.contains("sweeps no band area"), "{msg}");
}

#[test]
fn a_single_open_ring_arc_refuses_the_zero_area_lap_family() {
    // The same refusal for the OPEN arc of a ring: an arc at one
    // axial station and one radius turns onto itself, and its lap
    // family repeats one arc of the same circle - no band area
    // either.
    let facets = prism(72, R_ROOT, 0.0, 4.0);
    let step = TAU / 72.0;
    let arc: Vec<Point3> = (0..=36)
        .map(|k| gp(R_ROOT, step * k as f64, 2.0))
        .collect();
    let tool = ball(2.0);
    let e = rotary_flow(
        &[arc],
        &facets,
        &stated_axis(),
        &params(&tool, Some(0.9)),
        &AxisControl::default(),
    )
    .unwrap_err();
    let msg = format!("{e}");
    assert!(msg.contains("sweeps no band area"), "{msg}");
}
