//! The wire laws' pass split (#538 over #534): a skimmed cut is the
//! rough pass and its skims, each walked from the threading point and
//! back to it, each burning the kerf of its own condition (wire radius
//! plus that condition's spark gap). Each move is judged at its own
//! pass's kerf offset - the real half-width that pass burns, never a
//! narrower one - so a skim's nearer offset is not misread as the rough
//! kerf entering the part, and a skim that does come too near still is.
//! Provenance: public wire EDM practice (a rough cut, then skims at
//! decreasing offsets, each offset the wire radius plus that pass's
//! gap); our own code.

use super::dist;
use crate::geo2d::Point2;

/// One pass of the wire path: its moves `first..first + len` (0-based)
/// as a polyline, and the kerf offset it burns.
#[derive(Clone, Debug, PartialEq)]
pub(super) struct Pass {
    pub pts: Vec<Point2>,
    pub first: usize,
    pub k: f64,
}

/// How near (mm) a move's end must come to the threading point to
/// close a pass: the path ends each pass exactly on it.
const RETURN_TOL: f64 = 1e-6;

/// Split `pts` (the wire centre's polyline) into its passes. `k` is
/// the rough pass's kerf offset, `skims` each later pass's, in cut
/// order. With no skims the whole path is one pass at `k`, as before.
///
/// # Errors
/// A skim offset that is not a positive number, or a path that does
/// not return to its threading point once per pass - the moves of a
/// pass cannot then be told apart, so nothing is judged at a guess.
pub(super) fn split(pts: &[Point2], k: f64, skims: &[f64]) -> Result<Vec<Pass>, String> {
    if skims.is_empty() {
        return Ok(vec![Pass { pts: pts.to_vec(), first: 0, k }]);
    }
    if let Some(bad) = skims.iter().find(|s| !(s.is_finite() && **s > 0.0)) {
        return Err(format!("a skim's kerf offset {bad} is not a positive number"));
    }
    let Some(&thread) = pts.first() else {
        return Err("the path has no moves".into());
    };
    let offsets: Vec<f64> = std::iter::once(k).chain(skims.iter().copied()).collect();
    let mut out: Vec<Pass> = Vec::with_capacity(offsets.len());
    let mut cur = vec![thread];
    let mut first = 0;
    for (i, &q) in pts.iter().enumerate().skip(1) {
        cur.push(q);
        if dist::pp(q, thread) <= RETURN_TOL {
            let Some(&pk) = offsets.get(out.len()) else {
                return Err(format!("the path returns to its threading point more than once per pass ({} passes)", offsets.len()));
            };
            out.push(Pass { pts: std::mem::replace(&mut cur, vec![thread]), first, k: pk });
            first = i;
        }
    }
    if cur.len() > 1 || out.len() != offsets.len() {
        return Err(format!(
            "the path returns to its threading point {} time(s), not once per pass ({} passes) - regenerate it",
            out.len(),
            offsets.len()
        ));
    }
    Ok(out)
}

/// The largest kerf offset any pass burns - the clearance the
/// threading point needs, whichever pass is threaded.
pub(super) fn widest(passes: &[Pass]) -> f64 {
    passes.iter().map(|p| p.k).fold(0.0, f64::max)
}
