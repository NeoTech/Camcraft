//! The solidcore side of import (M12.1 stage 3): healing behind the seam,
//! and the port's native B-rep text format (SMLib's ASCII `.smb`) as the
//! importer the corrupt-input corpus drives. STEP stays on truck-stepio;
//! this is the port's own reader, the one format it reads without truck.
//! Provenance: calls solidcore 1.0.1's public API as documented in its
//! rustdoc (`heal::heal_data::p1::api_heal_brep`, `topology::brep::p7::
//! read_from_file`, `topology::brep_data_write`) and its `corpus` example's
//! read/write calls; no SMLib/solidcore internals reimplemented here.

use solidcore::EntityRef;
use solidcore::geom::attribute::{AttributeStore, ObjectKey};
use solidcore::geom::object::ObjectRef;
use solidcore::heal::heal_data::p1::api_heal_brep;
use solidcore::math::Point3;
use solidcore::topology::brep::p7::read_from_file;
use solidcore::topology::brep::p10::TopologyEnv;
use solidcore::topology::brep_data::BrepData;
use solidcore::topology::brep_data_write::FromBrepEnv;
use solidcore::topology::database_io::FileType;
use solidcore::topology::topology::NotifyRecords;
use solidcore::topology::{Brep, EdgeId, EdgeuseId, FaceId};

use crate::solidcore_impl::{SolidcoreKernel, checked, failed};
use crate::{Kernel, KernelError};

const NAME: &str = "solidcore";

/// Largest native file the seam hands the reader (64 MiB). The reader holds
/// the whole file and its parsed copy in memory; a larger input is refused
/// by name rather than risked.
pub const MAX_SMB_BYTES: usize = 64 << 20;

/// Geometry copies the reader makes get identities from here up, clear of
/// the topology identities [`key_of`] hands out (the corpus example's split).
const GEOMETRY_KEYS: u64 = 1 << 62;

/// A distinct identity per topology object of the one B-rep the seam reads
/// or writes (the reader and writer key their records by it). The type codes
/// are opaque tags here; only distinctness matters.
fn key_of(e: EntityRef) -> ObjectRef {
    let (type_code, kind, i) = match e {
        EntityRef::Brep(x) => (16011, 0u64, x.index()),
        EntityRef::Region(x) => (16012, 1, x.index()),
        EntityRef::Shell(x) => (16013, 2, x.index()),
        EntityRef::Faceuse(x) => (16022, 3, x.index()),
        EntityRef::Face(x) => (16002, 4, x.index()),
        EntityRef::Loopuse(x) => (16004, 5, x.index()),
        EntityRef::Loop(x) => (16003, 6, x.index()),
        EntityRef::Edgeuse(x) => (16005, 7, x.index()),
        EntityRef::Edge(x) => (16016, 8, x.index()),
        EntityRef::Vertexuse(x) => (16006, 9, x.index()),
        EntityRef::Vertex(x) => (16021, 10, x.index()),
    };
    ObjectRef {
        type_code,
        key: ObjectKey((1u64 << 36) | ((kind + 1) << 32) | u64::from(i)),
    }
}

/// The healing behind [`Kernel::heal`]. The port's heal reports success
/// whatever its healer found (SMLib's `HealBrep` ignores every status but
/// "unported"), so the seam judges the RESULT: it must pass the integrity
/// and closed-manifold checks, and keep a positive finite volume.
pub(crate) fn heal(k: &mut SolidcoreKernel, mut s: Brep) -> Result<Brep, KernelError> {
    if s.faces().is_empty() {
        return Err(KernelError::InvalidInput {
            op: "heal",
            detail: "empty B-rep (no faces)".into(),
        });
    }
    // The port's stitch doubles its tolerance from a tenth of the B-rep's
    // until gaps close; from zero or below it never would (upstream
    // HANG-24, bounded in the port by MAX_TOL_DOUBLINGS). Refuse it here.
    let tol = s.tolerance();
    if !(tol.is_finite() && tol > 0.0) {
        return Err(KernelError::InvalidInput {
            op: "heal",
            detail: format!("B-rep tolerance must be finite and > 0, got {tol}"),
        });
    }
    api_heal_brep(&mut k.k, Some(&mut s)).map_err(|e| failed("heal", &e))?;
    let mut s = checked("heal", s)?;
    k.measure(&mut s)?;
    // The port heals a vertex-edge gap by growing the vertex's tolerance to
    // cover it, not by moving the vertex (bench reading, tests/heal_import.rs):
    // any gap comes back "healed". The seam keeps only gaps under the limit.
    let (gap, _) = vertex_gaps(k, &s)?;
    if gap > HEAL_GAP_LIMIT {
        return Err(KernelError::InvalidResult {
            kernel: NAME,
            op: "heal",
            detail: format!(
                "a vertex sits {gap:.3e} off its edges (limit {HEAL_GAP_LIMIT:e}): \
                 the model is too far apart to heal"
            ),
        });
    }
    Ok(s)
}

/// The largest vertex-edge gap a healed solid may keep (model units, mm).
/// The port closes a gap of any size by growing tolerances, so the seam caps
/// it: a sub-micron gap is an export round-off; a gap past this is a model
/// that is not closed (docs/decisions/27-solidcore-healing.md).
pub const HEAL_GAP_LIMIT: f64 = 1e-3;

