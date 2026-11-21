//! Quantize the nominal polygon and find a perpendicular Agie entry on the stated waste side (planar geometry).
use super::{AgieJob, AgieSide};
mod entry;

type Point = [i64; 2];
const UNITS: f64 = 100_000.0; // Five decimal places in the native ISO geometry file.
const MAX_VERTICES: usize = 4096;

fn quantize(v: f64) -> Result<i64, String> {
    if !v.is_finite() || v.abs() > 9999.9999 {
        return Err("Agie coordinates must be finite and within ±9999.9999 mm".into());
    }
    Ok((v * UNITS).round() as i64)
}

fn cross(a: Point, b: Point, p: Point) -> i128 {
    i128::from(b[0] - a[0]) * i128::from(p[1] - a[1])
        - i128::from(b[1] - a[1]) * i128::from(p[0] - a[0])
}

fn between(a: Point, b: Point, p: Point) -> bool {
    (0..2).all(|d| (a[d].min(b[d])..=a[d].max(b[d])).contains(&p[d]))
}

fn intersects(a: Point, b: Point, c: Point, d: Point) -> bool {
    let (u, v, w, z) = (
        cross(a, b, c),
        cross(a, b, d),
        cross(c, d, a),
        cross(c, d, b),
    );
    (u.signum() * v.signum() == -1 && w.signum() * z.signum() == -1)
        || (u == 0 && between(a, b, c))
        || (v == 0 && between(a, b, d))
        || (w == 0 && between(c, d, a))
        || (z == 0 && between(c, d, b))
}

pub(super) fn stock_offset_um(job: &AgieJob) -> Result<u32, String> {
    for &v in job
        .stock_min
        .iter()
        .chain(&job.stock_max)
        .chain(job.start.iter().flatten())
    {
        quantize(v)?;
        if (v - (v * 1000.0).round() / 1000.0).abs() > 1e-9 {
            return Err(
                "Agie stock and start-hole coordinates must be representable to 0.001 mm".into(),
            );
        }
    }
    let [width, depth, height] = std::array::from_fn(|i| job.stock_max[i] - job.stock_min[i]);
    if !(width > 0.0 && width <= 350.0 && depth > 0.0 && depth <= 250.0) {
        return Err("Agie stock must fit the stated 350 × 250 mm working span".into());
    }
    if job.stock_max[2] != 0.0 {
        return Err("The native Agie quality requires stock top Z0".into());
    }
    job.quality
        .working_offsets(height)?
        .iter()
        .copied()
        .max()
        .map(u32::from)
        .ok_or_else(|| "The selected Agie quality has no working steps".into())
}

#[cfg(test)]
pub(super) fn prepare(job: &AgieJob) -> Result<Vec<[f64; 2]>, String> {
    prepare_with_reach(job).map(|(points, _)| points)
}

