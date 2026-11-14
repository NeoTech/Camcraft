//! A metered wrapper around the STEP importer's surfaces, so one face whose
//! tessellation never converges ends in a refusal by name instead of a
//! hang (#1536).
//!
//! Provenance: our own design (an evaluation budget on a parametric
//! surface); the subdivision it meters is truck's published
//! `parameter_division` (ricosjp, Apache-2.0), called unchanged. Nothing
//! from any CAM vendor.
//!
//! Why it is needed: truck's surface subdivision splits every parameter
//! cell whose midpoint stands off the bilinear patch of its corners by more
//! than the tolerance, and recurses until no cell splits. A B-spline that is
//! not continuous - an interior knot repeated past its degree - keeps one
//! cell "far" at every depth, so the recursion never ends (it outlives the
//! float resolution: a split at an unchanged midpoint still grows the
//! list). So does a range that runs far off the knot domain; the one known
//! source of that, a boundary point on a collapsed patch side, is settled
//! in `step_pole` (#1548) before it reaches the subdivision. It runs inside a rayon task we cannot
//! interrupt, and no facet is ever produced, so the facet budget never
//! fires.
//!
//! How it is bounded: each face's surface is wrapped in [`Metered`], which
//! counts every point truck evaluates on it during one tessellation pass.
//! B-spline and NURBS faces are subdivided by truck's own generic algorithm
//! called on the wrapper (exactly what their own `parameter_division` runs,
//! so the evaluations - and the mesh - are identical); other surface kinds
//! keep their specialised division. Past the cap the wrapper answers every
//! evaluation with one fixed point: no cell is then "far", the subdivision
//! ends within one more sweep, and the importer refuses the model naming
//! the face - unless the trip came on a finer pass over a face proven
//! continuous, which keeps the coarser mesh (#1568). No thread is spawned
//! and none is left behind.

use std::ops::Bound;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use truck_stepio::r#in::alias::{
    algo, ParameterDivision2D, ParametricSurface, ParametricSurface3D, ParameterRange, Point3,
    SPHint2D, SearchNearestParameter, SearchParameter, Surface, Vector3, D2,
};
use truck_topology::compress::{CompressedFace, CompressedShell};

use super::ImportError;

/// Surface evaluations one face may spend in one tessellation pass.
///
/// Measured (#1568; M5's 75-file corpus of testdata and users' models,
/// release): a healthy face needs far more than its facet count suggests -
/// a trimmed NURBS face is divided over its whole untrimmed range. Peaks on
/// one face in one pass: 32050.STEP 23.1 M at the default facet budget,
/// socket-head screws 16-19 M, and 10208.STEP's face #1073 28.07 M at the
/// 60 M budget (`step_budget::CEILING`), where it converges - the old 25 M
/// cap refused it. 150 M is 5.3x that heaviest healthy face, and ends a
/// runaway face in 10-15 s (the #1536 fixture 10 s, sc_logo.step's face
/// #836 15 s). It is fixed, not scaled with the facet budget: the budget
/// does not bound one face's evaluations, a low budget must not lower the
/// cap, and a broken face's time must not grow with a user setting.
/// Headroom is not the only line: a REFINEMENT pass that trips on a face
/// proven continuous keeps the coarser pass's mesh (`step_continuity`), so
/// such a face is never refused once a pass has met.
pub const FACE_EVAL_CAP: u64 = 150_000_000;

/// One face's evaluation count for the current pass.
#[derive(Debug)]
pub(super) struct Meter {
    used: AtomicU64,
    cap: u64,
    /// Some pass of this import divided the surface over a range outside
    /// its own parameter range (#1568: sc_logo's face #836 was asked for u
    /// from -2e51, a trimming bound gone wild), where its continuity is not
    /// proven - a rational surface extrapolated can meet a pole. Sticky
    /// across passes: truck derives the range from each pass's tolerance,
    /// so the coarser pass whose mesh a fallback would keep may be the one
    /// that went wild while the finer pass that tripped stayed in range.
    outside: AtomicBool,
}

impl Meter {
    fn new(cap: u64) -> Self {
        Self { used: AtomicU64::new(0), cap, outside: AtomicBool::new(false) }
    }
    /// Count one evaluation; `false` once the cap is spent.
    fn spend(&self) -> bool {
        self.used.fetch_add(1, Ordering::Relaxed) < self.cap
    }
    fn tripped(&self) -> bool {
        self.used.load(Ordering::Relaxed) > self.cap
    }
    /// A new pass: the whole budget again. `outside` stays set.
    fn reset(&self) {
        self.used.store(0, Ordering::Relaxed);
    }
}

/// A STEP surface whose evaluations are counted against a [`Meter`].
#[derive(Clone, Debug)]
pub(super) struct Metered {
    inner: Surface,
    meter: Arc<Meter>,
}

