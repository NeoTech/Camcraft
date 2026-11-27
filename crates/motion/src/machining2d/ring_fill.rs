//! The fill rings of the one ring set (#3788). Where the offsets of two
//! sides of a part merge (the waist of a figure eight, the neck of a dog
//! bone), a ring corner stands more than one stepover from the corner of
//! the next ring, and a ring there cuts the full tool width. A fill ring
//! is the same distance curve at half (and a quarter) of a step between
//! the two rings, cut only where they leave stock that a flank of either
//! one would meet: a level curve of the distance from the part, so it
//! crosses no ring, and Inside and Outside walk the same fills.
//!
//! Provenance: contour-parallel offset curves as level sets of the
//! distance to the boundary (Held, "On the Computational Geometry of
//! Pocket Machining", Springer LNCS 500, 1991), with intermediate levels
//! where the offsets merge; the zone test is ours (the flanks law of
//! `level_flanks`). No vendor material consulted.

use std::collections::BTreeMap;

use crate::geo2d::{Point2, Region, RingClip};

use super::face::GenError;
use super::level_flanks::{Material, full_width_flanks};
use super::level_order::{Cut, Piece, samples};
use super::ring::dist;

/// The sub-levels of one ring: ring `j` is level `RING * j`, its fills
/// the levels between `RING * (j - 1)` and `RING * j` (up to 1/16 step).
pub(super) const RING: u32 = 16;

/// How far (in tool radii) a flank point stands from a walk.
const FLANK: f64 = 0.9;

/// Conservative earlier-cut reach (in tool radii) when checking whether
/// a ring corner still needs a fill: gives margin over discrete meters.
const CUT_MARGIN: f64 = 0.95;

/// A zone smaller than this (mm2, after a 10 um erosion) needs no fill.
const ZONE_MIN: f64 = 0.002;

/// Fills shorter than this (mm) are dropped.
const MIN_FILL: f64 = 0.5;

/// Clipper integer lattice micro-steps shorter than this (mm, half of
/// `ring_side::SNAP`) are collapsed inside a walk so lattice jitter does
/// not flip the segment tangent.
const MICRO_SEG: f64 = 5e-5;

/// Half-width (mm) of a patch added around a full-width flank hit so the
/// clipped fill extends past both flanks into already-cut material.
const HIT_PAD: f64 = 0.2;

/// One band of the ring set: the ring `j` between the distance curves
/// `inner` (ring `j - 1`) and `outer` (ring `j`), with `hc` the part's
/// centre keep-out (distance 0), `c` the cutter-centre area, and `stock`
/// the stock footprint.
pub(super) struct Band<'a> {
    pub(super) c: &'a Region,
    pub(super) hc: &'a Region,
    pub(super) stock: Option<&'a Region>,
    pub(super) j: u32,
    pub(super) step: f64,
    pub(super) tool_r: f64,
    pub(super) climb: bool,
}

/// Drops Clipper lattice micro-steps (`< MICRO_SEG`) inside a walk while
/// keeping both endpoints of an open piece unchanged so `share_ends` keeps
/// exact boundary intersections.
pub(super) fn clean_walk(pts: Vec<Point2>, closed: bool) -> Vec<Point2> {
    if pts.len() <= 2 {
        return pts;
    }
    let Some(&first) = pts.first() else { return pts };
    let Some(&last) = pts.last() else { return pts };
    let mut out = Vec::with_capacity(pts.len());
    out.push(first);
    let end = if closed { pts.len() } else { pts.len() - 1 };
    for &p in &pts[1..end] {
        if let Some(&prev) = out.last()
            && dist(prev, p) >= MICRO_SEG
        {
            out.push(p);
        }
    }
    if closed {
        while out.len() > 2 && out.last().is_some_and(|&p| dist(p, first) < MICRO_SEG) {
            out.pop();
        }
    } else {
        while out.len() > 1 && out.last().is_some_and(|&p| dist(p, last) < MICRO_SEG) {
            out.pop();
        }
        out.push(last);
    }
    out
}

struct FillCtx<'a> {
    band: &'a Band<'a>,
    material: &'a Material<'a>,
    part_wall: &'a [Cut],
    outer_edge: &'a [Cut],
}

