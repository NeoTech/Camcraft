//! #3643: a spindle tool assembly - a stack of tool components (#3642)
//! mounted port on port - resolved to what the laws read: the gauge
//! length, the cutting diameter and corner radius, and the judged
//! holder body. Forward kinematics: the root's MCS is the identity,
//! each child's MCS sits on its parent's port, turned about the mating
//! Z and, on a shank mount, moved to its stated stickout. The result is
//! moved so that the cutting tip is the origin of the tool frame. Our
//! own design over the public ISO 13399 coordinate-system roles and
//! rigid-frame algebra; no vendor source or file consulted.
//!
//! The law: an assembly is resolved whole or refused by name - a fit
//! the component checks refuse, a missing frame, an ambiguous port, a
//! cycle, a non-finite transform, or a body that reaches into the
//! flutes. Lathe assemblies reuse this chain ([`placed`]) and are
//! resolved in [`crate::lathe_assembly`].

use crate::component::{ComponentKind, CuttingItem, ToolComponent};
use crate::component_fit::{check_fit, FitError};
use crate::component_geometry::ComponentGeometry;
use crate::connection_code::ConnectionCode;
use crate::frame::{Frame, FrameError};
use crate::holder_profile::HolderProfile;
use crate::spun::{SpunError, SpunProfile};
use crate::tools::Tool;

#[path = "tool_assembly_body.rs"]
mod body;
#[path = "tool_assembly_error.rs"]
mod error;
pub use error::AssemblyError;

/// The most components one assembly stacks.
pub const MAX_NODES: usize = 16;
/// How far (mm, and unit length on an axis) a frame may stand off the
/// spindle axis and still be read as on it.
pub const AXIS_TOL: f64 = 1e-6;

/// How a child sits on its parent.
#[derive(Clone, Debug, PartialEq)]
pub struct Mount {
    /// The parent's index in the node list.
    pub parent: usize,
    /// The parent's port; `None` when the parent has exactly one.
    pub port: Option<String>,
    /// The turn (degrees) about the mating Z axis.
    pub z_rot_deg: f64,
    /// The stickout (mm) from the parent's port to the cutting tip of a
    /// solid tool on a shank mount (a plain or Weldon bore). `None`: the
    /// shank sits fully home on the bore stop that the parent's geometry
    /// states ([`ComponentGeometry::with_stop`]). A shank mount with no
    /// stated stickout and no stated stop is refused: the nose face is
    /// never a home position, because there the whole tool hangs out of
    /// the bore and the holder is modelled too far from the tip.
    pub projection_mm: Option<f64>,
}

/// One component in the stack, its geometry, and its mount (`None` for
/// the root, which sits on the spindle).
#[derive(Clone, Debug, PartialEq)]
pub struct AssemblyNode {
    pub component: ToolComponent,
    pub geometry: ComponentGeometry,
    pub mount: Option<Mount>,
}

/// What cuts, as the toolpath laws read it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CuttingParams {
    pub diameter: f64,
    pub corner_r: f64,
}

/// The holder body the collision laws judge: the envelope bands, and a
/// drawn outline of it for display.
#[derive(Clone, Debug, PartialEq)]
pub struct AssemblyBody {
    pub profile: HolderProfile,
    pub spun: Option<SpunProfile>,
}

/// A resolved assembly, in the tool frame (tip at the origin, +Z
/// toward the spindle).
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedAssembly {
    /// From the cutting tip to the root's MCS (the spindle gauge line).
    pub gauge_len: f64,
    pub cutting: CuttingParams,
    /// The solid tool's own record, when the cutting item is one.
    pub tool: Option<Tool>,
    /// The non-cutting components' bodies; `None` when none has one.
    pub body: Option<AssemblyBody>,
    /// Each node's MCS frame, in node order.
    pub frames: Vec<Frame>,
}

