//! The rotary kinds' radial heights (#2774): a height on a turned part is a
//! RADIUS about the fitted rotation axis, not a world-Z plane - the tool must
//! keep its distance from the axis all the way round.
//!
//! The stored truth is the expression ([`RadialHeight`]); this module is its
//! one resolve. It fits the SAME axis the generators pose about
//! ([`camcraft_motion::multiaxis::axis::fit`] on the source facets), reads
//! the part's and the setup stock's largest radii about that line, and adds
//! the stored offset. A source that is not available (a stock radius with no
//! stock) refuses by name, never a guessed zero.
//!
//! Provenance: our own wiring of the domain model; radii about a fitted axis
//! are elementary geometry. No vendor material consulted.

use flecs_ecs::prelude::World;

use crate::geometry::{Point3, Triangle};
use crate::multiaxis::axis::{self, Axis};

use super::types::{OpParams, RadialHeight, RadialSource};

/// The four radial heights of one rotary op, resolved to millimetres.
/// `None` on a field means the op still runs the Z-plane law there.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ResolvedRadial {
    pub clearance: Option<f64>,
    pub retract: Option<f64>,
    pub top: Option<f64>,
    pub bottom: Option<f64>,
}

impl ResolvedRadial {
    /// Whether any field is radial - the op's switch from Z planes.
    #[must_use]
    pub fn any(&self) -> bool {
        self.clearance.is_some() || self.retract.is_some() || self.top.is_some() || self.bottom.is_some()
    }
}

impl From<ResolvedRadial> for crate::multiaxis::RadialHeights {
    fn from(r: ResolvedRadial) -> Self {
        crate::multiaxis::RadialHeights {
            clearance: r.clearance,
            retract: r.retract,
            top: r.top,
            bottom: r.bottom,
        }
    }
}

/// The radial fields an `OpParams` states, in the fixed order clearance,
/// retract, top, bottom, or `None` for every other kind. The panel and the
/// viewport read the same four slots the gate and the generators do.
pub fn radial_fields(params: &OpParams) -> Option<[Option<RadialHeight>; 4]> {
    match params {
        OpParams::RotaryContour { clearance_r, retract_r, top_r, bottom_r, .. }
        | OpParams::RotaryParallel { clearance_r, retract_r, top_r, bottom_r, .. } => {
            Some([*clearance_r, *retract_r, *top_r, *bottom_r])
        }
        OpParams::RotaryOffset { clearance_r, top_r, .. } => {
            Some([*clearance_r, None, *top_r, None])
        }
        _ => None,
    }
}

/// The largest perpendicular distance of `facets`' vertices from the line
/// `ax`. None when the mesh holds no finite vertex.
#[must_use]
pub fn rho_max(facets: &[Triangle], ax: &Axis) -> Option<f64> {
    let mut max: Option<f64> = None;
    for t in facets {
        for q in [t.a, t.b, t.c] {
            let d = q - ax.origin;
            let v = d.dot(ax.dir);
            let w = Point3::new(d.x - ax.dir.x * v, d.y - ax.dir.y * v, d.z - ax.dir.z * v);
            let rho = w.dot(w).sqrt();
            if rho.is_finite() {
                max = Some(max.map_or(rho, |m: f64| m.max(rho)));
            }
        }
    }
    max
}

/// Resolve one height against the part and stock radii. A relative source
/// with no stock refuses by name (the panel's wording for a missing stock).
/// The panel and the viewport read this too: the number on screen is the
/// number the generator moves to, never a second arithmetic.
pub fn resolve_one(
    name: &str,
    h: RadialHeight,
    part_rho: f64,
    stock_rho: Option<f64>,
) -> Result<f64, String> {
    let base = match h.source {
        RadialSource::Absolute => 0.0,
        RadialSource::ModelMaxRadius => part_rho,
        RadialSource::StockOd => stock_rho.ok_or_else(|| {
            format!(
                "{name} reads its radius from the stock OD, but no stock is set - set it up in Setup > Stock, or choose another source"
            )
        })?,
    };
    let r = base + h.offset;
    if r.is_finite() {
        Ok(r)
    } else {
        Err(format!("{name} resolves to a radius that is not finite"))
    }
}

/// The two geometry radii the radial heights read, for an operation's own
/// source in its setup, with the line they are measured about - the same
/// axis the generators pose about, so the viewport ring and the toolpath
/// answer for one and the same line.
pub struct RadialContext {
    pub axis: Axis,
    pub part_rho: f64,
    pub stock_rho: Option<f64>,
}

/// Read the source and stock facets and take the axis line the radii are
/// measured about - the SAME axis generation turns about (#2772): a stated
/// rotation axis (the op's own direction, else the mounted machine's
/// stated rotary lines, each anchored through the work offset origin) is
/// judged against the source's faces exactly as `generate` judges it; a
/// work with nothing stated keeps the free fit. The viewport ring and the
/// toolpath answer for one and the same line.
pub fn radial_context(
    world: &World,
    setup: Option<&str>,
    src: &super::types::SrcRef,
    params: &OpParams,
) -> Result<RadialContext, String> {
    radial_context_about(world, setup, src, crate::rotary_stated::op_stated(params))
}

