//! Rotary Flow (issue #2824, the first slice of #2804): the
//! 4-axis cylindrical flow-line finishing kind. The caller PICKS
//! the guide chains - the edges of a helical band on a rotor - and
//! states the rotation axis (the #2772 seam: the axis is a
//! PARAMETER here, never fitted from the mesh, so a helical model
//! no fit could answer still machines). The kernel unwraps each
//! chain to a continuous cylindrical track about the axis
//! (`rotary_flow_unwrap`), aligns each adjacent pair by the nearest
//! whole turn, develops the band `(s = R_ref * theta, v)`, chooses
//! the pass count from the stepover or the scallop law, blends
//! iso-parametric flow curves between the pair, pushes every
//! station's ball center out along its radial until the
//! closest-point oracle clears the whole mesh (`rotary_flow_pass`),
//! and poses and links the passes radially on the retract cylinders
//! (`rotary_flow_emit`) - ball-center ladders, tool tips out.
//!
//! This first slice needed a ball-nose tool and clean picks: empty,
//! off-model, on-axis and degenerate picks are refused by name,
//! never guessed past, and a chain wound against its pair is walked
//! backwards (the feature layer reads a chain's direction from the
//! mesh, never from the pick). The #2829 slice reads the chains as
//! ONE ordered set: the complete set is wound one way before any
//! band is cut, and the same normalized guide bounds both adjacent
//! bands (`rotary_flow_set::normalize_set`); three or more machine
//! one band per adjacent pair with the SHARED boundary chain cut
//! once; a single chain machines the full revolution as its own
//! next lap, unless it turns at one station and one radius - a ring
//! whose lap sweeps no band area, refused by name; and a set that
//! cannot tile - any two chains that meet or cross (#4032), bands
//! that double back - is refused by name (`rotary_flow_set`,
//! `rotary_flow_cross`).
//!
//! Provenance: cylindrical unwrapping and iso-parametric flow-line
//! CAM on developed rotary surfaces from published 4-axis machining
//! literature; the scallop, lean and pivot laws are this codebase's
//! own. No vendor material consulted.

use crate::cl::Toolpath;
use crate::finishing3d::scallop_stepover;
use crate::geometry::{Point3, Triangle};
use crate::machining2d::{GenError, ARC_FIT_TOL};
use crate::tools::{Tool, ToolKind};

use super::axis::Axis;
use super::retip::radial_at;
use super::rotary_flow_cross::refuse_crossed_set;
use super::rotary_flow_emit::Emitter;
use super::rotary_flow_guard::prove_path;
use super::rotary_flow_pass::{boxes_of, flow_pass};
use super::rotary_flow_set::{
    band_shape, lap_of, meet_or_cross, normalize_set, refuse_backtrack, refuse_ring_lap,
};
use super::rotary_flow_unwrap::{self, Basis, Unwrapped, TAU};
use super::tilt::AxisControl;

#[cfg(test)]
mod rotary_flow_tests;

/// The corner-lattice step for chain points (the drive-curve walk's
/// own DUST law, curve.rs): two reads of one CAD vertex differ by
/// float dust, never by a facet's breadth.
const DUST: f64 = 1e-6;
/// Matched-place samples that read a band's width and alignment.
const BAND_SAMPLES: usize = 33;

/// The pass's numbers. Heights read as BALL-CENTER radii about the
/// stated axis - the retract cylinders the links ride - never as
/// world Z: a rotor may stand or lie, and only a radius about the
/// axis names a safe place the same way for both.
pub struct RotaryFlowParams<'a> {
    /// The cutting tool. Rotary Flow is a ball law: the pass spacing,
    /// the push-out and the tip all ride a ball of `tool.diameter`;
    /// a tool of any other kind is refused by name.
    pub tool: &'a Tool,
    /// Developed stepover between flow passes, mm, read at the
    /// band's outermost radius (ground inside it gets finer, never
    /// coarser). `None` uses the scallop law
    /// [`scallop_stepover`]`(ball_r, tolerance)`.
    pub stepover: Option<f64>,
    /// Scallop height and chord budget, mm.
    pub tolerance: f64,
    /// Ball-center radius of the clearance cylinder: the program
    /// opens, hops bands and exits there.
    pub clearance: f64,
    /// Ball-center radius of the pass-link cylinder; `None` links at
    /// clearance.
    pub retract: Option<f64>,
    pub feed: f64,
    pub plunge_feed: f64,
    /// `true` walks each band as picked; `false` reverses each
    /// pass's travel (the drive-curve walk's own climb law).
    pub climb: bool,
}