/// Resolve `nodes` (the module's law).
pub fn resolve(nodes: &[AssemblyNode]) -> Result<ResolvedAssembly, AssemblyError> {
    let (frames, cut, cip) = placed(nodes)?;
    let gauge_len = -cip.origin()[2];
    if !gauge_len.is_finite() || gauge_len <= 0.0 || gauge_len > crate::holder_profile::MAX_BODY_HEIGHT {
        return Err(AssemblyError::BadGauge { gauge_len });
    }
    let to_tool = |i: usize, f: &Frame| {
        let o = f.origin();
        Frame::new([o[0], o[1], o[2] + gauge_len], f.z(), f.x()).map_err(|e| AssemblyError::frame(&nodes[i], e))
    };
    let frames = frames.iter().enumerate().map(|(i, f)| to_tool(i, f)).collect::<Result<Vec<_>, _>>()?;
    let cip = to_tool(cut, &cip)?;
    let solid = match nodes[cut].component.kind() {
        ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) => Some(&**tool),
        _ => None,
    };
    let body = body::judged(nodes, &frames, cut, solid)?;
    let (cutting, tool) = body::cutting(&nodes[cut], &frames[cut], &cip)?;
    Ok(ResolvedAssembly { gauge_len, cutting, tool, body, frames })
}

/// The checked stack placed in the root's frame (the root's MCS is the
/// identity): each node's MCS frame, the cutting item's index, and its
/// cutting point. The same checks as [`resolve`] up to the gauge length:
/// the lathe resolver (#3643) reads its own tip from here.
pub(crate) fn placed(nodes: &[AssemblyNode]) -> Result<(Vec<Frame>, usize, Frame), AssemblyError> {
    if nodes.is_empty() {
        return Err(AssemblyError::Empty);
    }
    if nodes.len() > MAX_NODES {
        return Err(AssemblyError::TooMany { count: nodes.len() });
    }
    let root = find_root(nodes)?;
    let ports = check_links(nodes)?;
    let cut = find_cutter(nodes)?;
    let mut frames: Vec<Option<Frame>> = vec![None; nodes.len()];
    frames[root] = Some(Frame::IDENTITY);
    for i in 0..nodes.len() {
        place(nodes, &ports, i, &mut frames)?;
    }
    let frames: Vec<Frame> = frames.into_iter().map(|f| f.unwrap_or(Frame::IDENTITY)).collect();
    let cip = frames[cut].then(&cip_local(&nodes[cut])?);
    Ok((frames, cut, cip))
}

fn find_root(nodes: &[AssemblyNode]) -> Result<usize, AssemblyError> {
    let mut roots = nodes.iter().enumerate().filter(|(_, n)| n.mount.is_none()).map(|(i, _)| i);
    let root = roots.next().ok_or(AssemblyError::NoRoot)?;
    if let Some(j) = roots.next() {
        return Err(AssemblyError::TwoRoots { a: name(nodes, root), b: name(nodes, j) });
    }
    if nodes[root].component.insert().is_some() {
        return Err(AssemblyError::InsertAtRoot { name: name(nodes, root) });
    }
    Ok(root)
}

fn name(nodes: &[AssemblyNode], i: usize) -> String {
    nodes.get(i).map_or_else(String::new, |n| n.component.name().to_owned())
}

