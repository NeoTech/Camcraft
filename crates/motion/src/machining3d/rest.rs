//! The 3-D ladder's islands law: the part material standing above a
//! level plane (split from `machining3d.rs` by M5.0.1; the hanging
//! shadow added by the blind-floor fix). Provenance: plane sectioning
//! plus the XY shadow of a closed solid's down-facing boundary - the
//! standard projection identity of computational geometry (a column's
//! solid above a plane is either cut by the plane or starts at a
//! downward-facing face above it); no vendor material consulted.

use super::params::Pocket3DParams;

use crate::geo2d::Region;
use crate::geometry::slice::{self, SliceError};
use crate::geometry::Triangle;
use crate::machining2d::{cut_levels_under, GenError};
use crate::stock_sim::Columns;
/// Half-open slices sit exactly on vertices for tessellated steps; this
/// is the same lattice the weld uses, so a grazing plane nudges rather
/// than lies.
pub(super) const EPS: f64 = 1e-6;

/// A hanging shadow that overlaps a level's target by less than this
/// (mm²) changes nothing a cutter can reach: the target is kept as it
/// was, byte for byte (the region ops would only re-normalize it).
const AREA_EPS: f64 = 1e-6;

pub(super) fn from_slice(e: SliceError) -> GenError {
    match e {
        SliceError::Geo(g) => GenError::Geo(g),
        // #1407: a model fault, named as one - never "machining parameters".
        e @ (SliceError::Open { .. } | SliceError::NonManifold { .. }) => GenError::Model(e.to_string()),
        other => GenError::Invalid(other.to_string()),
    }
}

/// The part material standing above `z`, projected to XY - the islands
/// law itself, [`slice::material_overhead`]: the model's cross-section
/// strictly above the plane (nudge-retried when a plane grazes a
/// tessellated shelf - reading an errored slice as no-island would be a
/// silent gouge, the retry lands above the shelf instead, failing safe),
/// united with the shadow of every down-facing face above it
/// ([`slice::shadow_above`]). The cross-section alone is only the whole
/// story for a part solid all the way down to the plane: a plane under
/// the part's underside (a blind floor's level run to the stock bottom)
/// or under an overhang sections nothing there, yet the material above
/// it shadows every column it covers. Public because it is the islands
/// law: the generators read it to shape each level (the 3-D ladder here,
/// the 2D contour's ring ladder and the app's pocket region on a part
/// through the same geometry function), and the tests pin the
/// monotonicity the ladder assumes (M2.1a).
pub fn material_above(facets: &[Triangle], z: f64) -> Result<Region, GenError> {
    slice::material_overhead(facets, z).map_err(from_slice)
}

/// A level's target: the footprint minus the material standing above
/// `z` ([`material_above`]'s law). Read in two steps so a part solid
/// down to the plane - every level a ladder cut before the blind-floor
/// fix - keeps its target exactly: the hanging shadow is subtracted
/// only when it overlaps what the cross-section left.
pub(super) fn level_target(facets: &[Triangle], footprint: &Region, z: f64) -> Result<Region, GenError> {
    let islands = slice::material_above(facets, z).map_err(from_slice)?;
    let target = footprint.difference(&islands)?;
    if target.is_empty() {
        return Ok(target);
    }
    let hang = slice::shadow_above(facets, z).map_err(from_slice)?;
    if hang.is_empty() || target.intersect(&hang)?.area() <= AREA_EPS {
        return Ok(target);
    }
    Ok(target.difference(&hang)?)
}

/// Cuts `level_target` at `z` down to the remaining stock columns above
/// `z` when rest-machining stock masking is active.
fn masked_target(
    facets: &[Triangle],
    footprint: &Region,
    z: f64,
    mask: Option<&Columns<'_>>,
) -> Result<Region, GenError> {
    let target = level_target(facets, footprint, z)?;
    match mask {
        None => Ok(target),
        Some(dex) => Ok(target.intersect(&dex.above(z)?)?),
    }
}

