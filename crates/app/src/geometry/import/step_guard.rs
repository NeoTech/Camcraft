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
//! not continuous - an interior knot repeated past its degree, a degenerate
//! corner blend - keeps one cell "far" at every depth, so the recursion
//! never ends (it outlives the float resolution: a split at an unchanged
//! midpoint still grows the list). It runs inside a rayon task we cannot
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
//! the face. No thread is spawned and none is left behind.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use truck_stepio::r#in::alias::{
    algo, ParameterDivision2D, ParametricSurface, ParametricSurface3D, ParameterRange, Point3,
    SPHint2D, SearchNearestParameter, SearchParameter, Surface, Vector3, D2,
};
use truck_topology::compress::{CompressedFace, CompressedShell};

use super::ImportError;

/// Surface evaluations one face may spend in one tessellation pass. A
/// healthy face at the finest chord the importer asks for needs a few
/// million (a 1000 x 1000 grid is ~7M with the subdivision's own probes);
/// a face that never converges reaches it in seconds and is refused.
pub const FACE_EVAL_CAP: u64 = 25_000_000;

/// One face's evaluation count for the current pass.
#[derive(Debug)]
pub(super) struct Meter {
    used: AtomicU64,
    cap: u64,
}

impl Meter {
    fn new(cap: u64) -> Self {
        Self { used: AtomicU64::new(0), cap }
    }
    /// Count one evaluation; `false` once the cap is spent.
    fn spend(&self) -> bool {
        self.used.fetch_add(1, Ordering::Relaxed) < self.cap
    }
    fn tripped(&self) -> bool {
        self.used.load(Ordering::Relaxed) > self.cap
    }
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

impl SearchParameter<D2> for Metered {
    type Point = Point3;
    fn search_parameter<H: Into<SPHint2D>>(&self, point: Point3, hint: H, trials: usize) -> Option<(f64, f64)> {
        self.inner.search_parameter(point, hint, trials)
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
        self.inner.search_nearest_parameter(point, hint, trials)
    }
}

type StepShell<S> = CompressedShell<Point3, truck_stepio::r#in::alias::Curve3D, S>;

/// A shell with every face metered, and the meters in face order.
pub(super) struct Guarded {
    pub shell: StepShell<Metered>,
    meters: Vec<Arc<Meter>>,
    kinds: Vec<&'static str>,
}

impl Guarded {
    pub(super) fn new(shell: StepShell<Surface>, cap: u64) -> Self {
        let mut meters = Vec::with_capacity(shell.faces.len());
        let mut kinds = Vec::with_capacity(shell.faces.len());
        let faces = shell
            .faces
            .into_iter()
            .map(|f| {
                kinds.push(kind(&f.surface));
                let surface = Metered::new(f.surface, cap);
                meters.push(Arc::clone(&surface.meter));
                CompressedFace { boundaries: f.boundaries, orientation: f.orientation, surface }
            })
            .collect();
        Self { shell: CompressedShell { vertices: shell.vertices, edges: shell.edges, faces }, meters, kinds }
    }

    /// Start a new pass: every face gets its whole budget again.
    pub(super) fn reset(&self) {
        self.meters.iter().for_each(|m| m.reset());
    }