/// The fills of `band`, given the pieces of its two rings: none where
/// each flank of each ring meets the other ring's swath (the common
/// case), and recursive binary fills (up to `1 / RING` step) where a
/// gap or a full-width flank remains in either Inside or Outside order.
pub(super) fn fills(
    band: &Band<'_>,
    material: &Material<'_>,
    part_wall: &[Cut],
    outer_edge: &[Cut],
    inner: &[Vec<Point2>],
    outer: &[Vec<Point2>],
) -> Result<Vec<Piece>, GenError> {
    let mut out = Vec::new();
    let j = band.j;
    if j == 0 || inner.is_empty() || outer.is_empty() {
        return Ok(out);
    }
    let lo_lev = RING.saturating_mul(j - 1);
    let hi_lev = RING.saturating_mul(j);
    let mut levels: BTreeMap<u32, Vec<(Vec<Point2>, Cut)>> = BTreeMap::new();
    levels.insert(lo_lev, inner.iter().cloned().map(|w| (w.clone(), Cut::new(w))).collect());
    levels.insert(hi_lev, outer.iter().cloned().map(|w| (w.clone(), Cut::new(w))).collect());
    let ctx = FillCtx { band, material, part_wall, outer_edge };
    split_band(&ctx, &mut levels, lo_lev, hi_lev, 0, &mut out)?;
    Ok(out)
}

fn split_band(
    ctx: &FillCtx<'_>,
    levels: &mut BTreeMap<u32, Vec<(Vec<Point2>, Cut)>>,
    lo_lev: u32,
    hi_lev: u32,
    depth: u32,
    out: &mut Vec<Piece>,
) -> Result<(), GenError> {
    if hi_lev - lo_lev < 2 {
        return Ok(());
    }
    let band = ctx.band;
    let d_at = |lev: u32| f64::from(lev) * band.step / f64::from(RING);
    let lo_reg = if lo_lev == 0 { band.hc.clone() } else { band.hc.offset(d_at(lo_lev))? };
    let ring = band.hc.offset(d_at(hi_lev))?.difference(&lo_reg)?.intersect(band.c)?;
    let lo_walks: Vec<Vec<Point2>> = levels.range(..=lo_lev).flat_map(|(_, v)| v.iter().map(|(w, _)| w.clone())).collect();
    let hi_walks: Vec<Vec<Point2>> = levels.range(hi_lev..).flat_map(|(_, v)| v.iter().map(|(w, _)| w.clone())).collect();
    let mut zone = if depth == 0 {
        gap(&ring, &lo_walks, &hi_walks, band.tool_r)?
    } else {
        let swept = Region::sweep(&lo_walks, band.tool_r)?.union(&Region::sweep(&hi_walks, band.tool_r)?)?;
        ring.difference(&swept)?
    };
    let hits = merge_flank_hits(ctx, levels, lo_lev, hi_lev);
    if !hits.is_empty() {
        let hit_zone = Region::sweep(&hits, HIT_PAD)?;
        zone = if zone.is_empty() { hit_zone } else { zone.union(&hit_zone)? };
    }
    if zone.offset(-0.01)?.area().abs() <= ZONE_MIN {
        return Ok(());
    }
    let mid_lev = lo_lev + (hi_lev - lo_lev) / 2;
    let reach = if hits.is_empty() { band.tool_r } else { 2.0 * band.tool_r };
    let half = level(band, d_at(mid_lev), &zone, reach, mid_lev)?;
    if half.is_empty() {
        return Ok(());
    }
    let mid_walks: Vec<(Vec<Point2>, Cut)> = half
        .iter()
        .map(closed_walk)
        .map(|w| (w.clone(), Cut::new(w)))
        .collect();
    out.extend(half);
    levels.entry(mid_lev).or_default().extend(mid_walks);
    split_band(ctx, levels, lo_lev, mid_lev, depth + 1, out)?;
    split_band(ctx, levels, mid_lev, hi_lev, depth + 1, out)?;
    Ok(())
}