/// The Rotary Flow program: the picked guide chains of the work's
/// helical ground, machined by flow lines.
///
/// The chains are 3D polylines ON the model - the picked edge
/// chains of the selection, each running ALONG the band (a helix of
/// the rotor), picked in band order. One chain machines the full
/// revolution between it and its own next lap (#2829, a single-start
/// screw or an auger). Two chains span one band. Three or more
/// machine one band per adjacent pair, and the shared boundary chain
/// is cut once - the band above already walked it.
pub fn rotary_flow(
    chains: &[Vec<Point3>],
    facets: &[Triangle],
    ax: &Axis,
    p: &RotaryFlowParams<'_>,
    c: &AxisControl,
) -> Result<Toolpath, GenError> {
    let r = validate_numbers(p)?;
    validate_axis(ax)?;
    if facets.is_empty() {
        return Err(GenError::Empty(
            "Rotary Flow found no model to read the band from".into(),
        ));
    }
    validate_facets(facets)?;
    if chains.is_empty() {
        return Err(GenError::Invalid(
            "Rotary Flow needs at least one picked guide chain; got 0 - pick one helical edge \
             for a full revolution, or the two edges that bound the band"
                .into(),
        ));
    }
    let basis: Basis = rotary_flow_unwrap::basis(ax)?;
    let boxes = boxes_of(facets);
    let rho_mesh = rotary_flow_unwrap::mesh_radius(ax, facets);
    let hop = p.retract.unwrap_or(p.clearance);
    for (name, v) in [("clearance", p.clearance), ("retract", hop)] {
        if !(v.is_finite() && v > rho_mesh + r + ARC_FIT_TOL) {
            // #677's own words, a radius this time: no spot is named.
            return Err(GenError::Invalid(format!(
                "Rotary Flow reads its {name} height as a ball-center radius about the stated \
                 axis: {v:.3} mm does not clear the work's {rho_mesh:.3} mm radius plus the \
                 {r:.3} mm ball - raise it"
            )));
        }
    }
    let diag = rotary_flow_unwrap::extent(facets);
    let cell = (diag * DUST).max(1e-9);
    let eps2 = (4.0 * cell) * (4.0 * cell);
    let mut tracks: Vec<Unwrapped> = Vec::with_capacity(chains.len() + 1);
    let mut clean: Vec<Vec<Point3>> = Vec::with_capacity(chains.len());
    for (ci, ch) in chains.iter().enumerate() {
        let pts = rotary_flow_unwrap::clean_chain(ch, ci, eps2)?;
        for (j, q) in pts.iter().enumerate() {
            if !rotary_flow_unwrap::on_model(*q, facets, &boxes, eps2) {
                return Err(GenError::invalid_at(
                    format!(
                        "Rotary Flow: guide chain {} point {} stands off the model surface - \
                         re-pick it, or project it onto the part",
                        ci + 1,
                        j + 1
                    ),
                    *q,
                ));
            }
            if radial_at(ax, *q).is_none() {
                return Err(GenError::invalid_at(
                    format!(
                        "Rotary Flow: guide chain {} point {} stands on the rotation axis line \
                         - it has no azimuth to unwrap",
                        ci + 1,
                        j + 1
                    ),
                    *q,
                ));
            }
        }
        tracks.push(rotary_flow_unwrap::unwrap(ax, &basis, &pts));
        clean.push(pts);
    }
    // The #2829 set normalization, ONE pass over the complete
    // ordered set: each guide's winding is resolved against its
    // pair-neighbour before the first band is cut, and the SAME
    // normalized guide bounds both adjacent bands. Never per band:
    // a band-by-band copy leaves the next band's lower boundary at
    // the original winding, and the cutting direction flips between
    // bands without a change to climb (PR #4018).
    let mut notes: Vec<String> = Vec::new();
    for k in normalize_set(&mut tracks) {
        notes.push(format!(
            "Rotary Flow: guide chain {} winds against guide chain {} - walked backwards",
            k + 1,
            k
        ));
    }
    // The #2829 single-chain case: the chain's own next lap is the
    // second boundary, exact by construction - no arbitrary branch
    // to align away. The lap's two edges are the SAME world curve,
    // so the band closes on itself: the picked chain emits once, at
    // the band's opening edge, and the closing pass is dropped. A
    // chain that turns at one axial station and one radius - a
    // ring, closed or open - sweeps no band when turned a lap:
    // refused by name before a pass is generated (PR #4018).
    // The #4032 complete-set law: two guides that are NOT neighbours
    // in the pick never meet or cross - the band loop below reads
    // only the adjacent pairs. Refused by name before a band is cut.
    refuse_crossed_set(&clean, eps2)?;
    let lap = clean.len() == 1;
    if lap {
        refuse_ring_lap(&tracks[0], cell)?;
        tracks.push(lap_of(&tracks[0]));
    }
    // The passes: one band per adjacent chain pair, the shared
    // boundary of consecutive bands cut once, hop-linked inside a
    // band, clearance-linked between bands.
    let step = match p.stepover {
        Some(s) => s,
        None => scallop_stepover(r, p.tolerance),
    };
    let tol_chord = ARC_FIT_TOL.min(p.tolerance);
    let mut emit = Emitter::new(ax, &basis, c, r, p.clearance, p.feed, p.plunge_feed);
    let mut opened = false;
    let mut prev_last: Option<Point3> = None;
    let mut prev_dir: Option<(f64, f64)> = None;
    for k in 0..tracks.len() - 1 {
        let a = &tracks[k];
        let mut b = tracks[k + 1].clone();
        let shift = if lap && k == 0 {
            0.0
        } else {
            rotary_flow_unwrap::align_pair(a, &mut b, BAND_SAMPLES)
        };
        let r_ref = a
            .rho
            .iter()
            .chain(b.rho.iter())
            .fold(0.0f64, |m, &x| m.max(x));
        let width = rotary_flow_unwrap::cross_width(a, &b, r_ref, BAND_SAMPLES);
        if !(width.is_finite() && width > 0.0) {
            return Err(GenError::Invalid(format!(
                "Rotary Flow: guide chains {} and {} lie on top of each other - the band has no \
                 width to machine",
                k + 1,
                k + 2
            )));
        }
        if !lap {
            meet_or_cross(&clean[k], &clean[k + 1], k, eps2)?;
        }
        let (dir, count) = band_shape(a, &b, r_ref, width, step, BAND_SAMPLES)?;
        if let Some(prev) = prev_dir {
            refuse_backtrack(prev, dir, k)?;
        }
        prev_dir = Some(dir);
        // The shared boundary chain of band k > 0 is band k - 1's own
        // last pass: cut once, this band starts at its first
        // interior pass. A lap band's own two edges are one world
        // curve: it closes on itself, and the closing pass is
        // dropped - the picked chain emits once.
        let start = if k == 0 { 0 } else { 1 };
        let end = count - usize::from(lap);
        let mut band: Vec<Vec<Point3>> = Vec::with_capacity(end - start);
        let mut capped = false;
        for i in start..end {
            let t = i as f64 / (count - 1) as f64;
            let out = flow_pass(ax, &basis, facets, &boxes, a, &b, t, r, tol_chord)?;
            capped |= out.depth_capped;
            let mut centers = out.centers;
            if !p.climb {
                centers.reverse();
            }
            band.push(centers);
        }
        if capped {
            notes.push(format!(
                "Rotary Flow: a pass of band {} reached the chord depth cap at tolerance \
                 {tol_chord} mm - check the picks",
                k + 1
            ));
        }
        for (e, centers) in band.iter().enumerate() {
            let Some(first) = centers.first().copied() else { continue };
            if !opened {
                emit.open(first)?;
                opened = true;
            } else if e == 0 {
                // Between bands the hop is the clearance cylinder.
                let last = prev_last.ok_or_else(|| {
                    GenError::Invalid("Rotary Flow lost the last pass's end".into())
                })?;
                emit.exit(last)?;
                emit.link(last, first, p.clearance)?;
            } else {
                let last = prev_last.ok_or_else(|| {
                    GenError::Invalid("Rotary Flow lost the last pass's end".into())
                })?;
                emit.retract(last, hop)?;
                emit.link(last, first, hop)?;
            }
            emit.pass(centers)?;
            prev_last = centers.last().copied();
        }
        let span = if shift.abs() > 1e-9 {
            format!("{shift:.2} rad of whole-turn alignment")
        } else {
            "no whole-turn alignment".into()
        };
        let tail = if lap {
            format!("; one guide chain, morphed a full revolution ({:.3} rad) in theta", TAU)
        } else if k > 0 {
            format!("; the boundary chain shared with band {k} is cut once")
        } else {
            String::new()
        };
        notes.push(format!(
            "Rotary Flow: band {} spans {width:.3} mm developed at R {r_ref:.3} mm; {} flow \
             passes at a {step:.3} mm stepover, {span}{tail}",
            k + 1,
            band.len(),
        ));
    }
    if let Some(last) = prev_last {
        emit.exit(last)?;
    }
    let path = emit.finish(notes)?;
    // The proof over every emitted move - cuts, plunges, retracts,
    // links and the exit: safe stations do not prove a safe sweep.
    prove_path(&path, facets, &boxes, r)?;
    Ok(path)
}