/// Sorts `zs` ascending (or descending when `descending` is true) and
/// collapses values within `EPS` so nearly coplanar heights merge once.
fn sort_dedup_heights(mut zs: Vec<f64>, descending: bool) -> Vec<f64> {
    if descending {
        zs.sort_by(|a, b| b.total_cmp(a));
    } else {
        zs.sort_by(f64::total_cmp);
    }
    zs.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    zs
}

/// Distinct Z heights of up-facing horizontal triangles strictly in
/// `(lo + EPS, hi - EPS)`, ascending. Using `zmax` ensures any tiny
/// tessellation tilt on a floor facet never puts the cut plane below
/// the facet's highest vertex.
fn flat_floor_heights_between(facets: &[Triangle], lo: f64, hi: f64) -> Vec<f64> {
    let mut zs = Vec::new();
    for t in facets {
        let zmin = t.a.z.min(t.b.z).min(t.c.z);
        let zmax = t.a.z.max(t.b.z).max(t.c.z);
        if !(zmin.is_finite() && zmax.is_finite()) || zmax - zmin > EPS {
            continue;
        }
        if zmax <= lo + EPS || zmax >= hi - EPS {
            continue;
        }
        let n = (t.b - t.a).cross(t.c - t.a);
        if n.z > 0.0 {
            zs.push(zmax);
        }
    }
    sort_dedup_heights(zs, false)
}

/// Distinct finite triangle vertex Z heights strictly inside
/// `(lo + EPS, hi - EPS)`, ascending, used when a shallow non-flat
/// pocket has no horizontal floor triangles.
fn vertex_heights_between(facets: &[Triangle], lo: f64, hi: f64) -> Vec<f64> {
    let zs: Vec<f64> = facets
        .iter()
        .flat_map(|t| [t.a.z, t.b.z, t.c.z])
        .filter(|&z| z.is_finite() && z > lo + EPS && z < hi - EPS)
        .collect();
    sort_dedup_heights(zs, false)
}

/// Depth ladder for 3-D pocketing (#2004, #2088): `cut_levels_under`
/// plus any blind floor strictly between `p.top_z` and the first cut
/// level below it, so a blind feature shallower than one stepdown is
/// cut at its floor (never skipped or refused as empty) without adding
/// levels on sloped faces or below the first cut level.
pub(super) fn rough_levels(
    facets: &[Triangle],
    footprint: &Region,
    p: &Pocket3DParams,
    mask: Option<&Columns<'_>>,
) -> Result<Vec<f64>, GenError> {
    let mut zs = cut_levels_under(p.top_z, p.links.approach.material_top, p.bottom_z, p.stepdown)?;
    let Some(first) = zs.iter().copied().find(|&z| z < p.top_z - EPS) else {
        return Ok(zs);
    };
    let r_pad = p.tool_diameter * 0.5 + p.stock_to_leave;
    let first_center = masked_target(facets, footprint, first, mask)?.offset(-r_pad)?;
    let empty_below = first_center.is_empty();
    let mut covered = first_center;
    let mut added = Vec::new();
    let mut open_flat = None;
    for z in flat_floor_heights_between(facets, first, p.top_z) {
        let target = masked_target(facets, footprint, z, mask)?;
        if target.is_empty() {
            continue;
        }
        let center = target.offset(-r_pad)?;
        if center.is_empty() {
            open_flat.get_or_insert(z);
            continue;
        }
        let has_new = if covered.is_empty() {
            true
        } else {
            !center.difference(&covered)?.offset(-1e-3)?.is_empty()
        };
        if has_new {
            covered = covered.union(&center)?;
            added.push(z);
        }
    }
    if empty_below && added.is_empty() {
        for z in vertex_heights_between(facets, first, p.top_z) {
            let target = masked_target(facets, footprint, z, mask)?;
            if target.is_empty() {
                continue;
            }
            if !target.offset(-r_pad)?.is_empty() {
                added.push(z);
                break;
            }
            open_flat.get_or_insert(z);
        }
        if added.is_empty()
            && let Some(z) = open_flat
        {
            added.push(z);
        }
    }
    if !added.is_empty() {
        zs.extend(added);
        zs = sort_dedup_heights(zs, true);
    }
    Ok(zs)
}
