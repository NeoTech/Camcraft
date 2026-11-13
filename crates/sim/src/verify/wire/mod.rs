//! Wire EDM 2.5D verification (#534, part of #290 D7): a vertical
//! 2-axis wire cut judged on the program plane against the part's
//! sections through the whole height the wire passes.
//!
//! - **Kerf law.** The wire burns a slot of half-width `k` (wire radius
//!   plus spark gap) along every move, through every height between its
//!   guides. A move whose kerf comes nearer the product's section than
//!   `k` - or stands in it - is a gouge, named by its move.
//! - **Threading law.** The wire is threaded at the start hole through
//!   the full height: the product must stand clear of the threading
//!   point by `k` at every judged plane. A start hole that is not
//!   stated, or that the path does not start from, is refused by name.
//! - **Slug law.** The kerf region is subtracted from the material
//!   section (stock and part); the connected pieces enclosed by the cut
//!   are what drops free when the cut closes. A die keeps the material
//!   outside the cut, so its freed piece must hold none of the product;
//!   a punch's freed piece IS the product. A cut that frees nothing, or
//!   frees the wrong side, is refused by name.
//!
//! The product's section by the stated side: a punch's product is the
//! model. A die's product is the model too when the start hole stands
//! outside it (the model is a die plate, the opening a hole in it); when
//! the start hole stands IN the model, the model is the opening (the
//! slug's shape, #532's whole-model die) and the product is the stock
//! around it.
//!
//! Provenance: plain 2D distance and region geometry (point-segment
//! distance, polygon booleans and the disc sweep of `geo2d`, clipper2)
//! over the public wire EDM practice the umbrella #290 cites: the kerf
//! is the wire diameter plus the gap on both sides, the wire is
//! threaded through a start hole, and a closed cut frees the enclosed
//! region (die slug or punch). Our own code; no vendor post, control
//! manual or CAM product behaviour was consulted.

mod dist;
mod slug;
#[cfg(test)]
mod tests;

use crate::cl::Toolpath;
use crate::geo2d::{Point2, Region};

pub use slug::Slug;

/// How far (mm) the kerf may come under its offset to the product
/// before the kerf law calls it a gouge: 0.1 um - the generator builds
/// the path at the offset to 10 nm, the sections are facet-exact.
pub const KERF_TOL: f64 = 1e-4;
/// The least area (mm²) of product a freed piece may hold before the
/// slug law reads it as holding product: a numerical sliver is not a
/// part.
pub const SLUG_AREA_TOL: f64 = 1e-3;

/// Which side of the cut is kept - stated by the operation, never read
/// from the geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutSide {
    /// The opening is the product: the material outside the cut stays.
    Die,
    /// The enclosed piece is the product: it drops when the cut closes.
    Punch,
}

impl CutSide {
    #[must_use]
    pub fn word(self) -> &'static str {
        match self {
            CutSide::Die => "die",
            CutSide::Punch => "punch",
        }
    }
}

/// The material at one judged height: the part model's section and the
/// stock's (when the setup has one).
#[derive(Clone, Debug)]
pub struct WireSection {
    pub z: f64,
    pub part: Region,
    pub stock: Option<Region>,
}

/// One wire cut to judge.
#[derive(Clone, Debug)]
pub struct WireJob<'a> {
    pub path: &'a Toolpath,
    /// Wire radius plus spark gap (mm).
    pub kerf_offset: f64,
    pub side: CutSide,
    /// The operation's stated start hole; `None` is not stated.
    pub start: Option<Point2>,
    /// The sections through the height the wire passes, low to high.
    pub sections: Vec<WireSection>,
}

