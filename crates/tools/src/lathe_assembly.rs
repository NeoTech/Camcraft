//! #3643: a lathe tool assembly - a turning holder or boring bar with
//! its insert (and any adapters) - resolved to what the turning laws
//! read: the nose centre as the tip of the turning tool frame, the
//! orientation (LinuxCNC 1-8), the nose radius and the other turning
//! values of the insert and seat, and the clearance bands of the
//! holder body ([`LatheClearance`]). The frames come from the spindle
//! chain ([`crate::tool_assembly`]); only the tip, the turning frame and
//! the body differ. Our own design over the public ISO 13399 roles (MCS,
//! CSW, the cutting reference point), the ISO 1832 / ISO 5608 codes and
//! the LinuxCNC lathe orientation chart; no vendor source or file
//! consulted.
//!
//! ## The tip
//!
//! An insert's cutting point (its stated CIP, else its seat frame) is
//! its cutting reference point (CRP): the control point of the
//! orientation chart, where the nose's tangent lines parallel to X and
//! Z meet. The nose centre is that point less `r * tip_offset(q)`
//! ([`orientation::tip_offset`]), read in the turning frame. The nose
//! centre is the tip, as [`crate::lathe_holder`] states it.
//!
//! ## The law
//!
//! An assembly is resolved whole or refused by name: every refusal of
//! the spindle chain (fits, frames, ports, cycles), a cutting item that
//! is no insert, an insert with no nose radius, an orientation off 1-8
//! or one that cannot be derived, a revolved body (a lathe body is a box
//! or a mesh), a holder body that reaches past the insert's cutting
//! lines within one insert of the nose (a wrong hand, orientation or
//! seat frame), and every refusal of the clearance bands - on the bodies of
//! all components together, never on one alone. The insert's own plate
//! is left out of the bands: the turning laws judge it by its cut model.

use crate::component::{ComponentKind, CuttingItem};
use crate::connection_code::ConnectionCode;
use crate::insert_codes::{Hand, InsertShape};
use crate::insert_iso1832::Corner;
use crate::lathe_holder::{LatheBodyError, LatheClearance, LathePlacement};
use crate::lathe_insert_zone::{MAX_INSERT_REACH, clip_axis};
use crate::orientation;
use crate::tool_assembly::{AssemblyNode, placed};

#[path = "lathe_assembly_error.rs"]
mod error;
pub use error::LatheAssemblyError;

type Tri = [[f64; 3]; 3];

/// How far (mm) a holder body may stand past the insert's cutting
/// lines: float dust only.
pub const EDGE_TOL: f64 = 1e-6;

/// An axis-aligned box in a component's own frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyBox {
    pub lo: [f64; 3],
    pub hi: [f64; 3],
}

/// A lathe component's body, as facets in its own frame.
#[derive(Clone, Debug, PartialEq)]
pub struct LatheBody {
    tris: Vec<Tri>,
}

impl LatheBody {
    /// The closed surface of `boxes` (twelve facets each). Refused by
    /// name: no box, a non-finite corner, a box with no volume.
    pub fn boxes(boxes: &[BodyBox]) -> Result<Self, LatheBodyError> {
        if boxes.is_empty() {
            return Err(LatheBodyError::Empty);
        }
        let mut tris = Vec::with_capacity(boxes.len() * 12);
        for b in boxes {
            if !b.lo.iter().chain(&b.hi).all(|v| v.is_finite()) {
                return Err(LatheBodyError::NonFinite);
            }
            if (0..3).any(|k| b.hi[k] - b.lo[k] <= 1e-9) {
                return Err(LatheBodyError::Degenerate);
            }
            tris.extend(box_facets(b));
        }
        Ok(Self { tris })
    }

    /// Facets as given. Refused by name: none, or a non-finite one.
    pub fn mesh(tris: Vec<Tri>) -> Result<Self, LatheBodyError> {
        if tris.is_empty() {
            return Err(LatheBodyError::Empty);
        }
        if !tris.iter().flatten().flatten().all(|v| v.is_finite()) {
            return Err(LatheBodyError::NonFinite);
        }
        Ok(Self { tris })
    }

    #[must_use]
    pub fn facets(&self) -> &[Tri] {
        &self.tris
    }
}

/// One component of a lathe stack and its lathe body.
#[derive(Clone, Debug, PartialEq)]
pub struct LatheNode {
    pub node: AssemblyNode,
    pub body: Option<LatheBody>,
}

/// How the root's frame sits on the lathe: the root-frame directions
/// that are +x (radial, away from the spindle axis) and +z (along the
/// spindle, away from the chuck).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatheStation {
    pub radial: [f64; 3],
    pub along: [f64; 3],
}

