//! #3643: a tool component's body from a mesh file - a STEP model
//! (meshed by the kernel import of `camcraft_geometry::step`, with its
//! ISO 13399 coordinate systems from `camcraft_geometry::step_csys`), an
//! STL or an OBJ (with frames the caller stored) - placed by its own
//! mounting frame (MCS, or MCS_INSERT on an insert) and judged as a
//! spindle body (a revolved outline) or a lathe body (facets). Our own
//! design over the public ISO 13399 coordinate-system roles; no vendor
//! source or file consulted.
//!
//! The law: a body is never smaller than the steel. A spindle body is
//! the revolved envelope of every facet ([`HolderProfile::envelope`]),
//! drawn as a stepped outline that only grows it, and checked to enclose
//! it. A meshed STEP body grows by the kernel's certified chord
//! deviation, in both judges. Every refusal names itself
//! ([`MeshBodyError`]); nothing here panics on any input.

use camcraft_geometry::step::{self, StepError};
use camcraft_geometry::step_csys::{self, CsysRole};

use crate::component_geometry::{ComponentBody, ComponentGeometry};
use crate::frame::Frame;
use crate::holder_profile::{merge, HolderBand, HolderProfile};
use crate::lathe_assembly::LatheBody;
use crate::mesh_file::{self, MeshFileError, Tri, MAX_BODY_FACETS, MAX_BODY_FILE_BYTES};
use crate::spun::{SpunProfile, MAX_POINTS, MIN_SEGMENT};

#[path = "mesh_body_error.rs"]
mod error;
pub use error::MeshBodyError;
#[cfg(test)]
#[path = "mesh_body_safety_tests.rs"]
mod safety_tests;

/// The chord (mm) a STEP body is meshed at, or coarser when that does
/// not fit the facet budget. The certified deviation is added back.
pub const BODY_CHORD_MM: f64 = 0.01;
/// The most bands the drawn outline of a spindle body keeps: two points
/// for each band and one at each end fit [`MAX_POINTS`].
const OUTLINE_BANDS: usize = (MAX_POINTS - 2) / 2;
/// Neighbouring outline bands closer than this in radius, or bands
/// shorter than it, are joined at the larger radius, so that no drawn
/// segment is shorter than [`MIN_SEGMENT`].
const TIDY: f64 = 2.0 * MIN_SEGMENT;

/// The file formats a component body is read from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshFormat {
    Step,
    Stl,
    Obj,
}

impl MeshFormat {
    /// The format a file extension names (case ignored), if any.
    #[must_use]
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.trim().trim_start_matches('.').to_ascii_lowercase().as_str() {
            "step" | "stp" => Some(Self::Step),
            "stl" => Some(Self::Stl),
            "obj" => Some(Self::Obj),
            _ => None,
        }
    }
}

/// A named coordinate system of a body, in the file's coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct NamedFrame {
    pub name: String,
    pub role: CsysRole,
    pub frame: Frame,
}

/// A body's facets and frames, in its mounting frame (the component
/// frame of [`ComponentGeometry`]: MCS at the origin, +Z to the machine).
#[derive(Clone, Debug, PartialEq)]
pub struct MeshParts {
    /// Each workpiece-side port (CSW, CSWx, CSWx_y), by its upper-case name.
    pub ports: Vec<(String, Frame)>,
    /// The cutting point (CIP), if the file states one.
    pub cip: Option<Frame>,
    pub facets: Vec<Tri>,
}

/// A component body read from a mesh file.
#[derive(Clone, Debug, PartialEq)]
pub struct MeshBody {
    tris: Vec<Tri>,
    frames: Vec<NamedFrame>,
    refused: Vec<RefusedFrame>,
    deviation_mm: f64,
}

/// A coordinate system the file states but the reader refused (degenerate,
/// non-finite, ambiguous): its role by name, and why, for the refusal.
#[derive(Clone, Debug, PartialEq)]
struct RefusedFrame {
    name: String,
    role: CsysRole,
    said: String,
}

impl MeshBody {
    /// The body in `bytes`, read as `format`.
    pub fn read(format: MeshFormat, bytes: &[u8]) -> Result<Self, MeshBodyError> {
        match format {
            MeshFormat::Step => Self::from_step(bytes),
            MeshFormat::Stl => Self::from_facets(mesh_file::read_stl(bytes).map_err(MeshBodyError::File)?),
            MeshFormat::Obj => Self::from_facets(mesh_file::read_obj(bytes).map_err(MeshBodyError::File)?),
        }
    }