/// Every finding refuses: the wire laws have no warning tier.
#[derive(Clone, Debug, PartialEq)]
pub enum WireFinding {
    /// The path cannot be read by these laws (an arc, a posed or
    /// non-planar move, no moves, a bad offset).
    Unreadable(String),
    NoStart,
    /// The path does not start at the stated start hole.
    StartMoved { stated: Point2, path: Point2 },
    /// The product stands nearer the threading point than the kerf.
    Threading { z: f64, at: Point2, clear: f64, need: f64 },
    /// The kerf of move `move_no` (1-based) enters the product.
    Gouge { move_no: usize, z: f64, at: Point2, depth: f64, moves: usize },
    /// The closed cut frees no material.
    NothingFreed { z: f64 },
    /// The piece freed is not the one the stated side drops.
    WrongSlug { z: f64, side: CutSide, at: Point2, product_area: f64 },
}

impl WireFinding {
    /// The finding in words, mm.
    #[must_use]
    pub fn describe(&self) -> String {
        match self {
            WireFinding::Unreadable(why) => format!("the wire laws cannot read the path - {why}"),
            WireFinding::NoStart => {
                "no start hole is stated - the wire cannot be threaded, nothing is judged clear".into()
            }
            WireFinding::StartMoved { stated, path } => format!(
                "the path starts at X{:.3} Y{:.3}, not at the stated start hole X{:.3} Y{:.3} - regenerate it",
                path.x, path.y, stated.x, stated.y
            ),
            WireFinding::Threading { z, at, clear, need } => format!(
                "the start hole at X{:.3} Y{:.3} is not clear for threading at Z{z:.3}: the part stands {} - the wire and its gap need {need:.4} mm all round",
                at.x,
                at.y,
                if *clear <= 0.0 { "at the threading point itself".to_owned() } else { format!("{clear:.4} mm from it") }
            ),
            WireFinding::Gouge { move_no, z, at, depth, moves } => format!(
                "gouge: the kerf of move {move_no} enters the part by {depth:.4} mm near X{:.3} Y{:.3} at Z{z:.3} ({moves} move(s) gouge)",
                at.x, at.y
            ),
            WireFinding::NothingFreed { z } => format!(
                "the cut frees no material at Z{z:.3} - nothing drops when it closes (no closed cut, or no stock around the opening to judge the slug)"
            ),
            WireFinding::WrongSlug { z, side: CutSide::Die, at, product_area } => format!(
                "slug: the piece freed near X{:.3} Y{:.3} at Z{z:.3} holds {product_area:.3} mm² of the part - a die keeps the material outside the cut, but here the part drops",
                at.x, at.y
            ),
            WireFinding::WrongSlug { z, side: CutSide::Punch, at, .. } => format!(
                "slug: the piece freed near X{:.3} Y{:.3} at Z{z:.3} holds none of the part - a punch's cut frees the punch, but here only waste drops and the part stays in the skeleton",
                at.x, at.y
            ),
        }
    }
}

/// The verdict: every refusal, and the pieces the cut frees at the
/// lowest judged height that holds material (the report line's slug).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WireVerdict {
    pub findings: Vec<WireFinding>,
    pub slugs: Vec<Slug>,
}

/// The product's section at one height (module head): the model, or
/// the stock around it when the model is a die's opening.
pub(crate) enum Product<'a> {
    Part(&'a Region),
    /// Everything outside `opening`, within `stock` when there is one.
    Around { opening: &'a Region, stock: Option<&'a Region> },
}

impl Product<'_> {
    fn contains(&self, p: Point2) -> bool {
        match self {
            Product::Part(r) => r.contains(p),
            Product::Around { opening, stock } => !opening.contains(p) && stock.is_none_or(|s| s.contains(p)),
        }
    }

    fn boundary(&self) -> &Region {
        match self {
            Product::Part(r) => r,
            Product::Around { opening, .. } => opening,
        }
    }
}

