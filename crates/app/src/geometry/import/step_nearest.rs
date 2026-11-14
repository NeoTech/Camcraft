//! A second, global parameter search for a boundary point of a B-spline or
//! NURBS STEP face when truck's local search lands off the surface (#1602).
//!
//! Provenance: point inversion on a NURBS surface as in Piegl & Tiller,
//! "The NURBS Book" (2nd ed., sec. 6.1: evaluate the surface at a set of
//! parameters spread over every knot span, start Newton iteration from the
//! nearest, accept the result inside the domain), around truck's published
//! Newton search (ricosjp, Apache-2.0), called unchanged. Nothing from any
//! CAM vendor.
//!
//! Why: truck finds the parameters of a face's boundary points by Newton
//! iteration from the previous point, or, with no hint, from the nearest of
//! a 51 x 51 grid over the whole domain. A thread flank exported as one
//! rational strip that winds fourteen times along v has 150 knot spans
//! there, so that grid samples a turn about three times: the nearest sample
//! is often on the NEIGHBOURING turn, one pitch away. Newton then settles on
//! a local nearest point there - extrapolated far outside the knot domain,
//! 1.7 mm (one pitch) off the boundary - and every later point follows it
//! from that hint. The face is trimmed against a loop that is not its
//! boundary: on the #1602 models a hole of 0.5-3 mm along the flank, which
//! the closed-shell check refuses.
//!
//! The same search goes wrong a second way on a band that winds round a
//! cylinder (a thread's root): the band's straight rulings, extrapolated
//! past the knot domain, meet the band's other turns, so Newton can find
//! the point exactly - at u = 7 on a domain of [0, 1]. The face is then
//! trimmed against a loop through parameters the surface never has, and
//! the mesher joins it with triangles across the turns.
//!
//! The rule: an answer inside the knot domain and within truck's
//! acceptance distance of the point is kept as it is (a pole's free
//! parameter is settled by `step_pole` before this, so it is inside). Any
//! other answer, or none, is compared with Newton started from the nearest
//! samples of a grid with a few samples in every knot span, and the nearest
//! answer INSIDE the knot domain wins when it is on the surface or nearer
//! than the first. A boundary that really runs off its surface keeps
//! truck's answer.

use truck_stepio::r#in::alias::{
    MetricSpace, ParametricSurface, Point3, SPHint2D, SearchNearestParameter, SearchParameter, Surface, D2,
};

use super::step_guard::Metered;

/// truck's acceptance distance for a parameter search (its `TOLERANCE`,
/// in model units): an answer this near the point is on the surface.
const SAME_POINT: f64 = 1.0e-6;
/// The most surface evaluations one sample grid may take.
const MAX_SAMPLES: usize = 4_096;
/// The most samples per knot span in each direction.
const MAX_PER_SPAN: usize = 4;
/// How many of the nearest samples Newton is started from.
const STARTS: usize = 8;
/// Newton iterations per start, as truck's mesher asks.
const TRIALS: usize = 100;

type Domain = ((f64, f64), (f64, f64));

/// truck's answer for `point`, or a nearer one inside the knot domain when
/// truck's lands off the surface. See the module comment.
pub(super) fn rescue(surface: &Surface, point: Point3, found: Option<(f64, f64)>) -> Option<(f64, f64)> {
    let Some(domain) = super::step_pole::knot_domain(surface) else {
        return found;
    };
    let dist = |(u, v): (f64, f64)| surface.subs(u, v).distance(point);
    let first = found.map(dist).filter(|d| d.is_finite());
    if first.is_some_and(|d| d <= SAME_POINT) && found.is_some_and(|uv| inside(uv, domain)) {
        return found;
    }
    let (us, vs) = samples(surface, domain);
    let mut near: Vec<(f64, (f64, f64))> = Vec::with_capacity(us.len() * vs.len());
    for &u in &us {
        for &v in &vs {
            let d = dist((u, v));
            if d.is_finite() {
                near.push((d, (u, v)));
            }
        }
    }
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    let best = near
        .iter()
        .take(STARTS)
        .filter_map(|(_, start)| surface.search_nearest_parameter(point, *start, TRIALS))
        .filter(|uv| inside(*uv, domain))
        .filter_map(|uv| Some((dist(uv), uv)).filter(|(d, _)| d.is_finite()))
        .min_by(|a, b| a.0.total_cmp(&b.0));
    match (best, first) {
        (Some((d, uv)), Some(f)) if d <= SAME_POINT || d < f => Some(uv),
        (Some((_, uv)), None) => Some(uv),
        _ => found,
    }
}