/// Each child's parent index, its cycle-free path to the root, its
/// port (named or the parent's only one), one child per port, and the
/// fit the component checks state. Returns each node's port.
fn check_links(nodes: &[AssemblyNode]) -> Result<Vec<Option<String>>, AssemblyError> {
    for (i, n) in nodes.iter().enumerate() {
        if let Some(m) = &n.mount
            && (m.parent >= nodes.len() || m.parent == i)
        {
            return Err(AssemblyError::BadParent { name: name(nodes, i), parent: m.parent });
        }
    }
    let mut ports: Vec<Option<String>> = vec![None; nodes.len()];
    for (i, n) in nodes.iter().enumerate() {
        let Some(m) = &n.mount else { continue };
        // Every parent index is in range: walk up at most n steps.
        let mut at = m.parent;
        let mut steps = 0;
        while let Some(up) = nodes[at].mount.as_ref() {
            steps += 1;
            if at == i || steps > nodes.len() {
                return Err(AssemblyError::Cycle { name: name(nodes, i) });
            }
            at = up.parent;
        }
        let parent = &nodes[m.parent].component;
        let port = match &m.port {
            Some(p) => p.clone(),
            None => match parent.ports().as_slice() {
                [only] => (*only).to_owned(),
                [] => return Err(AssemblyError::NoPort { parent: parent.name().to_owned() }),
                many => {
                    let ports = many.join(", ");
                    return Err(AssemblyError::AmbiguousPort { parent: parent.name().to_owned(), ports });
                }
            },
        };
        check_fit(parent, &port, &n.component).map_err(AssemblyError::Fit)?;
        let taken = nodes[..i].iter().zip(&ports).any(|(o, p)| {
            o.mount.as_ref().is_some_and(|om| om.parent == m.parent) && p.as_deref() == Some(port.as_str())
        });
        if taken {
            return Err(AssemblyError::PortTaken { parent: parent.name().to_owned(), port });
        }
        ports[i] = Some(port);
    }
    Ok(ports)
}

/// The one cutting item, which must carry nothing.
fn find_cutter(nodes: &[AssemblyNode]) -> Result<usize, AssemblyError> {
    let is_cut = |n: &AssemblyNode| matches!(n.component.kind(), ComponentKind::CuttingItem(_));
    let mut cuts = nodes.iter().enumerate().filter(|(_, n)| is_cut(n)).map(|(i, _)| i);
    let cut = cuts.next().ok_or(AssemblyError::NoCutter)?;
    if let Some(j) = cuts.next() {
        return Err(AssemblyError::TwoCutters { a: name(nodes, cut), b: name(nodes, j) });
    }
    Ok(cut)
}

/// The cutting point in the cutting item's own frame.
fn cip_local(n: &AssemblyNode) -> Result<Frame, AssemblyError> {
    if let Some(f) = n.geometry.cip() {
        return Ok(*f);
    }
    match n.component.kind() {
        ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) => {
            Frame::at_z(-(tool.flute_len + tool.shank_len)).map_err(|e| AssemblyError::frame(n, e))
        }
        _ => Ok(Frame::IDENTITY),
    }
}

/// Place node `i` (and its parents first): the parent's port frame,
/// turned, and moved to the stickout.
fn place(
    nodes: &[AssemblyNode],
    ports: &[Option<String>],
    i: usize,
    frames: &mut [Option<Frame>],
) -> Result<Frame, AssemblyError> {
    let mut chain = vec![i];
    while let Some(&top) = chain.last() {
        if frames[top].is_some() || chain.len() > nodes.len() {
            break;
        }
        match &nodes[top].mount {
            Some(m) => chain.push(m.parent),
            None => break,
        }
    }
    for &k in chain.iter().rev() {
        if frames[k].is_some() {
            continue;
        }
        let (Some(m), Some(port)) = (&nodes[k].mount, &ports[k]) else { continue };
        let parent = frames[m.parent].ok_or_else(|| AssemblyError::Cycle { name: name(nodes, k) })?;
        let pnode = &nodes[m.parent];
        let local = pnode.geometry.port(port).ok_or_else(|| AssemblyError::MissingFrame {
            name: pnode.component.name().to_owned(),
            port: port.clone(),
        })?;
        let n = &nodes[k];
        let turned = parent.then(local).turned(m.z_rot_deg).map_err(|_| AssemblyError::NonFinite {
            name: n.component.name().to_owned(),
            what: "z_rot_deg",
        })?;
        let shift = match m.projection_mm {
            None => home_shift(nodes, k, m.parent, port)?,
            Some(p) => Some(stickout_shift(nodes, k, m.parent, port, p)?),
        };
        let placed = match shift {
            None => turned,
            Some(d) => turned.moved(d).map_err(|e| AssemblyError::frame(n, e))?,
        };
        frames[k] = Some(placed);
    }
    frames[i].ok_or_else(|| AssemblyError::Cycle { name: name(nodes, i) })
}