    /// A STEP body: its solids meshed in mm at [`BODY_CHORD_MM`] (or
    /// coarser, inside [`MAX_BODY_FACETS`]), and its named coordinate
    /// systems. A coordinate system that the reader refused is kept as
    /// a reason, for the refusal that needs it.
    pub fn from_step(bytes: &[u8]) -> Result<Self, MeshBodyError> {
        if bytes.is_empty() {
            return Err(MeshBodyError::File(MeshFileError::Empty));
        }
        if bytes.len() > MAX_BODY_FILE_BYTES {
            return Err(MeshBodyError::File(MeshFileError::TooBig { bytes: bytes.len() }));
        }
        let text = std::str::from_utf8(bytes).map_err(|_| MeshBodyError::Step("the file is not valid UTF-8".into()))?;
        let report = step_csys::read_tool_csys(text).map_err(MeshBodyError::Step)?;
        let mesh = step::parse(bytes, MAX_BODY_FACETS, BODY_CHORD_MM).map_err(|e| match e {
            StepError::TooLarge => MeshBodyError::StepOverBudget,
            other => MeshBodyError::Step(other.to_string()),
        })?;
        let tris: Vec<Tri> = mesh.mesh.facets.iter().map(|t| [t.a, t.b, t.c].map(|p| [p.x, p.y, p.z])).collect();
        let mut body = Self::from_facets(tris)?;
        let deviation = mesh.deviation_mm.max(0.0);
        if !deviation.is_finite() {
            return Err(MeshBodyError::Step("the mesher's deviation is not a finite number".into()));
        }
        body.deviation_mm = deviation;
        body.frames = report
            .systems
            .iter()
            .map(|c| {
                let frame = Frame::new(c.origin, c.z, c.x)
                    .map_err(|err| MeshBodyError::Frame { name: c.name.clone(), err })?;
                Ok(NamedFrame { name: c.name.trim().to_ascii_uppercase(), role: c.role, frame })
            })
            .collect::<Result<_, MeshBodyError>>()?;
        body.refused = report
            .refused
            .iter()
            .map(|r| RefusedFrame {
                role: CsysRole::classify(&r.name),
                name: r.name.trim().to_ascii_uppercase(),
                said: format!("{} (#{}): {}", r.name, r.entity, r.reason),
            })
            .collect();
        Ok(body)
    }

    /// Facets as given (an STL or OBJ, or a caller's mesh), with no
    /// frames yet. Refused by name: no facets, over the facet budget, a
    /// non-finite corner.
    pub fn from_facets(tris: Vec<Tri>) -> Result<Self, MeshBodyError> {
        if tris.is_empty() {
            return Err(MeshBodyError::File(MeshFileError::NoFacets));
        }
        if tris.len() > MAX_BODY_FACETS {
            return Err(MeshBodyError::File(MeshFileError::OverBudget { facets: tris.len() }));
        }
        if !tris.iter().flatten().flatten().all(|v| v.is_finite()) {
            return Err(MeshBodyError::File(MeshFileError::NonFinite { line: 0 }));
        }
        Ok(Self { tris, frames: Vec::new(), refused: Vec::new(), deviation_mm: 0.0 })
    }

    /// The same facets with `frames` (name, frame in the file's
    /// coordinates) in place of the file's own: the stored frames of an
    /// STL or OBJ body, classified by their ISO 13399 names.
    pub fn with_frames(mut self, frames: Vec<(String, Frame)>) -> Result<Self, MeshBodyError> {
        self.frames = frames
            .into_iter()
            .map(|(name, frame)| {
                let name = name.trim().to_ascii_uppercase();
                if name.is_empty() {
                    return Err(MeshBodyError::EmptyFrameName);
                }
                Ok(NamedFrame { role: CsysRole::classify(&name), name, frame })
            })
            .collect::<Result<_, _>>()?;
        self.refused.clear();
        Ok(self)
    }

    #[must_use]
    pub fn facets(&self) -> &[Tri] {
        &self.tris
    }

    #[must_use]
    pub fn frames(&self) -> &[NamedFrame] {
        &self.frames
    }

    /// How far (mm) the steel may stand outside the facets: the STEP
    /// mesher's certified deviation, 0 for an STL or OBJ (the file is
    /// the body).
    #[must_use]
    pub fn deviation_mm(&self) -> f64 {
        self.deviation_mm
    }