/// `uv` lies in the knot domain, give or take a millionth of its span.
fn inside((u, v): (f64, f64), ((u0, u1), (v0, v1)): Domain) -> bool {
    let su = (u1 - u0) * 1e-6;
    let sv = (v1 - v0) * 1e-6;
    u >= u0 - su && u <= u1 + su && v >= v0 - sv && v <= v1 + sv
}

/// The sample parameters in u and in v: every distinct knot inside the
/// domain, each span split evenly, as many per span as `MAX_SAMPLES` allows.
fn samples(surface: &Surface, ((u0, u1), (v0, v1)): Domain) -> (Vec<f64>, Vec<f64>) {
    let (uk, vk) = match surface {
        Surface::BSplineSurface(s) => (knots(s.uknot_vec(), u0, u1), knots(s.vknot_vec(), v0, v1)),
        Surface::NurbsSurface(s) => {
            let b = s.non_rationalized();
            (knots(b.uknot_vec(), u0, u1), knots(b.vknot_vec(), v0, v1))
        }
        _ => (vec![u0, u1], vec![v0, v1]),
    };
    let spans = (uk.len().saturating_sub(1).max(1), vk.len().saturating_sub(1).max(1));
    let mut per = MAX_PER_SPAN;
    while per > 1 && (spans.0 * per + 1).saturating_mul(spans.1 * per + 1) > MAX_SAMPLES {
        per -= 1;
    }
    (split(&uk, per), split(&vk, per))
}

/// The distinct knots of `kv` inside `[lo, hi]`, with both ends.
fn knots(kv: &truck_stepio::r#in::alias::KnotVec, lo: f64, hi: f64) -> Vec<f64> {
    let (single, _) = kv.to_single_multi();
    let mut out = vec![lo];
    out.extend(single.into_iter().filter(|k| k.is_finite() && *k > lo && *k < hi));
    out.push(hi);
    out
}

/// Every span of `knots` split into `per` equal parts.
fn split(knots: &[f64], per: usize) -> Vec<f64> {
    let mut out: Vec<f64> = knots.first().copied().into_iter().collect();
    for w in knots.windows(2) {
        for i in 1..=per {
            out.push(w[0] + (w[1] - w[0]) * i as f64 / per as f64);
        }
    }
    out
}

// The searches truck's mesher runs on a metered face: in the face's own
// parameters, a pole settled (#1548), a miss retried, then back to the
// mesher's stretched plane (#1602).

impl SearchParameter<D2> for Metered {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(&self, point: Point3, hint: H, trials: usize) -> Option<(f64, f64)> {
        let hint = self.own_hint(hint.into());
        let found = self.inner.search_parameter(point, hint, trials)?;
        // #1548: at a pole the free parameter comes back anywhere.
        let found = super::step_pole::settle(&self.inner, point, previous(hint), found);
        // #1602: an answer outside the knot domain is searched again.
        let found = rescue(&self.inner, point, Some(found))?;
        Some(self.stretch.outer(found))
    }
}

impl SearchNearestParameter<D2> for Metered {
    type Point = Point3;
    fn search_nearest_parameter<H: Into<SPHint2D>>(
        &self,
        point: Point3,
        hint: H,
        trials: usize,
    ) -> Option<(f64, f64)> {
        let hint = self.own_hint(hint.into());
        let found = self.inner.search_nearest_parameter(point, hint, trials);
        let found = found.map(|f| super::step_pole::settle(&self.inner, point, previous(hint), f));
        // #1602: an answer off the surface (a neighbouring turn of a
        // helical strip) or outside the knot domain is searched again.
        let found = rescue(&self.inner, point, found)?;
        Some(self.stretch.outer(found))
    }
}

impl Metered {
    /// A search hint from the mesher's plane in the surface's own parameters.
    fn own_hint(&self, hint: SPHint2D) -> SPHint2D {
        match hint {
            SPHint2D::Parameter(u, v) => {
                let (u, v) = self.own(u, v);
                SPHint2D::Parameter(u, v)
            }
            SPHint2D::Range((u0, u1), (v0, v1)) => {
                let ((u0, v0), (u1, v1)) = (self.own(u0, v0), self.own(u1, v1));
                SPHint2D::Range((u0, u1), (v0, v1))
            }
            SPHint2D::None => SPHint2D::None,
        }
    }
}

/// The previous boundary point's parameter, when the search was given one.
fn previous(hint: SPHint2D) -> Option<(f64, f64)> {
    match hint {
        SPHint2D::Parameter(u, v) => Some((u, v)),
        _ => None,
    }
}

#[cfg(test)]
#[path = "step_nearest_tests.rs"]
mod tests;