/// The model, or the refusal that names its first non-finite facet:
/// the bounds and the distance oracle would read past a NaN.
fn validate_facets(facets: &[Triangle]) -> Result<(), GenError> {
    match facets
        .iter()
        .position(|f| !(f.a.is_finite() && f.b.is_finite() && f.c.is_finite()))
    {
        Some(i) => Err(GenError::Invalid(format!(
            "Rotary Flow: model facet {} carries a non-finite vertex - repair the model before \
             it is machined",
            i + 1
        ))),
        None => Ok(()),
    }
}

/// The tool and the numbers, or the refusal that names them.
fn validate_numbers(p: &RotaryFlowParams) -> Result<f64, GenError> {
    if p.tool.kind != ToolKind::BallNose {
        return Err(GenError::Invalid(format!(
            "Rotary Flow cuts with a ball-nose tool; the setup's tool is a {} - mount a \
             ball-nose tool, or finish the band with a kind that takes it",
            p.tool.kind
        )));
    }
    let r = p.tool.diameter * 0.5;
    if !(r.is_finite() && r > 0.0) {
        return Err(GenError::Invalid(format!(
            "Rotary Flow needs a finite positive ball diameter; the tool states {} mm",
            p.tool.diameter
        )));
    }
    for (name, v) in [("feed", p.feed), ("plunge feed", p.plunge_feed)] {
        if !v.is_finite() {
            return Err(GenError::Invalid(format!(
                "Rotary Flow needs a finite {name}, not {v}"
            )));
        }
    }
    if !(p.clearance.is_finite() && p.retract.is_none_or(f64::is_finite)) {
        return Err(GenError::Invalid(
            "Rotary Flow needs finite clearance and retract radii".into(),
        ));
    }
    if let Some(s) = p.stepover
        && !(s.is_finite() && s > 0.0)
    {
        return Err(GenError::Invalid(format!(
            "Rotary Flow needs a positive finite stepover, not {s} mm"
        )));
    }
    if !(p.tolerance.is_finite() && p.tolerance > 0.0) {
        return Err(GenError::Invalid(format!(
            "Rotary Flow needs a positive finite tolerance, not {} mm",
            p.tolerance
        )));
    }
    // The scallop ceiling, the finishing kinds' own law (params.rs): a
    // stepover a ball diameter apart leaves uncut ground, and the auto
    // law reaches a full diameter at half the ball radius.
    if let Some(s) = p.stepover
        && s >= p.tool.diameter
    {
        return Err(GenError::Invalid(format!(
            "Rotary Flow stepover {s} mm reaches the scallop ceiling: passes a ball diameter \
             ({} mm) apart leave uncut ground, not a cusp",
            p.tool.diameter
        )));
    }
    if p.tolerance >= r / 2.0 {
        return Err(GenError::Invalid(format!(
            "Rotary Flow tolerance {} mm reaches the scallop ceiling: at half the ball radius \
             ({:.3} mm) the scallop stepover is a full ball diameter",
            p.tolerance,
            r / 2.0
        )));
    }
    Ok(r)
}

/// The stated axis, or the refusal that names what is wrong with it.
fn validate_axis(ax: &Axis) -> Result<(), GenError> {
    if !ax.origin.is_finite() || !ax.dir.is_finite() {
        return Err(GenError::Invalid(
            "the stated rotation axis carries a non-finite point or direction".into(),
        ));
    }
    let l = ax.dir.dot(ax.dir).sqrt();
    if (l - 1.0).abs() > 1e-6 {
        return Err(GenError::Invalid(format!(
            "the stated rotation axis direction is not unit length ({l:.6}) - the stated-axis \
             resolver or the caller must normalize it"
        )));
    }
    Ok(())
}