    /// The facets, ports and cutting point in the mounting frame.
    /// Refused by name: no mounting frame or two, two cutting points, a
    /// port or cutting point that is not below the mounting frame, or a
    /// mounting frame, port or cutting point that the file states but the
    /// reader refused (a stated frame is never read as an absent one).
    pub fn parts(&self) -> Result<MeshParts, MeshBodyError> {
        let used = |r: &CsysRole| matches!(r, CsysRole::Mounting | CsysRole::InProcess | CsysRole::WorkpieceSide { .. });
        let has_mount = self.frames.iter().any(|f| f.role == CsysRole::Mounting);
        // With no mount at all, NoMount lists the refused frames instead.
        if has_mount && let Some(r) = self.refused.iter().find(|r| used(&r.role)) {
            return Err(MeshBodyError::RefusedFrame { name: r.name.clone(), why: r.said.clone() });
        }
        let mounts: Vec<&NamedFrame> = self.frames.iter().filter(|f| f.role == CsysRole::Mounting).collect();
        let mcs = match mounts.as_slice() {
            [one] => one.frame,
            [] => return Err(MeshBodyError::NoMount { found: self.found() }),
            many => return Err(MeshBodyError::TwoMounts(many.iter().map(|f| f.name.clone()).collect())),
        };
        let below = |f: &NamedFrame| {
            let local = mcs.relative(&f.frame);
            let z = local.origin()[2];
            if z < 0.0 { Ok(local) } else { Err(MeshBodyError::NotBelowMount { name: f.name.clone(), z }) }
        };
        let mut ports = Vec::new();
        let mut cips: Vec<(&str, Frame)> = Vec::new();
        for f in &self.frames {
            match f.role {
                CsysRole::WorkpieceSide { .. } => ports.push((f.name.clone(), below(f)?)),
                CsysRole::InProcess => cips.push((&f.name, below(f)?)),
                _ => {}
            }
        }
        let cip = match cips.as_slice() {
            [] => None,
            [(_, one)] => Some(*one),
            many => return Err(MeshBodyError::TwoCutPoints(many.iter().map(|(n, _)| (*n).to_owned()).collect())),
        };
        let facets = self.tris.iter().map(|t| t.map(|p| mcs.unmap_point(p))).collect();
        Ok(MeshParts { ports, cip, facets })
    }

    /// The body as a spindle component: its ports and cutting point, and
    /// a revolved body never smaller than the facets grown by the
    /// deviation.
    pub fn spindle_geometry(&self) -> Result<ComponentGeometry, MeshBodyError> {
        let parts = self.parts()?;
        let body = spun_body(&parts.facets, self.deviation_mm)?;
        ComponentGeometry::new(parts.ports, Some(body), parts.cip).map_err(MeshBodyError::Geometry)
    }

    /// The body as a lathe component: its ports and cutting point with
    /// no revolved body, and its facets as the lathe body, grown by the
    /// deviation in the clearance bands.
    pub fn lathe_geometry(&self) -> Result<(ComponentGeometry, LatheBody), MeshBodyError> {
        let parts = self.parts()?;
        let geometry = ComponentGeometry::new(parts.ports, None, parts.cip).map_err(MeshBodyError::Geometry)?;
        let body = LatheBody::mesh(parts.facets)
            .and_then(|b| b.padded(self.deviation_mm))
            .map_err(MeshBodyError::Lathe)?;
        Ok((geometry, body))
    }

    /// What the file names in place of a mounting frame, for the refusal.
    fn found(&self) -> String {
        let mut said: Vec<String> = self.frames.iter().map(|f| f.name.clone()).collect();
        said.extend(self.refused.iter().map(|r| format!("refused {}", r.said)));
        if said.is_empty() { "it names no coordinate systems".into() } else { format!("it names {}", said.join(", ")) }
    }
}