/// [`radial_context`] from the op's own statement alone - what the
/// viewport's ring overlay reads of the open draft (a display-only read of
/// the same resolver; the machine's lines and the offset are read from the
/// world here too).
pub fn radial_context_about(
    world: &World,
    setup: Option<&str>,
    src: &super::types::SrcRef,
    stated: Option<[f64; 3]>,
) -> Result<RadialContext, String> {
    let part = super::material::source_facets(world, setup, src)?;
    let ax = match crate::rotary_stated::axis_lines_of(world, setup, stated) {
        Some(Ok(lines)) => crate::multiaxis::axis_stated::judged_any(&part, &lines).map_err(|e| e.to_string())?,
        Some(Err(e)) => return Err(e.to_string()),
        None => axis::fit(&part).map_err(|e| e.to_string())?,
    };
    let part_rho = rho_max(&part, &ax).ok_or_else(|| "the model has no radius about its axis".to_string())?;
    let stock = crate::model::find_by_name(world, &crate::setup::stock_entity_name(world, setup))
        .and_then(|e| crate::model::world_facets(world, &e));
    let stock_rho = stock.as_deref().and_then(|s| rho_max(s, &ax)).filter(|r| *r > 0.0);
    Ok(RadialContext { axis: ax, part_rho, stock_rho })
}

/// Resolve every radial height of `params` against the world. A non-rotary
/// kind or an op with no radial field returns all `None` without a world read.
pub fn resolve_radial(
    world: &World,
    setup: Option<&str>,
    src: Option<&super::types::SrcRef>,
    params: &OpParams,
) -> Result<ResolvedRadial, String> {
    let Some([clearance, retract, top, bottom]) = radial_fields(params) else {
        return Ok(ResolvedRadial::default());
    };
    if clearance.is_none() && retract.is_none() && top.is_none() && bottom.is_none() {
        return Ok(ResolvedRadial::default());
    }
    let src = src.ok_or_else(|| "the operation has no source to read its radii about".to_string())?;
    let c = radial_context(world, setup, src, params)?;
    let one = |name: &str, h: Option<RadialHeight>| -> Result<Option<f64>, String> {
        h.map(|h| resolve_one(name, h, c.part_rho, c.stock_rho)).transpose()
    };
    Ok(ResolvedRadial {
        clearance: one("Clearance radius", clearance)?,
        retract: one("Retract radius", retract)?,
        top: one("Top radius", top)?,
        bottom: one("Bottom radius", bottom)?,
    })
}

/// The radial gate (#2774): every SET radial field is validated before the
/// parameters are stored - its source resolves (a stock radius with no
/// stock names the setup page), the resolved radius is a distance above
/// the axis, and a clearance or retract radius that stands inside the
/// stock's or the part's own radius about the fitted axis would drive the
/// tool through metal on a rapid. The cut band must also order: the top
/// cylinder above the bottom one (and above the axis when it stands
/// alone), and the top under the clearance cylinder. Refused by name at
/// the command, in the generators' own words. Open when no radial field is
/// set, or when the axis itself cannot be read (the generator names that
/// one), so an old Z-plane op and a non-turning model never gain a new
/// wall.
pub fn radial_gate(world: &World, setup: Option<&str>, src: &super::types::SrcRef, params: &OpParams) -> Option<String> {
    let [clearance, retract, top, bottom] = radial_fields(params)?;
    if clearance.is_none() && retract.is_none() && top.is_none() && bottom.is_none() {
        return None;
    }
    let c = radial_context(world, setup, src, params).ok()?;
    let floor = c.part_rho.max(c.stock_rho.unwrap_or(0.0));
    // Every SET field resolves and stands in range before anything is
    // stored: a resolution failure (a stock radius with no stock, a
    // result that is not finite) refuses by name - never silently
    // skipped - and a resolved radius is a distance above the axis.
    let mut resolved = [None; 4];
    for (i, (name, h)) in
        [("Clearance radius", clearance), ("Retract radius", retract), ("Top radius", top), ("Bottom radius", bottom)]
            .into_iter()
            .enumerate()
    {
        let Some(h) = h else { continue };
        let v = match resolve_one(name, h, c.part_rho, c.stock_rho) {
            Ok(v) => v,
            Err(why) => return Some(why),
        };
        if let Some(why) = radius_range(name, v) {
            return Some(why);
        }
        resolved[i] = Some(v);
    }
    let (cl_v, re_v, t_v, b_v) = (resolved[0], resolved[1], resolved[2], resolved[3]);
    for (name, v) in [("Clearance radius", cl_v), ("Retract radius", re_v)] {
        if let Some(v) = v
            && v < floor
        {
            return Some(format!(
                "{name} {v:.3} mm stands inside the work's {floor:.3} mm radius about the bar's axis - raise it past the stock"
            ));
        }
    }
    if let (Some(t), Some(b)) = (t_v, b_v)
        && t <= b
    {
        return Some(format!(
            "Top radius {t:.3} mm stands below or at Bottom radius {b:.3} mm - raise the top or lower the bottom"
        ));
    }
    // The band must sit under the rapids: a top radius at or above the
    // clearance radius would put a cut where the hop travels.
    if let (Some(t), Some(cl)) = (t_v, cl_v)
        && t >= cl
    {
        return Some(format!(
            "Top radius {t:.3} mm stands at or above Clearance radius {cl:.3} mm - the rapids would run through the band - lower the top or raise the clearance"
        ));
    }
    // The one-sided band's own range: a top radius with no bottom states
    // the band 0..top, which is empty unless the top stands over the axis.
    if let Some(t) = t_v
        && bottom.is_none()
        && t <= 0.0
    {
        return Some(format!(
            "Top radius {t:.3} mm stands at or under the bar's axis - the band it states is empty - raise it"
        ));
    }
    None
}

/// A resolved radius is a distance above the axis: a negative one is no
/// radius at all. Zero stands (a band floor at the axis is the checkbox
/// default); a negative radius refuses by name.
fn radius_range(name: &str, v: f64) -> Option<String> {
    if v.is_finite() && v >= 0.0 {
        None
    } else if v.is_finite() {
        Some(format!(
            "{name} {v:.3} mm stands under the bar's axis - a radius is a distance above it, never negative"
        ))
    } else {
        Some(format!("{name} resolves to a radius that is not finite"))
    }
}
