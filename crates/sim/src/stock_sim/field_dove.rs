//! The dovetail head solid (#1444, #4004): exact Euclidean SDF to a
//! sharp or corner-rounded dovetail frustum in its (rho, along-axis)
//! half-plane; elementary plane geometry, no vendor source.

use super::field_solid::dist_pt_seg2;

/// Exact Euclidean SDF to the dovetail head's meridian (#1444, #4004):
/// flat bottom from `(0, 0)` to `(r - corner, 0)`, corner arc of radius
/// `corner` centred at `(r - corner, corner)` up to tangent height
/// `corner * (1 + sin t)`, and straight flank of slope `cot = tan t`
/// up to `(top, flank)` and `(0, flank)`. Negative inside.
pub(crate) fn dove_dist(rho: f64, zl: f64, r: f64, cot: f64, corner: f64, flank: f64) -> f64 {
    if corner <= 0.0 {
        let top = r - flank * cot;
        let side = (rho - r + zl * cot) / 1.0_f64.hypot(cot);
        if zl >= 0.0 && zl <= flank && side <= 0.0 {
            return side.max(-zl).max(zl - flank);
        }
        return dist_pt_seg2(rho, zl, 0.0, 0.0, r, 0.0)
            .min(dist_pt_seg2(rho, zl, r, 0.0, top, flank))
            .min(dist_pt_seg2(rho, zl, 0.0, flank, top, flank));
    }
    let c = corner;
    let h = 1.0_f64.hypot(cot);
    let sin_t = cot / h;
    let cos_t = 1.0 / h;
    let k = r - c;
    let arc_end = c * (1.0 + sin_t);
    let s_tan = arc_end.min(flank);
    let (r_tan, top, nx, ny) = if flank <= arc_end {
        let r_end = k + (c * c - (s_tan - c) * (s_tan - c)).max(0.0).sqrt();
        (r_end, r_end, (r_end - k) / c, (s_tan - c) / c)
    } else {
        let r_end = k + c * cos_t;
        let top_r = k + c * (h + cot) - flank * cot;
        (r_end, top_r, cos_t, sin_t)
    };
    let inside = zl >= 0.0
        && zl <= flank
        && if zl <= s_tan {
            rho <= k || (rho - k).hypot(zl - c) <= c
        } else {
            (rho - k - c * (h + cot) + zl * cot) <= 0.0
        };
    let dist_bottom = if k > 0.0 {
        dist_pt_seg2(rho, zl, 0.0, 0.0, k, 0.0)
    } else {
        rho.hypot(zl)
    };
    let vx = rho - k;
    let vy = zl - c;
    let dist_arc = if vx >= 0.0 && vx * ny - vy * nx >= 0.0 {
        (vx.hypot(vy) - c).abs()
    } else {
        vx.hypot(zl).min((rho - r_tan).hypot(zl - s_tan))
    };
    let dist_flank = if flank > s_tan {
        dist_pt_seg2(rho, zl, r_tan, s_tan, top, flank)
    } else {
        f64::INFINITY
    };
    let dist_top = dist_pt_seg2(rho, zl, 0.0, flank, top, flank);
    let d = dist_bottom.min(dist_arc).min(dist_flank).min(dist_top);
    if inside { -d } else { d }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reference(rho: f64, zl: f64, r: f64, cot: f64, c: f64, flank: f64) -> f64 {
        let h = 1.0_f64.hypot(cot);
        let sin_t = cot / h;
        let cos_t = 1.0 / h;
        let k = r - c;
        let arc_end = c * (1.0 + sin_t);
        let s_tan = arc_end.min(flank);
        let r_tan = if c <= 0.0 {
            r
        } else if flank <= arc_end {
            k + (c * c - (s_tan - c).powi(2)).max(0.0).sqrt()
        } else {
            k + c * cos_t
        };
        let top = if c <= 0.0 {
            r - flank * cot
        } else if flank <= arc_end {
            r_tan
        } else {
            k + c * (h + cot) - flank * cot
        };
        let n = 4000;
        let mut best = f64::INFINITY;
        let mut eat = |x: f64, y: f64| best = best.min((rho - x).hypot(zl - y));
        for i in 0..=n {
            let s = i as f64 / n as f64;
            eat(s * k, 0.0);
            eat(s * top, flank);
            if flank > s_tan {
                eat(r_tan + s * (top - r_tan), s_tan + s * (flank - s_tan));
            }
            if c > 0.0 {
                let a1 = ((s_tan - c) / c).clamp(-1.0, 1.0).asin();
                let th = -std::f64::consts::FRAC_PI_2 + s * (a1 + std::f64::consts::FRAC_PI_2);
                eat(k + c * th.cos(), c + c * th.sin());
            }
        }
        let prof_r = if zl < 0.0 || zl > flank {
            -1.0
        } else if c <= 0.0 {
            r - zl * cot
        } else if zl <= s_tan {
            k + (c * c - (zl - c).powi(2)).max(0.0).sqrt()
        } else {
            k + c * (h + cot) - zl * cot
        };
        let member = (0.0..=flank).contains(&zl) && rho <= prof_r;
        if member { -best } else { best }
    }

    #[test]
    fn the_dovetail_distance_matches_a_dense_boundary_reference() {
        let cot = 1.0 / 60.0_f64.to_radians().tan();
        let cases = [
            (9.525, cot, 0.0, 9.525),
            (9.525, cot, 0.254, 9.525),
            (9.525, cot, 3.0, 9.525),
            (10.0, cot, 2.0, 3.0),
            (10.0, 1.0, 10.0, 15.0),
        ];
        let mut count = 0;
        for (r, cot, c, flank) in cases {
            for i in 0..=55 {
                for j in 0..=65 {
                    let rho = i as f64 * 0.23;
                    let zl = -2.0 + j as f64 * 0.25;
                    let got = dove_dist(rho, zl, r, cot, c, flank);
                    let want = reference(rho, zl, r, cot, c, flank);
                    assert!(
                        (got - want).abs() < 3e-3,
                        "r {r} c {c} flank {flank} at ({rho}, {zl}): {got} vs {want}"
                    );
                    count += 1;
                }
            }
        }
        assert!(count > 15_000, "{count} samples");
    }

    #[test]
    fn the_dovetail_distance_reads_landmarks_and_wider_flank_exactly() {
        let cot = 1.0 / 60.0_f64.to_radians().tan();
        let (r, c, flank) = (9.525, 3.0, 9.525);
        // Bottom corner arc equator (r, c): zero distance.
        assert!(dove_dist(r, c, r, cot, c, flank).abs() < 1e-12);
        // Outside bottom flat corner (k, 0) along 45 deg down-out from center (k, c):
        // distance from center is 2*sqrt(2) * ..., pick (r - c + 3, -1) -> dist 1 below flat.
        assert!((dove_dist(r - c, -1.0, r, cot, c, flank) - 1.0).abs() < 1e-12);
        // At s = 6 mm, rounded flank radius is 8.257 vs sharp 6.061:
        // query at rho = 7.5 is inside the rounded solid and outside the sharp one.
        assert!(dove_dist(7.5, 6.0, r, cot, c, flank) < -0.5);
        assert!(dove_dist(7.5, 6.0, r, cot, 0.0, flank) > 1.0);
    }
}