impl LatheStation {
    /// A shank across the spindle (an OD holder on a tool post): its
    /// back end (the root's +Z) points away from the axis.
    pub const RADIAL_SHANK: Self = Self { radial: [0.0, 0.0, 1.0], along: [1.0, 0.0, 0.0] };
    /// A shank along the spindle (a boring bar): its back end points
    /// away from the chuck.
    pub const AXIAL_SHANK: Self = Self { radial: [1.0, 0.0, 0.0], along: [0.0, 0.0, 1.0] };
}

/// The turning values of the insert and its seat.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatheParams {
    /// The nose radius (mm): the insert's corner, half a round insert.
    pub nose_radius: f64,
    /// The LinuxCNC orientation, 1-8.
    pub orientation: u8,
    /// Whether the caller stated the orientation (else it is derived).
    pub stated: bool,
    /// The seat's cutting edge angle (degrees, ISO 5608 position 3).
    pub approach_deg: Option<f64>,
    /// The seat's hand.
    pub hand: Hand,
    /// The insert's corner angle (degrees); `None` for a round insert.
    pub insert_angle_deg: Option<f64>,
    /// The insert's clearance angle (degrees).
    pub clearance_deg: Option<f64>,
    /// The insert's inscribed circle (mm), where the code states it.
    pub ic_mm: Option<f64>,
}

/// A resolved lathe assembly.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedLathe {
    pub params: LatheParams,
    /// The root frame placed in the turning frame: `tip` is the nose
    /// centre, in root coordinates.
    pub placement: LathePlacement,
    /// The CRP (the control point) in the turning frame.
    pub crp: [f64; 3],
    /// The clearance bands of every body but the insert's; `None` when
    /// no component has a body.
    pub clearance: Option<LatheClearance>,
    /// The facets the bands were read from: every body but the insert's,
    /// placed in the turning frame (empty when no component has one).
    pub placed: Vec<Tri>,
}

/// Resolve `nodes` on `station` (the module's law). `orientation`
/// overrides the derived code.
pub fn resolve_lathe(
    nodes: &[LatheNode],
    station: LatheStation,
    orientation: Option<u8>,
) -> Result<ResolvedLathe, LatheAssemblyError> {
    let plain: Vec<AssemblyNode> = nodes.iter().map(|n| n.node.clone()).collect();
    let (frames, cut, cip) = placed(&plain)?;
    let cutter = &nodes[cut].node;
    let name = || cutter.component.name().to_owned();
    let ComponentKind::CuttingItem(CuttingItem::Insert(spec)) = cutter.component.kind() else {
        return Err(LatheAssemblyError::NotInsert { name: name() });
    };
    let seat = seat_of(&plain, cut).ok_or_else(|| LatheAssemblyError::NoSeat { name: name() })?;
    // A round insert's size code is its diameter (mm).
    let round = (spec.corner == Corner::Round).then(|| f64::from(spec.size));
    let ic_mm = spec.inscribed_circle_mm().or(round);
    let nose_radius = match spec.corner {
        Corner::Radius(t) if t > 0 => f64::from(t) / 10.0,
        Corner::Round => round.map_or(0.0, |d| d * 0.5),
        _ => 0.0,
    };
    if !(nose_radius.is_finite() && nose_radius > 0.0) {
        return Err(LatheAssemblyError::NoNose { name: name(), code: spec.to_string() });
    }
    let (q, stated) = match orientation {
        Some(q) if (1..=8).contains(&q) => (q, true),
        Some(q) => return Err(LatheAssemblyError::Orientation { q }),
        None => (derive(&plain, seat.hand, spec.shape)?, false),
    };
    let (ox, oz) = orientation::tip_offset(q).ok_or(LatheAssemblyError::Orientation { q })?;
    let axes = LathePlacement { radial: station.radial, along: station.along, tip: [0.0; 3] };
    let [bx, _, bz] = axes.basis().map_err(LatheAssemblyError::Station)?;
    let c = cip.origin();
    let r = nose_radius;
    let tip: [f64; 3] = core::array::from_fn(|k| c[k] - r * (ox * bx[k] + oz * bz[k]));
    let placement = LathePlacement { tip, ..axes };
    placement.basis().map_err(LatheAssemblyError::Station)?;
    let mut all: Vec<Tri> = Vec::new();
    for (i, (n, f)) in nodes.iter().zip(&frames).enumerate() {
        if i == cut {
            continue; // the insert plate: judged by its cut model
        }
        let named = || n.node.component.name().to_owned();
        if n.node.geometry.body().is_some() {
            return Err(LatheAssemblyError::SpunBody { name: named() });
        }
        let Some(body) = &n.body else { continue };
        let root: Vec<Tri> = body.facets().iter().map(|t| t.map(|p| f.map_point(p))).collect();
        let local = placement.place(&root).map_err(LatheAssemblyError::Station)?;
        for p in local.iter().filter_map(near_tip).flatten() {
            for (axis, o, v) in [('X', ox, p[0]), ('Z', oz, p[2])] {
                if o != 0.0 && o * v > r + EDGE_TOL {
                    return Err(LatheAssemblyError::BodyPastEdge { name: named(), axis, reach: o * v, nose: r });
                }
            }
        }
        all.extend(local);
    }
    let clearance =
        if all.is_empty() { None } else { Some(LatheClearance::of(&all).map_err(LatheAssemblyError::Clearance)?) };    let params = LatheParams {
        nose_radius,
        orientation: q,
        stated,
        approach_deg: seat.style.approach_deg(),
        hand: seat.hand,
        insert_angle_deg: spec.shape.corner_deg(),
        clearance_deg: spec.clearance.degrees(),
        ic_mm,
    };
    Ok(ResolvedLathe { params, placement, crp: [r * ox, 0.0, r * oz], clearance, placed: all })
}