/// Nominal ISO vertices and the maximum reach of any compensated corner, in mm.
pub(super) fn prepare_with_reach(job: &AgieJob) -> Result<(Vec<[f64; 2]>, f64), String> {
    let offset_um = stock_offset_um(job)?;
    let start = job
        .start
        .ok_or("Agie requires a predrilled 2 mm start hole")?;
    if !(0..2).all(|i| start[i] - 1.0 >= job.stock_min[i] && start[i] + 1.0 <= job.stock_max[i]) {
        return Err("The complete 2 mm Agie start hole must lie inside the stock".into());
    }
    if job.side == AgieSide::Die && job.stock_min[2] != -20.0 {
        return Err("The Agie die opening requires the qualified 20 mm stock height".into());
    }
    let input = &job.contour;
    if input.len() < 3 || input.len() > MAX_VERTICES + 1 {
        return Err(format!(
            "Agie needs one closed nominal polygon with 3–{MAX_VERTICES} vertices"
        ));
    }
    let mut ring = Vec::with_capacity(input.len());
    for &[x, y] in input {
        let p = [quantize(x)?, quantize(y)?];
        if !(0..2).all(|i| {
            let value = p[i] as f64 / UNITS;
            value >= job.stock_min[i] && value <= job.stock_max[i]
        }) {
            return Err("Agie nominal contour extends outside the stock".into());
        }
        ring.push(p);
    }
    if input.first() == input.last() {
        ring.pop();
    }
    let n = ring.len();
    if !(3..=MAX_VERTICES).contains(&n) {
        return Err("Agie contour vertex count exceeds the supported range".into());
    }
    // Validate quantized segments, including the closing edge, before choosing a start point.
    // Keep collinear intermediate vertices out of the search for the perpendicular entry edge.
    let mut outline = Vec::with_capacity(n);
    for i in 0..n {
        let a = ring[(i + n - 1) % n];
        let b = ring[i];
        let c = ring[(i + 1) % n];
        if a == b || (cross(a, b, c) == 0 && !between(a, c, b)) {
            return Err("Agie nominal contour contains a collapsed or reversed edge".into());
        }
        if cross(a, b, c) != 0 {
            outline.push(b);
        }
    }
    let n = outline.len();
    let mut area = 0_i128;
    for i in 0..n {
        let (a, b) = (outline[i], outline[(i + 1) % n]);
        area += cross([0, 0], a, b);
        for j in (i + 2)..n {
            if i == 0 && j + 1 == n {
                continue;
            }
            if intersects(a, b, outline[j], outline[(j + 1) % n]) {
                return Err("Agie nominal contour crosses or touches itself".into());
            }
        }
    }
    if area == 0 {
        return Err("Agie nominal contour has no enclosed area".into());
    }
    if area < 0 {
        outline.reverse();
    }
    if job.side == AgieSide::Die {
        // Convexity proves that every straight lead from the inside hole
        // to the chosen wall stays in the opening throughout its length.
        if (0..n).any(|i| cross(outline[i], outline[(i + 1) % n], outline[(i + 2) % n]) <= 0) {
            return Err("Agie requires a convex die opening".into());
        }
        outline.reverse();
    }
    // Material is on the left: inward miters of a die now use the same
    // join reach and feature-spacing checks as inside corners of a punch.
    let mut reach = check_cut_envelope(&outline, job, offset_um)?;
    let hole = [quantize(start[0])?, quantize(start[1])?];
    let (edge, foot) = entry::entry_face(&outline, hole, job.side)?;
    // Begin the nominal program at the middle of the chosen edge, facing along that edge. The
    // hole and the full three-millimetre lead share its waste half-plane. Compensation is
    // deliberately left to the Agie technology. The closing move returns to the inserted point.
    let exact_face = cross(outline[edge], outline[(edge + 1) % n], foot) == 0;
    outline.rotate_left((edge + 1) % n);
    let mut vertices = vec![foot];
    vertices.extend(outline);
    // An exterior rounded seam adds a small convex corner and slightly
    // changes its neighbours. Check the envelope of the actual ISO lines.
    if !exact_face {
        reach = reach.max(check_cut_envelope(&vertices, job, offset_um)?);
    }
    if job.side == AgieSide::Die {
        // Use the same CCW nominal ISO convention for both closed cycles.
        // Retain the first point so separation still occurs on the chosen face.
        vertices[1..].reverse();
    }
    let points = vertices
        .into_iter()
        .map(|p| [p[0] as f64 / UNITS, p[1] as f64 / UNITS])
        .collect();
    Ok((points, reach))
}