    /// The first face (index, kind) whose pass ran past the cap.
    pub(super) fn tripped(&self) -> Option<(usize, &'static str)> {
        let i = self.meters.iter().position(|m| m.tripped())?;
        Some((i, self.kinds.get(i).copied().unwrap_or("surface")))
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

/// The refusal for face `index` (0-based) of a shell of `faces` faces,
/// naming its STEP entity when the file's face list maps one-to-one.
pub(super) fn refusal(
    index: usize,
    kind: &str,
    faces: usize,
    entity: Option<u64>,
    shell: Option<u64>,
    cap: u64,
) -> ImportError {
    let face = match entity {
        Some(id) => format!("face #{id} ({kind}, face {} of {faces}", index + 1),
        None => format!("face {} of {faces} ({kind}", index + 1),
    };
    let shell = shell.map(|s| format!(" in shell #{s}")).unwrap_or_default();
    ImportError::Step(format!(
        "{face}{shell}) could not be tessellated: its surface subdivision did not converge within {cap} evaluations \
         (a discontinuous or degenerate surface) - re-export the model, or import an STL"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use truck_stepio::r#in::alias::{BSplineSurface, KnotVec};

    /// Degree 1 in u with the knot 1.0 repeated twice: the left half sits
    /// at z = 0, the right half at z = 1, so the surface jumps at u = 1.
    fn discontinuous() -> Surface {
        let uk = KnotVec::from(vec![0.0, 0.0, 1.0, 1.0, 2.0, 2.0]);
        let vk = KnotVec::bezier_knot(1);
        let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
        let cps = vec![
            vec![p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.0)],
            vec![p(1.0, 0.0, 0.0), p(1.0, 1.0, 0.0)],
            vec![p(1.0, 0.0, 1.0), p(1.0, 1.0, 1.0)],
            vec![p(2.0, 0.0, 1.0), p(2.0, 1.0, 1.0)],
        ];
        Surface::BSplineSurface(Box::new(BSplineSurface::new((uk, vk), cps)))
    }

    fn smooth() -> Surface {
        let k = KnotVec::bezier_knot(2);
        let p = |x: f64, y: f64, z: f64| Point3::new(x, y, z);
        let cps = vec![
            vec![p(0.0, 0.0, 0.0), p(0.0, 1.0, 0.5), p(0.0, 2.0, 0.0)],
            vec![p(1.0, 0.0, 0.5), p(1.0, 1.0, 2.0), p(1.0, 2.0, 0.5)],
            vec![p(2.0, 0.0, 0.0), p(2.0, 1.0, 0.5), p(2.0, 2.0, 0.0)],
        ];
        Surface::BSplineSurface(Box::new(BSplineSurface::new((k.clone(), k), cps)))
    }

    #[test]
    fn a_discontinuous_bspline_division_stops_at_the_cap_and_trips_the_meter() {
        let m = Metered::new(discontinuous(), 200_000);
        let started = std::time::Instant::now();
        let (u, v) = m.parameter_division(((0.0, 2.0), (0.0, 1.0)), 1e-3);
        assert!(m.meter.tripped(), "the meter must report the runaway face");
        assert_eq!((u, v), (vec![0.0, 2.0], vec![0.0, 1.0]), "a tripped face hands truck no grid to triangulate");
        assert!(started.elapsed().as_secs() < 10, "bounded: {:?}", started.elapsed());
    }

    #[test]
    fn a_smooth_bspline_divides_exactly_as_truck_does_and_does_not_trip() {
        let s = smooth();
        let m = Metered::new(s.clone(), FACE_EVAL_CAP);
        let range = ((0.0, 1.0), (0.0, 1.0));
        for tol in [0.1, 1e-2, 1e-3] {
            m.meter.reset();
            assert_eq!(m.parameter_division(range, tol), s.parameter_division(range, tol), "tol {tol}");
            assert!(!m.meter.tripped());
        }
    }

    /// No work outlives a refused pass: truck's rayon tasks are joined
    /// before `robust_triangulation` returns, no thread is spawned, so the
    /// runaway face's meter is still once the call is back.
    #[test]
    fn nothing_keeps_running_after_a_refused_pass() {
        use truck_meshalgo::prelude::RobustMeshableShape;
        let text = include_str!("../../../../../testdata/models/bspline_discontinuous.step");
        let exchange = truck_stepio::r#in::ruststep::parser::parse(text).expect("fixture parses");
        let table = truck_stepio::r#in::Table::from_data_section(&exchange.data[0]);
        let shell = table.shell.values().next().expect("one shell");
        let cshell = table.to_compressed_shell(shell).expect("converts");
        // A cap large enough that the pass takes a while: work left running
        // in the background would still be counting when we look.
        let g = Guarded::new(cshell, 2_000_000);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| g.shell.robust_triangulation(0.01)));
        assert_eq!(g.tripped(), Some((5, "B-spline surface")));
        let spent = |g: &Guarded| g.meters.iter().map(|m| m.used.load(Ordering::Relaxed)).sum::<u64>();
        let before = spent(&g);
        std::thread::sleep(std::time::Duration::from_millis(300));
        assert_eq!(spent(&g), before, "a tessellation is still evaluating after the call returned");
    }

    #[test]
    fn the_refusal_names_the_face_its_kind_and_the_cap() {
        let e = refusal(36, "NURBS surface", 62, Some(1234), Some(17), 99).to_string();
        assert!(e.contains("face #1234 (NURBS surface, face 37 of 62 in shell #17)"), "{e}");
        assert!(e.contains("within 99 evaluations"), "{e}");
        let e = refusal(0, "B-spline surface", 1, None, None, 5).to_string();
        assert!(e.contains("face 1 of 1 (B-spline surface)"), "{e}");
    }
}
