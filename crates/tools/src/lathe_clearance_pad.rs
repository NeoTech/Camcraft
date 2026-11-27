//! #3643: a lathe clearance description grown by a distance - the
//! certified chord deviation of a meshed STEP body, so the bands are
//! never smaller than the steel that the facets stand for. Our own
//! construction (a Minkowski grow of fine bands by a box of half-side
//! `d`, read on the same band count); no vendor source consulted.
//!
//! Why it holds: each point of the steel is within `d` of a facet point.
//! The grown bands cover `[z0 - d, z_hi + d]` with the same count `n`, so
//! each new band is `s' = s + 2d/n` long. A steel point at height `z` in
//! new band `k` has its facet point within `d` of `z`, which lies in an
//! old band `j` with `|j - k| <= m` when `m * s >= 2d`. The new band
//! takes the union of those old bands, each span widened by `d`.

use super::{LatheBand, LatheClearance};

impl LatheClearance {
    /// This description grown by `d` mm in every direction. `d` must be
    /// a finite number of 0 or more; 0 (or a non-finite or negative
    /// value, which a caller has refused before) gives the bands back
    /// unchanged.
    #[must_use]
    pub fn padded(&self, d: f64) -> Self {
        let n = self.bands.len();
        if !d.is_finite() || d <= 0.0 || n == 0 || self.step.is_nan() || self.step <= 0.0 {
            return self.clone();
        }
        // n <= MAX_FINE_BANDS, so m is clamped to a small count and the
        // cast is exact.
        let m = ((2.0 * d / self.step).ceil().clamp(1.0, n as f64)) as usize;
        let bands = (0..n)
            .map(|k| {
                let lo = k.saturating_sub(m);
                let hi = (k + m).min(n - 1);
                let mut out = LatheBand::EMPTY;
                for b in self.bands[lo..=hi].iter().filter(|b| b.occupied()) {
                    out = LatheBand {
                        x_lo: out.x_lo.min(b.x_lo - d),
                        x_hi: out.x_hi.max(b.x_hi + d),
                        y_lo: out.y_lo.min((b.y_lo - d).max(0.0)),
                        y_hi: out.y_hi.max(b.y_hi + d),
                    };
                }
                out
            })
            .collect();
        let h = self.step * n as f64;
        Self { z0: self.z0 - d, step: (h + 2.0 * d) / n as f64, bands }
    }
}