/// Check the swept wire against the stock before Agie chooses its corner joins.
/// A round join lies within the rough offset of its nominal vertex. A miter
/// reaches the intersection of the two face lines offset toward the waste.
/// The segments connecting adjacent joins stay inside their stock bounds.
fn check_cut_envelope(ring: &[Point], job: &AgieJob, offset_um: u32) -> Result<f64, String> {
    // Round the measured wire-centre offset plus the 125 µm wire radius
    // upwards to a tenth of a millimetre for the straight/round swept envelope.
    let margin_um = (offset_um + 125).next_multiple_of(100);
    let stock_margin = i64::from(margin_um) * 100;
    let rough_offset = f64::from(offset_um) * 100.0;
    let wire_and_allowance = stock_margin as f64 - rough_offset;
    let lo = [quantize(job.stock_min[0])?, quantize(job.stock_min[1])?];
    let hi = [quantize(job.stock_max[0])?, quantize(job.stock_max[1])?];
    let n = ring.len();
    let outward = |a: Point, b: Point| {
        let dx = (b[0] - a[0]) as f64;
        let dy = (b[1] - a[1]) as f64;
        let length = dx.hypot(dy);
        [dy / length, -dx / length]
    };
    let mut reentrant = false;
    let mut max_miter: f64 = 0.0;
    for (i, &corner) in ring.iter().enumerate() {
        let previous = ring[(i + n - 1) % n];
        let next = ring[(i + 1) % n];
        let a = outward(previous, corner);
        let b = outward(corner, next);
        // Use the squared bisector length instead of 1 + dot(a, b);
        // the latter loses precision when the punch corner is very sharp.
        let bisector = [a[0] + b[0], a[1] + b[1]];
        let norm2 = bisector[0] * bisector[0] + bisector[1] * bisector[1];
        if norm2 <= f64::EPSILON {
            return Err(
                "Agie cannot bound the rough pass at a nearly reversed contour corner".into(),
            );
        }
        let reach = 2.0 * rough_offset / norm2.sqrt();
        max_miter = max_miter.max(reach);
        if cross(previous, corner, next) < 0 {
            reentrant = true;
            // At an inside turn the two offset faces intersect outside the
            // drawing corner. The strict wire cover law permits 0.002 mm of
            // rest. Use the largest resident offset across all four passes.
            if reach - rough_offset > 0.002 * UNITS {
                return Err(
                    "Agie rough compensation cannot reach the reentrant corner within 0.002 mm"
                        .into(),
                );
            }
        }
        for axis in 0..2 {
            if corner[axis] - stock_margin < lo[axis] || corner[axis] + stock_margin > hi[axis] {
                return Err(format!(
                    "The Agie rough pass and wire need at least {:.1} mm of stock beyond the punch",
                    f64::from(margin_um) / 1000.0,
                ));
            }
            let miter = 2.0 * rough_offset * bisector[axis] / norm2;
            let left = (lo[axis] - corner[axis]) as f64;
            let right = (hi[axis] - corner[axis]) as f64;
            if miter - wire_and_allowance < left || miter + wire_and_allowance > right {
                return Err(
                    "The Agie rough corner and wire can reach outside the stock at a contour vertex"
                        .into(),
                );
            }
        }
    }
    if reentrant {
        // The full segment between two joins stays inside the nominal edge's
        // capsule. Bound each join by the longest miter, plus a complete rough
        // kerf allowance. Disjoint capsules on unrelated faces exclude contact
        // at every smaller offset, including all trims.
        let clearance = (2.0 * (max_miter + rough_offset)).ceil() as i64;
        check_feature_spacing(ring, clearance)?;
    }
    Ok(max_miter / UNITS)
}

fn point_clears_segment(p: Point, a: Point, b: Point, clearance2: i128) -> bool {
    let edge = vector(a, b);
    let from_a = vector(a, p);
    let length2 = dot(edge, edge);
    let projection = dot(edge, from_a);
    if projection <= 0 {
        dot(from_a, from_a) > clearance2
    } else if projection >= length2 {
        let from_b = vector(b, p);
        dot(from_b, from_b) > clearance2
    } else {
        let side = cross(a, b, p);
        side * side > clearance2 * length2
    }
}

fn check_feature_spacing(ring: &[Point], clearance: i64) -> Result<(), String> {
    let n = ring.len();
    let clearance2 = i128::from(clearance).pow(2);
    for (i, &a) in ring.iter().enumerate() {
        let b = ring[(i + 1) % n];
        let edge = vector(a, b);
        // The two joins on a short face could meet before full rough offset.
        if dot(edge, edge) <= clearance2 {
            return Err(
                "Agie cannot separate adjacent corner joins on a short reentrant profile segment"
                    .into(),
            );
        }
        for j in i + 2..n {
            if i == 0 && j + 1 == n {
                continue;
            }
            let (c, d) = (ring[j], ring[(j + 1) % n]);
            // Nominal segments are disjoint. The minimum distance between two
            // disjoint segments occurs at an endpoint of at least one segment.
            if !point_clears_segment(a, c, d, clearance2)
                || !point_clears_segment(b, c, d, clearance2)
                || !point_clears_segment(c, a, b, clearance2)
                || !point_clears_segment(d, a, b, clearance2)
            {
                return Err(
                    "Agie cannot separate the rough pass from a nearby nonadjacent profile segment"
                        .into(),
                );
            }
        }
    }
    Ok(())
}

fn vector(a: Point, b: Point) -> Point {
    [b[0] - a[0], b[1] - a[1]]
}

fn dot(a: Point, b: Point) -> i128 {
    i128::from(a[0]) * i128::from(b[0]) + i128::from(a[1]) * i128::from(b[1])
}