impl Metered {
    pub(super) fn new(inner: Surface, cap: u64) -> Self {
        Self { inner, meter: Arc::new(Meter::new(cap)) }
    }
}

/// The surface kind, for a refusal a user can act on.
pub(super) fn kind(s: &Surface) -> &'static str {
    match s {
        Surface::ElementarySurface(_) => "elementary surface",
        Surface::SweptCurve(_) => "swept surface",
        Surface::BSplineSurface(_) => "B-spline surface",
        Surface::NurbsSurface(_) => "NURBS surface",
    }
}

impl ParametricSurface for Metered {
    type Point = Point3;
    type Vector = Vector3;
    fn subs(&self, u: f64, v: f64) -> Point3 {
        // Past the cap every point is the same point: the subdivision then
        // sees no cell off its patch and stops. The pass is refused after.
        if self.meter.spend() {
            self.inner.subs(u, v)
        } else {
            Point3::new(0.0, 0.0, 0.0)
        }
    }
    fn uder(&self, u: f64, v: f64) -> Vector3 {
        self.inner.uder(u, v)
    }
    fn vder(&self, u: f64, v: f64) -> Vector3 {
        self.inner.vder(u, v)
    }
    fn uuder(&self, u: f64, v: f64) -> Vector3 {
        self.inner.uuder(u, v)
    }
    fn uvder(&self, u: f64, v: f64) -> Vector3 {
        self.inner.uvder(u, v)
    }
    fn vvder(&self, u: f64, v: f64) -> Vector3 {
        self.inner.vvder(u, v)
    }
    fn parameter_range(&self) -> (ParameterRange, ParameterRange) {
        self.inner.parameter_range()
    }
    fn u_period(&self) -> Option<f64> {
        self.inner.u_period()
    }
    fn v_period(&self) -> Option<f64> {
        self.inner.v_period()
    }
}

impl ParametricSurface3D for Metered {
    fn normal(&self, u: f64, v: f64) -> Vector3 {
        self.inner.normal(u, v)
    }
}

impl ParameterDivision2D for Metered {
    fn parameter_division(&self, range: ((f64, f64), (f64, f64)), tol: f64) -> (Vec<f64>, Vec<f64>) {
        match &self.inner {
            // truck's B-spline and NURBS surfaces divide with exactly this
            // generic algorithm; running it on the wrapper routes every
            // probe through the meter and changes nothing else.
            Surface::BSplineSurface(_) | Surface::NurbsSurface(_) => {
                let (ur, vr) = self.inner.parameter_range();
                if !within(range.0, ur) || !within(range.1, vr) {
                    self.meter.outside.store(true, Ordering::Relaxed);
                }
                let division = algo::surface::parameter_division(self, range, tol);
                // A tripped face is refused after the pass; handing truck
                // the grid it grew (millions of points down the runaway
                // seam) would only make it triangulate them first.
                if self.meter.tripped() {
                    let ((u0, u1), (v0, v1)) = range;
                    return (vec![u0, u1], vec![v0, v1]);
                }
                division
            }
            other => other.parameter_division(range, tol),
        }
    }
}

/// `asked` lies inside `own` (a surface's parameter range), give or take
/// a millionth of its span for a trimming curve's round-off.
fn within((lo, hi): (f64, f64), own: ParameterRange) -> bool {
    let slack = |a: f64, b: f64| 1e-6 * (b - a).abs().max(1.0);
    let (lo_ok, hi_ok) = match own {
        (Bound::Included(a) | Bound::Excluded(a), Bound::Included(b) | Bound::Excluded(b)) => {
            (lo >= a - slack(a, b), hi <= b + slack(a, b))
        }
        (Bound::Included(a) | Bound::Excluded(a), Bound::Unbounded) => (lo >= a - slack(a, a), true),
        (Bound::Unbounded, Bound::Included(b) | Bound::Excluded(b)) => (true, hi <= b + slack(b, b)),
        (Bound::Unbounded, Bound::Unbounded) => (true, true),
    };
    lo.is_finite() && hi.is_finite() && lo_ok && hi_ok
}

impl SearchParameter<D2> for Metered {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(&self, point: Point3, hint: H, trials: usize) -> Option<(f64, f64)> {
        let hint = hint.into();
        let found = self.inner.search_parameter(point, hint, trials)?;
        // #1548: at a pole the free parameter comes back anywhere.
        Some(super::step_pole::settle(&self.inner, point, previous(hint), found))
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
        let hint = hint.into();
        let found = self.inner.search_nearest_parameter(point, hint, trials)?;
        Some(super::step_pole::settle(&self.inner, point, previous(hint), found))
    }
}

/// The previous boundary point's parameter, when the search was given one.
fn previous(hint: SPHint2D) -> Option<(f64, f64)> {
    match hint {
        SPHint2D::Parameter(u, v) => Some((u, v)),
        _ => None,
    }
}