/// The largest distance from a vertex to the nearer end of an edge curve it
/// bounds, and the largest vertex tolerance, over the whole B-rep (edges
/// without a 3D curve are skipped). A healed
/// model whose vertices sit off their edges (or whose tolerances were grown
/// to swallow the gap) is not a model a toolpath may trust.
pub fn vertex_gaps(k: &SolidcoreKernel, s: &Brep) -> Result<(f64, f64), KernelError> {
    let op = "vertex_gaps";
    let dist = |a: Point3, b: Point3| {
        ((a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)).sqrt()
    };
    let mut gap = 0.0_f64;
    let mut tol = 0.0_f64;
    for (e, edge) in s.edges().iter() {
        // A boolean's intersection edges may carry only UV trims, no 3D
        // curve: there is no curve end to measure against, so the edge is
        // skipped (the integrity check has already passed on it).
        let Some(curve) = edge.curve.as_ref() else {
            continue;
        };
        let p0 = curve
            .evaluate_point(&k.k, edge.interval.min())
            .map_err(|e| failed(op, &e))?;
        let p1 = curve
            .evaluate_point(&k.k, edge.interval.max())
            .map_err(|e| failed(op, &e))?;
        let (a, b) =
            solidcore::topology::edge::start_end_vertices(s, e).map_err(|e| failed(op, &e))?;
        for v in [a, b].into_iter().flatten() {
            let vx = s.vertex(v).map_err(|e| failed(op, &e))?;
            gap = gap.max(dist(vx.point, p0).min(dist(vx.point, p1)));
            tol = tol.max(vx.tolerance);
        }
    }
    if gap.is_finite() && tol.is_finite() {
        Ok((gap, tol))
    } else {
        Err(KernelError::InvalidResult {
            kernel: NAME,
            op,
            detail: format!("non-finite gap {gap} or tolerance {tol}"),
        })
    }
}

/// Writes `s` in the port's native ASCII B-rep format.
pub fn write_smb(k: &SolidcoreKernel, s: &Brep) -> Result<Vec<u8>, KernelError> {
    let store = AttributeStore::default();
    let records = NotifyRecords::new();
    // The writer keeps only the surfaces a face owns (SMLib's `GetSurfaces`),
    // by comparing the surface's recorded owner with the face's identity; a
    // B-rep the port built names its faces its own way, so a face's identity
    // here IS its surface's recorded owner, else the seam's own key.
    let face_key = |e: EntityRef| match e {
        EntityRef::Face(f) => s
            .face(f)
            .ok()
            .and_then(|x| x.surface.as_ref()?.owner())
            .unwrap_or_else(|| key_of(e)),
        _ => key_of(e),
    };
    let env = FromBrepEnv {
        attributes: &store,
        key_of: &face_key,
        records: &records,
        curve_key: &|_: EdgeId| ObjectKey(0),
        uv_curve_key: &|_: EdgeuseId| ObjectKey(0),
        surface_key: &|_: FaceId| ObjectKey(0),
    };
    let mut all = Vec::new();
    let mut data = BrepData::from_brep(s, &env, &mut all).map_err(|e| failed("write_smb", &e))?;
    data.write_to_file_with_nurbs(&k.k, FileType::Ascii)
        .map_err(|e| failed("write_smb", &e))
}

/// Reads one B-rep from the port's native ASCII format, exactly as read: no
/// healing, no solid check (the corpus measures the raw reader). Use
/// [`import_smb`] for a model the rest of the program may trust.
pub fn read_smb(k: &mut SolidcoreKernel, bytes: &[u8]) -> Result<Brep, KernelError> {
    if bytes.is_empty() {
        return Err(KernelError::InvalidInput {
            op: "read_smb",
            detail: "empty file".into(),
        });
    }
    if bytes.len() > MAX_SMB_BYTES {
        return Err(KernelError::InvalidInput {
            op: "read_smb",
            detail: format!(
                "{} bytes is past the {MAX_SMB_BYTES}-byte limit",
                bytes.len()
            ),
        });
    }
    let mut b = Brep::new_with_infinite_region().map_err(|e| failed("read_smb", &e))?;
    let mut attributes = AttributeStore::default();
    let mut records = NotifyRecords::new();
    let mut env = TopologyEnv {
        all_attributes: &[],
        attributes: &mut attributes,
        key_of: &key_of,
        records: &mut records,
        next_key: GEOMETRY_KEYS,
    };
    read_from_file(
        &mut k.k,
        &mut b,
        bytes.to_vec(),
        FileType::Ascii,
        false,
        &mut env,
    )
    .map_err(|e| failed("read_smb", &e))?;
    Ok(b)
}

/// The importer: read, heal, and hand out only a closed solid with a
/// positive finite measure. Every failure is a located `Err`.
pub fn import_smb(k: &mut SolidcoreKernel, bytes: &[u8]) -> Result<Brep, KernelError> {
    let b = read_smb(k, bytes)?;
    heal(k, b).map_err(|e| match e {
        KernelError::InvalidInput { detail, .. } => KernelError::InvalidResult {
            kernel: NAME,
            op: "import_smb",
            detail,
        },
        other => other,
    })
}
