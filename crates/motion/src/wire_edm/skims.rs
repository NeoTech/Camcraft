//! Wire EDM skim passes (#538, part of #290): after the rough pass,
//! each skim walks the same profile again at its own condition's
//! offset (wire radius + that condition's spark gap), from the same
//! threading point, optionally in the other direction. The passes'
//! offsets step toward the profile: no pass may stand nearer the
//! profile than the finishing (last) pass - it would cut past the
//! finished profile - and none may stand farther out than the pass
//! before it, where it would cut only air. One threading cuts every
//! pass, so every skim's condition must state the rough pass's wire.
//! An open run (#3216 over #538) threads each pass anew in the air: a
//! travel move (a rapid, no wire through it) joins one pass's run-out
//! to the next pass's run-in and marks the pass boundary that the
//! verify and the post read each pass from.
//!
//! Provenance: general wire EDM practice (a rough cut followed by
//! skim cuts at decreasing offsets, the finishing pass setting the
//! size), as the public EDM literature #290 cites describes it; our
//! own code over the module's single-pass generator. No vendor
//! manual, condition table or CAM product behaviour was consulted.

use super::{cut_pass, invalid, WireParams, PATH_TOL};
use crate::cl::{MoveKind, Move, Toolpath};
use crate::geo2d::{Loop, Point2};
use crate::machining2d::GenError;

/// One skim pass, resolved from the machine's condition.
#[derive(Clone, Debug, PartialEq)]
pub struct WirePass {
    /// The condition's id, as the control selects it.
    pub condition: String,
    /// The wire's diameter the condition states, mm.
    pub wire_diameter: f64,
    /// The condition's spark gap per side, mm.
    pub gap: f64,
    /// Walk this pass clockwise (the rough pass runs counter-clockwise).
    pub reverse: bool,
}

impl WirePass {
    /// This pass's generator numbers: the rough's, with its own
    /// condition, wire and gap.
    fn params(&self, rough: &WireParams) -> WireParams {
        WireParams {
            wire_diameter: self.wire_diameter,
            gap: self.gap,
            condition: self.condition.clone(),
            ..rough.clone()
        }
    }
}

/// The passes' laws, by name, before any geometry is read - also the
/// command door's (the numbers are all the machine's).
///
/// # Errors
/// Skims with tabs, a skim naming no condition, a skim whose wire is not the rough
/// pass's, a pass nearer the profile than the finishing pass, or a
/// skim farther out than the pass before it. An open run takes skims
/// too (#3216): each is entered from the air at the run's start end
/// and walked in the run's own direction - a reversed one is refused
/// by the open-run generator.
pub fn check_passes(rough: &WireParams, skims: &[WirePass]) -> Result<(), GenError> {
    let Some(last) = skims.last() else {
        return Ok(());
    };
    if rough.tabs.is_some() {
        return Err(invalid(format!(
            "tabs are stated with {} skim pass(es) - skims are cut on an untabbed profile only (#538 x #539); remove the tabs or the skims",
            skims.len()
        )));
    }
    for (i, s) in skims.iter().enumerate() {
        if s.condition.trim().is_empty() {
            return Err(invalid(format!("skim {} names no EDM condition - every skim burns with a condition of its own", i + 1)));
        }
        s.params(rough).door()?;
    }
    let finish = last.params(rough).kerf_offset();
    if rough.kerf_offset() < finish - PATH_TOL {
        return Err(invalid(format!(
            "the rough pass ({}) stands {:.4} mm from the profile, nearer than the finishing pass's {finish:.4} mm - it would cut past the finished profile",
            rough.condition,
            rough.kerf_offset()
        )));
    }
    let mut before = ("the rough pass".to_owned(), rough.kerf_offset());
    for (i, s) in skims.iter().enumerate() {
        let who = format!("skim {} (condition {})", i + 1, s.condition);
        let p = s.params(rough);
        if (s.wire_diameter - rough.wire_diameter).abs() > PATH_TOL {
            return Err(invalid(format!(
                "{who} states a {:.4} mm wire, the rough pass ({}) a {:.4} mm wire - one threading cuts every pass with one wire",
                s.wire_diameter, rough.condition, rough.wire_diameter
            )));
        }
        let k = p.kerf_offset();
        if k < finish - PATH_TOL {
            return Err(invalid(format!(
                "{who} stands {k:.4} mm from the profile, nearer than the finishing pass's {finish:.4} mm - it would cut past the finished profile"
            )));
        }
        if k > before.1 + PATH_TOL {
            return Err(invalid(format!(
                "{who} stands {k:.4} mm from the profile, farther out than {} ({:.4} mm) - it would cut only air; order the skims toward the profile",
                before.0, before.1
            )));
        }
        before = (who, k);
    }
    Ok(())
}

/// The rough pass, then every skim, each from the threading point and
/// back to it at its own offset; every pass judged by the single-pass
/// laws at its offset. With no skims this is [`super::wire_contour`]
/// exactly. An open run (#3216) takes its skims through the open-run
/// generator: each re-enters from the air at the run's start end, at
/// its own run-in point, and is walked in the run's own direction -
/// the wire is threaded anew for every pass, and a travel move (a
/// rapid, no wire through it) joins one pass's run-out to the next
/// pass's run-in, marking the pass boundary.
///
/// # Errors
/// [`check_passes`]'s refusals, then any pass's own.
pub fn wire_passes(loops: &[Loop], chains: &[Vec<Point2>], rough: &WireParams, skims: &[WirePass]) -> Result<Toolpath, GenError> {
    check_passes(rough, skims)?;
    let open = rough.side.is_open_side();
    let mut path = cut_pass(loops, chains, rough, false)?;
    for (i, s) in skims.iter().enumerate() {
        let p = s.params(rough);
        let pass = cut_pass(loops, chains, &p, s.reverse)?;
        // An open run is threaded anew for every pass (#3216 over #538):
        // the pass before ended at its own run-out, the wire cut there,
        // and the next pass begins at its own run-in back at the run's
        // start end. A travel move carries the boundary - the wire is
        // not threaded through it - so the path stays one chained
        // program, and the verify and the post read each pass as its
        // own from the boundary the travel marks.
        if open {
            let (Some(from), Some(to)) = (path.end(), pass.start()) else {
                return Err(invalid("the open run lays down no cut".into()));
            };
            path.moves.push(Move::new(MoveKind::Rapid, from, to, None));
        }
        path.moves.extend(pass.moves);
        let how = if open {
            "in the run's own direction, threaded anew in the air at the run's start end".to_owned()
        } else if s.reverse {
            "clockwise (reversed)".to_owned()
        } else {
            "counter-clockwise".to_owned()
        };
        path.notes.push(format!(
            "wire EDM: skim {} - condition {} at {:.4} mm (wire radius {:.4} + spark gap {:.4}), cut {how}",
            i + 1,
            s.condition,
            p.kerf_offset(),
            p.wire_diameter * 0.5,
            p.gap,
        ));
    }
    Ok(path)
}