type StepShell<S> = CompressedShell<Point3, truck_stepio::r#in::alias::Curve3D, S>;

/// A shell with every face metered, and the meters in face order.
pub(super) struct Guarded {
    pub shell: StepShell<Metered>,
    meters: Vec<Arc<Meter>>,
    kinds: Vec<&'static str>,
    continuous: Vec<bool>,
}

/// A face that ran past the cap in a pass.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Trip {
    /// The face's index in the shell (0-based).
    pub index: usize,
    pub kind: &'static str,
    /// Its surface is proven continuous (`step_continuity`): the
    /// subdivision would have ended, it was only expensive.
    pub continuous: bool,
}

impl Guarded {
    pub(super) fn new(shell: StepShell<Surface>, cap: u64) -> Self {
        let mut meters = Vec::with_capacity(shell.faces.len());
        let mut kinds = Vec::with_capacity(shell.faces.len());
        let mut continuous = Vec::with_capacity(shell.faces.len());
        let faces = shell
            .faces
            .into_iter()
            .map(|f| {
                kinds.push(kind(&f.surface));
                continuous.push(super::step_continuity::proven_continuous(&f.surface));
                let surface = Metered::new(f.surface, cap);
                meters.push(Arc::clone(&surface.meter));
                CompressedFace { boundaries: f.boundaries, orientation: f.orientation, surface }
            })
            .collect();
        Self { shell: CompressedShell { vertices: shell.vertices, edges: shell.edges, faces }, meters, kinds, continuous }
    }

    /// Start a new pass: every face gets its whole budget again.
    pub(super) fn reset(&self) {
        self.meters.iter().for_each(|m| m.reset());
    }

    /// The face to report for a pass in which faces ran past the cap. The
    /// pass counts as continuous only if EVERY tripped face is proven and
    /// was never divided outside its range; otherwise the first face that
    /// is not is reported, so the refusal names the face that may jump
    /// (#1568 review: a proven face tripping beside one that may jump must
    /// not let the fallback keep the jump's smear).
    pub(super) fn tripped(&self) -> Option<Trip> {
        let mut trips = self.meters.iter().enumerate().filter(|(_, m)| m.tripped()).map(|(i, _)| self.trip(i));
        let first = trips.next()?;
        Some(if first.continuous { trips.find(|t| !t.continuous).unwrap_or(first) } else { first })
    }

    fn trip(&self, index: usize) -> Trip {
        Trip {
            index,
            kind: self.kinds.get(index).copied().unwrap_or("surface"),
            // Proven on the surface's own range only: a face any pass
            // divided outside that range is not covered by the proof.
            continuous: self.continuous.get(index).copied().unwrap_or(false)
                && self.meters.get(index).is_some_and(|m| !m.outside.load(Ordering::Relaxed)),
        }
    }
}

/// The STEP entity number of each face of `shell`, in the order truck
/// converts them - or `None` when the counts differ (truck drops a face it
/// cannot convert, and the numbering would then name the wrong face).
pub(super) fn face_entities(shell: &truck_stepio::r#in::ShellHolder, faces: usize) -> Option<Vec<u64>> {
    use truck_stepio::r#in::ruststep::{ast::Name, tables::PlaceHolder};
    let ids: Option<Vec<u64>> = shell
        .cfs_faces
        .iter()
        .map(|f| match f {
            PlaceHolder::Ref(Name::Entity(id)) => Some(*id),
            _ => None,
        })
        .collect();
    ids.filter(|ids| ids.len() == faces)
}

/// The refusal for the tripped face of a shell of `faces` faces, naming
/// its STEP entity when the file's face list maps one-to-one. A face
/// proven continuous is only reached here from the first, coarsest pass
/// (a finer one falls back to the coarser mesh), so its text says it is
/// too dense, not broken.
pub(super) fn refusal(trip: Trip, faces: usize, entity: Option<u64>, shell: Option<u64>, cap: u64) -> ImportError {
    let Trip { index, kind, continuous } = trip;
    let face = match entity {
        Some(id) => format!("face #{id} ({kind}, face {} of {faces}", index + 1),
        None => format!("face {} of {faces} ({kind}", index + 1),
    };
    let shell = shell.map(|s| format!(" in shell #{s}")).unwrap_or_default();
    let why = if continuous {
        "did not finish within {cap} evaluations at the coarsest tolerance (a continuous surface, too dense to divide)"
    } else {
        "did not converge within {cap} evaluations (a discontinuous or degenerate surface)"
    };
    let why = why.replace("{cap}", &cap.to_string());
    ImportError::Step(format!(
        "{face}{shell}) could not be tessellated: its surface subdivision {why} - re-export the model, or import an STL"
    ))
}

#[cfg(test)]
#[path = "step_guard_tests.rs"]
mod tests;