/// The revolved body of `tris` (in the component frame), grown by `pad`:
/// the envelope bands, drawn as a stepped outline that only grows them,
/// checked to enclose them, with its height 0 at the body's lowest point.
fn spun_body(tris: &[Tri], pad: f64) -> Result<ComponentBody, MeshBodyError> {
    let z0 = tris.iter().flatten().map(|p| p[2]).fold(f64::INFINITY, f64::min);
    if !z0.is_finite() {
        return Err(MeshBodyError::File(MeshFileError::NoFacets));
    }
    let lifted: Vec<Tri> = tris.iter().map(|t| t.map(|p| [p[0], p[1], p[2] - z0])).collect();
    let env = HolderProfile::envelope(&lifted).map_err(MeshBodyError::Envelope)?;
    let grown = grown_bands(env.bands(), pad);
    let bands = outline_bands(&grown);
    let profile = SpunProfile::from_bands(&bands).ok_or(MeshBodyError::Outline)?;
    let drawn = profile.bands();
    // The safety net: every envelope band, grown by the pad outward and
    // at BOTH axial ends (so an internal shoulder grows along the axis
    // too), lies inside the drawn outline over its whole grown length.
    if !grown.iter().all(|b| covers(&drawn, b.above, b.above + b.len, b.r)) {
        return Err(MeshBodyError::Outline);
    }
    ComponentBody::new(profile.points().to_vec(), z0 - pad).map_err(MeshBodyError::Geometry)
}

/// Each envelope band grown by `pad`: its radius by `pad`, and its axial
/// interval by `pad` at both ends. The heights shift up by `pad`, so the
/// lowest grown band starts at 0 (the body's lowest point less the pad).
/// A point within `pad` of the steel is inside one of these bands.
fn grown_bands(env: &[HolderBand], pad: f64) -> Vec<HolderBand> {
    env.iter().map(|x| HolderBand { above: x.above, len: x.len + 2.0 * pad, r: x.r + pad }).collect()
}

/// The union of the `grown` bands as one stepped outline (each step at
/// the largest radius that reaches over it, at least [`TIDY`] from the
/// axis), joined until no step is shorter than [`TIDY`] and at most
/// [`OUTLINE_BANDS`] remain. Each join takes the larger radius: it only
/// grows.
fn outline_bands(grown: &[HolderBand]) -> Vec<HolderBand> {
    const EPS: f64 = 1e-12;
    let mut src: Vec<HolderBand> = grown.to_vec();
    src.sort_by(|a, b| a.above.total_cmp(&b.above));
    let mut cuts: Vec<f64> = src.iter().flat_map(|x| [x.above, x.above + x.len]).collect();
    cuts.sort_by(f64::total_cmp);
    cuts.dedup_by(|a, b| (*a - *b).abs() <= EPS);
    // The running largest top of src[..=i]: every band before the first
    // index whose running top passes `lo` ends at or below `lo`.
    let mut tops = Vec::with_capacity(src.len());
    let mut top = f64::NEG_INFINITY;
    for x in &src {
        top = top.max(x.above + x.len);
        tops.push(top);
    }
    let mut start = 0;
    let mut b = Vec::with_capacity(cuts.len());
    for w in cuts.windows(2) {
        let (lo, hi) = (w[0], w[1]);
        while start < src.len() && tops[start] <= lo + EPS {
            start += 1;
        }
        let r = src[start..]
            .iter()
            .take_while(|x| x.above < hi - EPS)
            .filter(|x| x.above + x.len > lo + EPS)
            .map(|x| x.r)
            .fold(TIDY, f64::max);
        b.push(HolderBand { above: lo, len: hi - lo, r });
    }
    tidy(&mut b);
    let mut b = merge(b, OUTLINE_BANDS);
    tidy(&mut b);
    b
}

fn tidy(b: &mut Vec<HolderBand>) {
    let mut i = 0;
    while i + 1 < b.len() {
        let (x, y) = (b[i], b[i + 1]);
        if x.len < TIDY || y.len < TIDY || (x.r - y.r).abs() < TIDY {
            b[i] = HolderBand { above: x.above, len: (y.above + y.len) - x.above, r: x.r.max(y.r) };
            b.remove(i + 1);
            i = i.saturating_sub(1);
        } else {
            i += 1;
        }
    }
}

/// Do `drawn` bands cover `[lo, hi]` at a radius of `r` or more?
fn covers(drawn: &[HolderBand], lo: f64, hi: f64, r: f64) -> bool {
    const EPS: f64 = 1e-9;
    let mut at = lo;
    for d in drawn.iter().filter(|d| d.above < hi - EPS && d.above + d.len > lo + EPS) {
        if d.above > at + EPS || d.r < r - EPS {
            return false;
        }
        at = at.max(d.above + d.len);
    }
    at >= hi - EPS
}
