//! Place the perpendicular Agie lead on the waste side using native ISO coordinates.
use super::{AgieSide, Point, cross, dot, vector};

const LEAD: i128 = 300_000; // 3 mm on the ISO lattice.

/// The complete approach stays on the waste side of this emitted face.
fn clears_entry_line(a: Point, b: Point, hole: Point) -> bool {
    let side = cross(a, b, hole);
    let edge = vector(a, b);
    side < 0 && side * side >= LEAD * LEAD * dot(edge, edge)
}

/// Quantize the perpendicular projection without moving the start hole or
/// cutting into retained material. Compare the four nearest cell corners against the
/// exact rational projection. Allow at most one grid unit (0.00001 mm)
/// Euclidean error to choose a point on the waste side when nearest rounding is inward.
fn outside_seam(a: Point, b: Point, hole: Point, along: i128) -> Option<Point> {
    let edge = vector(a, b);
    let length2 = dot(edge, edge);
    let numerator: [i128; 2] =
        std::array::from_fn(|k| i128::from(a[k]) * length2 + i128::from(edge[k]) * along);
    // The projected point lies strictly between bounded i64 endpoints.
    let floor = numerator.map(|v| v.div_euclid(length2) as i64);
    let mut best = None;
    for x in floor[0]..=floor[0] + 1 {
        for y in floor[1]..=floor[1] + 1 {
            let p = [x, y];
            let residual: [i128; 2] =
                std::array::from_fn(|k| i128::from(p[k]) * length2 - numerator[k]);
            let error2 = residual[0] * residual[0] + residual[1] * residual[1];
            if error2 > length2 * length2 || cross(a, b, p) > 0 {
                continue;
            }
            let from_a = dot(vector(a, p), edge);
            let from_b = length2 - from_a;
            // Measure room along the original face, before and after the seam.
            // Neither coordinate rounding nor a short angled leg may add
            // fictitious room for the transition and separation segment.
            if from_a <= 0
                || from_b <= 0
                || from_a * from_a < LEAD * LEAD * length2
                || from_b * from_b < LEAD * LEAD * length2
                || !clears_entry_line(a, p, hole)
                || !clears_entry_line(p, b, hole)
            {
                continue;
            }
            if best.is_none_or(|(previous, _)| error2 < previous) {
                best = Some((error2, p));
            }
        }
    }
    best.map(|(_, p)| p)
}

pub(super) fn entry_face(
    ring: &[Point],
    hole: Point,
    side: AgieSide,
) -> Result<(usize, Point), String> {
    if side == AgieSide::Die
        && ring.iter().enumerate().any(|(i, &a)| {
            let b = ring[(i + 1) % ring.len()];
            let d = cross(a, b, hole);
            let e = vector(a, b);
            d >= 0 || d * d <= i128::from(100_000).pow(2) * dot(e, e)
        })
    {
        return Err("The complete 2 mm Agie start hole must lie inside the die opening".into());
    }
    let mut best = None;
    let mut nominal_room = false;
    let mut nearest = f64::INFINITY;
    for (i, &a) in ring.iter().enumerate() {
        let b = ring[(i + 1) % ring.len()];
        if !clears_entry_line(a, b, hole)
            || (side == AgieSide::Punch && ring.iter().any(|&p| cross(a, b, p) < 0))
        {
            continue;
        }
        let edge = vector(a, b);
        let length2 = dot(edge, edge);
        let along = dot(vector(a, hole), edge);
        let remaining = length2 - along;
        if along <= 0
            || remaining <= 0
            || along * along < LEAD * LEAD * length2
            || remaining * remaining < LEAD * LEAD * length2
        {
            continue;
        }
        nominal_room = true;
        let distance = -(cross(a, b, hole) as f64) / (length2 as f64).sqrt();
        if distance < nearest
            && let Some(foot) = outside_seam(a, b, hole, along)
        {
            // A convex opening is an intersection of half-planes. Both
            // endpoints of the approach must clear every other wall, so the
            // swept wire stays inside. Use the 20 mm rough-pass + wire bound.
            if side == AgieSide::Die
                && ring.iter().enumerate().any(|(j, &c)| {
                    let d = ring[(j + 1) % ring.len()];
                    let signed = cross(c, d, foot);
                    let e = vector(c, d);
                    j != i
                        && (signed >= 0 || signed * signed <= i128::from(40_000).pow(2) * dot(e, e))
                })
            {
                continue;
            }
            nearest = distance;
            best = Some((i, foot));
        }
    }
    best.ok_or_else(|| {
        if side == AgieSide::Die {
            "Agie needs 3 mm of clear inward lead and 3 mm on both sides of its entry on a straight opening wall".into()
        } else if nominal_room {
            "Agie cannot place an exterior entry on the 0.00001 mm ISO grid and keep the required 3 mm clearance".into()
        } else {
            "Agie needs the start hole at least 3 mm outside a straight face, with at least 3 mm of straight face on both sides of the perpendicular entry for the lead and separation".into()
        }
    })
}