/// The part of facet `t` within one insert of the nose centre in the
/// turning plane (|x| and |z| up to [`MAX_INSERT_REACH`]): where the
/// holder head stands behind the insert. Farther out (a turret block, a
/// sleeve) a body may stand on either side of the cutting lines.
fn near_tip(t: &Tri) -> Option<Vec<[f64; 3]>> {
    let w = MAX_INSERT_REACH;
    let mut poly = t.to_vec();
    for k in [0, 2] {
        poly = clip_axis(&clip_axis(&poly, k, -w, true), k, w, false);
    }
    (poly.len() >= 3).then_some(poly)
}

/// The seat the insert at `cut` sits in.
fn seat_of(nodes: &[AssemblyNode], cut: usize) -> Option<crate::insert_seat::InsertSeatSpec> {
    let m = nodes[cut].mount.as_ref()?;
    let parent = &nodes.get(m.parent)?.component;
    let port = match &m.port {
        Some(p) => p.clone(),
        None => (*parent.ports().first()?).to_owned(),
    };
    parent.seat(&port).copied()
}

/// The orientation from the root's shank (a rectangular shank is an OD
/// holder, a round one a boring bar), the seat's hand (right cuts
/// toward the chuck) and the insert (a round one is neutral).
fn derive(nodes: &[AssemblyNode], hand: Hand, shape: InsertShape) -> Result<u8, LatheAssemblyError> {
    let root = nodes
        .iter()
        .find(|n| n.mount.is_none())
        .ok_or(LatheAssemblyError::Assembly(crate::tool_assembly::AssemblyError::NoRoot))?;
    let outside = match root.component.mcs() {
        Some(ConnectionCode::RectShank { .. }) => true,
        Some(ConnectionCode::Cylindrical { .. } | ConnectionCode::Weldon { .. }) => false,
        other => {
            let coupling = other.map_or_else(|| "none".to_owned(), ToString::to_string);
            return Err(LatheAssemblyError::NeedOrientation { name: root.component.name().to_owned(), coupling });
        }
    };
    let hand = if shape == InsertShape::R { Hand::N } else { hand };
    Ok(match (outside, hand) {
        (true, Hand::R) => 2,
        (true, Hand::L) => 1,
        (true, Hand::N) => 6,
        (false, Hand::R) => 3,
        (false, Hand::L) => 4,
        (false, Hand::N) => 8,
    })
}

/// The twelve facets of a box.
fn box_facets(b: &BodyBox) -> [Tri; 12] {
    let p = |i: usize| -> [f64; 3] {
        [
            if i & 1 == 0 { b.lo[0] } else { b.hi[0] },
            if i & 2 == 0 { b.lo[1] } else { b.hi[1] },
            if i & 4 == 0 { b.lo[2] } else { b.hi[2] },
        ]
    };
    let quad = |a, b, c, d| [[p(a), p(b), p(c)], [p(a), p(c), p(d)]];
    let faces =
        [quad(0, 2, 3, 1), quad(4, 5, 7, 6), quad(0, 1, 5, 4), quad(2, 6, 7, 3), quad(0, 4, 6, 2), quad(1, 3, 7, 5)];
    let mut out = [[[0.0; 3]; 3]; 12];
    for (k, f) in faces.iter().flatten().enumerate() {
        out[k] = *f;
    }
    out
}