/// Checks both Inside (`hi_lev` after `<= lo_lev`) and Outside (`lo_lev`
/// after `>= hi_lev`) walking orders for full-width flanks between the
/// two sides of the sub-band, returning line segments `[at, flank]` to
/// expand `zone` around any uncovered corner.
fn merge_flank_hits(
    ctx: &FillCtx<'_>,
    levels: &BTreeMap<u32, Vec<(Vec<Point2>, Cut)>>,
    lo_lev: u32,
    hi_lev: u32,
) -> Vec<Vec<Point2>> {
    let tool_r = ctx.band.tool_r;
    let step = 0.25 * tool_r;
    let r_cut = CUT_MARGIN * tool_r;
    let cut_lo = |q: Point2, d: f64| {
        ctx.part_wall.iter().any(|c| c.near(q, d))
            || levels.range(..=lo_lev).flat_map(|(_, v)| v.iter()).any(|(_, c)| c.near(q, d))
    };
    let cut_hi = |q: Point2, d: f64| {
        ctx.outer_edge.iter().any(|c| c.near(q, d))
            || levels.range(hi_lev..).flat_map(|(_, v)| v.iter()).any(|(_, c)| c.near(q, d))
    };
    let mut segs = Vec::new();
    let mut collect = |walk: &[Point2], cut_near: &dyn Fn(Point2, f64) -> bool| {
        let pts = samples(walk, step);
        let hits = full_width_flanks(&pts, ctx.material, tool_r, r_cut, cut_near);
        let mut last: Option<Point2> = None;
        for (idx, &[at, fl, fr]) in hits.iter().enumerate() {
            if idx + 1 == hits.len() || last.is_none_or(|p| dist(p, at) >= step) {
                segs.push(vec![at, fl]);
                segs.push(vec![at, fr]);
                last = Some(at);
            }
        }
    };
    if let Some(lo) = levels.get(&lo_lev) {
        for (w, _) in lo {
            let rev: Vec<Point2> = w.iter().rev().copied().collect();
            collect(&rev, &cut_hi);
        }
    }
    if let Some(hi) = levels.get(&hi_lev) {
        for (w, _) in hi {
            collect(w, &cut_lo);
        }
    }
    segs
}

/// The stock between two rings that a flank of one meets and the swath
/// of the other leaves, or that neither swath reaches: either order of
/// the walk cuts the full width there.
fn gap(ring: &Region, inner: &[Vec<Point2>], outer: &[Vec<Point2>], tool_r: f64) -> Result<Region, GenError> {
    let near = |w: &[Vec<Point2>], r: f64| Region::sweep(w, r);
    let (i_flank, i_cut) = (near(inner, FLANK * tool_r)?, near(inner, tool_r)?);
    let (o_flank, o_cut) = (near(outer, FLANK * tool_r)?, near(outer, tool_r)?);
    let a = ring.intersect(&i_flank)?.difference(&o_cut)?;
    let b = ring.intersect(&o_flank)?.difference(&i_cut)?;
    let mid = ring.difference(&i_cut.union(&o_cut)?)?;
    Ok(a.union(&b)?.union(&mid)?)
}

/// A piece's walk, closed when the piece is a whole ring.
pub(super) fn closed_walk(p: &Piece) -> Vec<Point2> {
    let mut w = p.pts.clone();
    if p.closed && let Some(&f) = w.first() {
        w.push(f);
    }
    w
}

/// The distance curve at `d` from the part, walked as the rings are, and
/// clipped to the area round `zone` that a fill needs (`zone` grown by
/// `reach`, inside the cutter-centre area). Open pieces that end within
/// `3.5 * tool_r` of `c`'s boundary extend to that boundary so pass
/// links along the edge in air reach them without lifting.
fn level(band: &Band<'_>, d: f64, zone: &Region, reach: f64, lev: u32) -> Result<Vec<Piece>, GenError> {
    let base_near = zone.offset(reach)?.intersect(band.c)?;
    let mut out = Vec::new();
    for l in band.hc.offset(d)?.loops() {
        // As `level_rings::curves` walks ring `j`: the distance curve has
        // the cut side on its left, reversed for climb.
        let walk: Vec<Point2> = if band.climb { l.points().to_vec() } else { l.points().iter().rev().copied().collect() };
        let tails = tail_extension(band, &walk, &base_near)?;
        let near = if tails.is_empty() {
            base_near.clone()
        } else {
            base_near.union(&Region::sweep(&tails, 0.1)?)?.intersect(band.c)?
        };
        match near.clip_ring(&walk, true)? {
            RingClip::Whole => {
                let pts = clean_walk(walk, true);
                let len: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
                if pts.len() >= 3 || (pts.len() == 2 && len >= MIN_FILL) {
                    out.push(Piece { level: lev, pts, closed: true, core: false });
                }
            }
            RingClip::Pieces(ps) => {
                for pts in ps {
                    let pts = clean_walk(pts, false);
                    let len: f64 = pts.windows(2).map(|w| dist(w[0], w[1])).sum();
                    if pts.len() >= 2 && len >= MIN_FILL {
                        out.push(Piece { level: lev, pts, closed: false, core: false });
                    }
                }
            }
        }
    }
    Ok(out)
}