/// Judge one wire cut by the three laws.
#[must_use]
pub fn judge(job: &WireJob<'_>) -> WireVerdict {
    let mut v = WireVerdict::default();
    let k = job.kerf_offset;
    if !(k.is_finite() && k > 0.0) {
        v.findings.push(WireFinding::Unreadable(format!("the kerf offset {k} is not a positive number")));
        return v;
    }
    let polyline = match polyline(job.path) {
        Ok(p) => p,
        Err(why) => {
            v.findings.push(WireFinding::Unreadable(why));
            return v;
        }
    };
    let Some(start) = job.start else {
        v.findings.push(WireFinding::NoStart);
        return v;
    };
    let first = polyline[0];
    if dist::pp(first, start) > 1e-6 {
        v.findings.push(WireFinding::StartMoved { stated: start, path: first });
    }
    // The die's reading of the model (module head), decided once for
    // the whole height: the start hole stands in the model at some plane.
    let opening = job.side == CutSide::Die && job.sections.iter().any(|s| s.part.contains(start));
    // The threading point's worst plane, and the first plane that gouges.
    let mut worst: Option<(f64, f64)> = None;
    let mut gouge: Option<WireFinding> = None;
    for s in &job.sections {
        let p = product(s, opening);
        let (clear, _) = dist::clearance(&p, start, start);
        if worst.is_none_or(|(c, _)| clear < c) {
            worst = Some((clear, s.z));
        }
        if gouge.is_none() {
            gouge = gouge_at(&p, &polyline, k, s.z);
        }
    }
    if let Some((clear, z)) = worst.filter(|(c, _)| *c < k - KERF_TOL) {
        v.findings.push(WireFinding::Threading { z, at: start, clear, need: k });
    }
    v.findings.extend(gouge);
    slug::judge(job, &polyline, opening, &mut v);
    v
}

fn product(s: &WireSection, opening: bool) -> Product<'_> {
    if opening {
        Product::Around { opening: &s.part, stock: s.stock.as_ref() }
    } else {
        Product::Part(&s.part)
    }
}

/// The first move whose kerf enters the product at this height, with
/// the count of moves that do.
fn gouge_at(p: &Product<'_>, pts: &[Point2], k: f64, z: f64) -> Option<WireFinding> {
    let mut first: Option<(usize, Point2, f64)> = None;
    let mut moves = 0;
    for (i, w) in pts.windows(2).enumerate() {
        let (d, at) = dist::clearance(p, w[0], w[1]);
        if d < k - KERF_TOL {
            moves += 1;
            if first.is_none() {
                first = Some((i + 1, at, k - d));
            }
        }
    }
    first.map(|(move_no, at, depth)| WireFinding::Gouge { move_no, z, at, depth, moves })
}

/// The path as the polyline the wire's centre walks: straight, planar,
/// unposed, contiguous moves only - anything else is not what these
/// laws can read, and is refused by name.
fn polyline(path: &Toolpath) -> Result<Vec<Point2>, String> {
    let Some(m0) = path.moves.first() else {
        return Err("the path has no moves".into());
    };
    let mut pts = vec![Point2::new(m0.from.x, m0.from.y)];
    for (i, m) in path.moves.iter().enumerate() {
        let n = i + 1;
        if m.arc.is_some() || m.axis.is_some() {
            return Err(format!("move {n} is an arc or states a pose - the 2-axis wire laws read straight vertical moves only"));
        }
        let finite = [m.from.x, m.from.y, m.from.z, m.to.x, m.to.y, m.to.z].iter().all(|c| c.is_finite());
        if !finite || (m.from.z - m0.from.z).abs() > 1e-9 || (m.to.z - m0.from.z).abs() > 1e-9 {
            return Err(format!("move {n} leaves the programmed plane"));
        }
        let from = Point2::new(m.from.x, m.from.y);
        if pts.last().is_some_and(|l| dist::pp(*l, from) > 1e-6) {
            return Err(format!("move {n} does not start where the move before it ended"));
        }
        pts.push(Point2::new(m.to.x, m.to.y));
    }
    Ok(pts)
}
