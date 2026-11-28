//! #3643: the cutting parameters and the judged holder body of a placed
//! spindle assembly. The body is the revolved envelope the collision
//! laws already read (`HolderProfile`): one body is exactly its spun
//! profile's bands, as a library holder is judged today; several are
//! the union of their bands, the larger radius wins in each span.
//! Elementary interval algebra; our own design.

use super::{AssemblyBody, AssemblyError, AssemblyNode, CuttingParams, AXIS_TOL};
use crate::component::{ComponentKind, CuttingItem};
use crate::frame::Frame;
use crate::holder_profile::{HolderBand, HolderProfile};
use crate::insert_iso1832::Corner;
use crate::spun::{SpunPoint, SpunProfile};
use crate::tools::Tool;

/// The cutting diameter and corner radius: a solid tool's own, or an
/// insert's from where its seat puts its cutting point (twice the
/// distance from the axis) and its corner (the nose radius, half a
/// round insert's diameter, 0 for a wiper facet).
pub(super) fn cutting(
    n: &AssemblyNode,
    frame: &Frame,
    cip: &Frame,
) -> Result<(CuttingParams, Option<Tool>), AssemblyError> {
    let name = n.component.name().to_owned();
    match n.component.kind() {
        ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) => {
            if !frame.on_axis(AXIS_TOL) || !cip.on_axis(AXIS_TOL) {
                return Err(AssemblyError::OffAxis { name });
            }
            Ok((CuttingParams { diameter: tool.diameter, corner_r: tool.corner_r }, Some((**tool).clone())))
        }
        ComponentKind::CuttingItem(CuttingItem::Insert(spec)) => {
            let o = cip.origin();
            let r = o[0].hypot(o[1]);
            if !r.is_finite() || r <= AXIS_TOL {
                let why = "its cutting point is on the spindle axis, so it cuts no diameter".to_owned();
                return Err(AssemblyError::BadCutter { name, why });
            }
            let corner_r = match spec.corner {
                Corner::Round => f64::from(spec.size) * 0.5,
                Corner::Facet { .. } => 0.0,
                c => c.nose_radius_mm().unwrap_or(0.0),
            };
            if corner_r > r {
                let why = format!("its corner radius {corner_r} mm is larger than its cutting radius {r:.3} mm");
                return Err(AssemblyError::BadCutter { name, why });
            }
            Ok((CuttingParams { diameter: 2.0 * r, corner_r }, None))
        }
        _ => Err(AssemblyError::NoCutter),
    }
}

/// The judged body of every component but the cutting item. A body
/// must stand on the axis; each is checked as a drawn spun profile at
/// its place in the tool frame (so one below the tip, or past the size
/// caps, is refused by name); and the whole must stay out of a solid
/// tool's flutes.
pub(super) fn judged(
    nodes: &[AssemblyNode],
    frames: &[Frame],
    cut: usize,
    tool: Option<&Tool>,
) -> Result<Option<AssemblyBody>, AssemblyError> {
    let mut placed: Vec<SpunProfile> = Vec::new();
    for (i, (n, f)) in nodes.iter().zip(frames).enumerate() {
        let Some(b) = n.geometry.body().filter(|_| i != cut) else { continue };
        if !f.on_axis(AXIS_TOL) {
            return Err(AssemblyError::OffAxis { name: n.component.name().to_owned() });
        }
        let at = f.origin()[2] + b.base();
        let points: Vec<SpunPoint> =
            b.profile().points().iter().map(|p| SpunPoint { z: p.z + at, ..*p }).collect();
        placed.push(SpunProfile::new(points).map_err(|e| AssemblyError::body(n, e))?);
    }
    let body = match placed.as_slice() {
        [] => return Ok(None),
        [one] => AssemblyBody { profile: one.holder_profile(), spun: Some(one.clone()) },
        many => {
            let bands = union(many.iter().flat_map(SpunProfile::bands).collect());
            let profile = HolderProfile::from_bands(bands);
            // `spun` is only the drawn outline for display. If the united
            // bands cannot be drawn, the outline is left out (`None`); the
            // judged `profile` that the laws read is kept either way.
            AssemblyBody { spun: SpunProfile::from_bands(profile.bands()), profile }
        }
    };
    if let Some(t) = tool
        && body.profile.enclose(t).is_none()
    {
        return Err(AssemblyError::BodyInFlutes { bottom: body.profile.bottom(), flute_len: t.flute_len });
    }
    Ok(Some(body))
}

/// The union of `bands` as contiguous bands bottom-up: each span takes
/// the largest radius over it. A span no body covers (two bodies that
/// do not touch) takes the smaller radius of its neighbours - the
/// child's shank or spigot runs through it - so the bands stay
/// contiguous, as the laws read them.
fn union(bands: Vec<HolderBand>) -> Vec<HolderBand> {
    let mut edges: Vec<f64> = bands.iter().flat_map(|b| [b.above, b.above + b.len]).collect();
    edges.sort_by(f64::total_cmp);
    edges.dedup_by(|b, a| (*b - *a).abs() <= 1e-9);
    let mut out: Vec<(HolderBand, bool)> = Vec::new();
    for w in edges.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        let mid = 0.5 * (lo + hi);
        let r = bands.iter().filter(|b| b.above < mid && mid < b.above + b.len).map(|b| b.r).reduce(f64::max);
        out.push((HolderBand { above: lo, len: hi - lo, r: r.unwrap_or(0.0) }, r.is_some()));
    }
    for i in 0..out.len() {
        if out[i].1 {
            continue;
        }
        let below = out[..i].iter().rev().find(|b| b.1).map(|b| b.0.r);
        let above = out[i + 1..].iter().find(|b| b.1).map(|b| b.0.r);
        out[i].0.r = match (below, above) {
            (Some(a), Some(b)) => a.min(b),
            (a, b) => a.or(b).unwrap_or(0.0),
        };
    }
    out.into_iter().map(|(b, _)| b).collect()
}