/// How far a solid tool's MCS moves along the port's Z so that its tip
/// stands `p` below the port. Only a solid tool on a plain or Weldon
/// bore (a collet or a side-lock holder) has a stickout, and it must
/// leave some shank in the bore.
fn stickout_shift(nodes: &[AssemblyNode], k: usize, parent: usize, port: &str, p: f64) -> Result<f64, AssemblyError> {
    let n = &nodes[k];
    let no = |why: &'static str| AssemblyError::Stickout { name: n.component.name().to_owned(), why };
    if !p.is_finite() {
        return Err(AssemblyError::NonFinite { name: n.component.name().to_owned(), what: "projection_mm" });
    }
    let ComponentKind::CuttingItem(CuttingItem::Solid { .. }) = n.component.kind() else {
        return Err(no("only a solid tool has a stickout"));
    };
    if !shank_bore(nodes, parent, port) {
        return Err(no("a stickout needs a plain or Weldon bore (a collet or a side-lock holder)"));
    }
    let tip = -cip_local(n)?.origin()[2];
    if p <= 0.0 {
        return Err(no("the stickout must be more than 0 mm"));
    }
    if p >= tip {
        return Err(no("the stickout leaves no shank in the bore"));
    }
    if let Some(stop) = nodes[parent].geometry.stop(port)
        && tip - p > stop + AXIS_TOL
    {
        return Err(no("the stickout pushes the shank past the bore stop"));
    }
    Ok(tip - p)
}

/// Where a shank with no stated stickout sits: fully home on the
/// parent's stated bore stop (its MCS moves the stop depth up the
/// port's Z). A node that is not a solid tool on a plain or Weldon bore
/// sits on the port itself (`None`). A shank mount without a stated
/// stop is refused by name: the nose face is not a stop.
fn home_shift(nodes: &[AssemblyNode], k: usize, parent: usize, port: &str) -> Result<Option<f64>, AssemblyError> {
    let n = &nodes[k];
    let ComponentKind::CuttingItem(CuttingItem::Solid { .. }) = n.component.kind() else {
        return Ok(None);
    };
    if !shank_bore(nodes, parent, port) {
        return Ok(None);
    }
    let no = |why: &'static str| AssemblyError::Stickout { name: n.component.name().to_owned(), why };
    let Some(stop) = nodes[parent].geometry.stop(port) else {
        return Err(no("the holder states no bore stop, so the shank has no home position: state a stickout"));
    };
    let tip = -cip_local(n)?.origin()[2];
    if stop >= tip {
        return Err(no("the bore stop is as deep as the tool is long, so no tool stands out of the bore"));
    }
    Ok(Some(stop))
}

/// Is `port` of `parent` a plain or Weldon bore that takes a shank?
fn shank_bore(nodes: &[AssemblyNode], parent: usize, port: &str) -> bool {
    matches!(
        nodes[parent].component.kind(),
        ComponentKind::AdaptiveItem { csw: ConnectionCode::Cylindrical { .. } | ConnectionCode::Weldon { .. }, .. }
            if port == crate::component::ADAPTIVE_PORT
    )
}

impl AssemblyError {
    fn frame(n: &AssemblyNode, e: FrameError) -> Self {
        Self::BadFrame { name: n.component.name().to_owned(), err: e }
    }

    fn body(n: &AssemblyNode, e: SpunError) -> Self {
        Self::Body { name: n.component.name().to_owned(), err: e }
    }
}

impl From<FitError> for AssemblyError {
    fn from(e: FitError) -> Self {
        Self::Fit(e)
    }
}