fn tail_extension(band: &Band<'_>, walk: &[Point2], near: &Region) -> Result<Vec<Vec<Point2>>, GenError> {
    let mut tails = Vec::new();
    let RingClip::Pieces(cands) = band.c.clip_ring(walk, true)? else {
        return Ok(tails);
    };
    let max_tail = 3.5 * band.tool_r;
    for cand in cands {
        let inside: Vec<usize> = cand.iter().enumerate().filter(|(_, p)| near.contains(**p)).map(|(i, _)| i).collect();
        let (Some(&first), Some(&last)) = (inside.first(), inside.last()) else {
            continue;
        };
        let head: f64 = cand[..=first].windows(2).map(|w| dist(w[0], w[1])).sum();
        if first > 0 && head <= max_tail {
            tails.push(cand[..=first].to_vec());
        }
        let tail: f64 = cand[last..].windows(2).map(|w| dist(w[0], w[1])).sum();
        if last + 1 < cand.len() && tail <= max_tail {
            tails.push(cand[last..].to_vec());
        }
    }
    Ok(tails)
}

#[cfg(test)]
mod tests {
    use super::{Band, RING, fills};
    use crate::geo2d::{Loop, Point2, Region};
    use crate::machining2d::level_flanks::Material;

    fn circle(r: f64, from: f64, to: f64, n: u32) -> Vec<Point2> {
        (0..=n).map(|k| {
            let a = from + (to - from) * f64::from(k) / f64::from(n);
            Point2::new(r * a.cos(), r * a.sin())
        })
        .collect()
    }

    fn region(pts: Vec<Point2>) -> Region {
        Region::new(vec![Loop::new(pts).unwrap_or_else(|e| panic!("{e:?}"))]).unwrap_or_else(|e| panic!("{e:?}"))
    }

    /// #3788 pin: a band whose outer ring is whole needs no fill; a band
    /// whose outer ring stops half way leaves stock that the inner ring's
    /// flank meets, and gets a fill at half a step, between the two rings.
    #[test]
    fn a_band_with_a_ring_gap_gets_a_fill_and_a_whole_band_none() {
        let tau = std::f64::consts::TAU;
        let hc = region(circle(10.0, 0.0, tau, 255).into_iter().take(256).collect());
        let c = region(vec![Point2::new(-30.0, -30.0), Point2::new(30.0, -30.0), Point2::new(30.0, 30.0), Point2::new(-30.0, 30.0)]);
        let mat = Material::new(&c, None, 4.0).unwrap_or_else(|e| panic!("{e:?}"));
        let band = Band { c: &c, hc: &hc, stock: None, j: 1, step: 2.0, tool_r: 4.0, climb: true };
        let inner = vec![circle(10.0, 0.0, tau, 256)];
        let whole = fills(&band, &mat, &[], &[], &inner, &[circle(12.0, 0.0, tau, 256)]).unwrap_or_else(|e| panic!("{e:?}"));
        assert!(whole.is_empty(), "a whole band: {} fill(s)", whole.len());
        // The wide band: step 8 (2 radii), the outer ring on one half only.
        let wide = Band { step: 8.0, ..band };
        let half = fills(&wide, &mat, &[], &[], &inner, &[circle(18.0, 0.0, 0.5 * tau, 128)]).unwrap_or_else(|e| panic!("{e:?}"));
        assert!(!half.is_empty(), "the gap gets a fill");
        for f in &half {
            assert!(f.level > 0 && f.level < RING, "a fill lies between ring 0 and ring 1: {}", f.level);
            let r = f.pts.iter().map(|q| q.x.hypot(q.y)).fold(0.0_f64, f64::max);
            assert!(r > 10.5 && r < 17.5, "the fill stands between the two rings: {r}");
        }
    }
}
