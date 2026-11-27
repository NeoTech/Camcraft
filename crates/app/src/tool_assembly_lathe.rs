//! #3643: the lathe side of the document's tool derivation - a turning
//! stack (a holder or boring bar with its ISO insert) from the document
//! component table, resolved by `camcraft_tools::lathe_assembly` and
//! landed as the Turning record and its lathe-frame body. Our own design
//! over the public ISO 13399 roles and the LinuxCNC orientation chart;
//! no vendor source consulted.
//!
//! The laws here:
//! - **Turning inserts only.** A grooving, threading or form tool is
//!   refused by name: the resolver states no blade or thread values yet.
//! - **The station comes from the root.** A rectangular shank sits
//!   across the spindle (an OD holder), a round or Weldon shank along it
//!   (a boring bar). Any other root coupling (a VDI sleeve) is refused by
//!   name: its station is not stated in the document yet.
//! - **The geometry states the orientation.** The resolver derives it
//!   from the shank, the seat's hand and the insert. The record keeps the
//!   kind's default as `None`, so a tool that a user types and the same
//!   tool assembled are one record.
//! - **Every resolver refusal is a refusal by name**, with the tool.

use camcraft_tools::connection_code::ConnectionCode;
use camcraft_tools::lathe_assembly::{LatheNode, LatheStation, resolve_lathe};
use camcraft_tools::orientation;
use camcraft_tools::tool_assembly::AssemblyNode;

use crate::geometry::{Point3, Triangle};
use crate::schema::DocumentComponent;
use crate::tools::{Tool, ToolKind};

use super::{ASSEMBLY_SOURCE, Derived};

/// Resolve the turning stack `nodes` (with their table `rows`, in the
/// same order) for the tool `name` (now `current`).
pub(super) fn derive(
    name: &str,
    current: &Tool,
    nodes: Vec<AssemblyNode>,
    rows: &[&DocumentComponent],
) -> Result<Derived, String> {
    if current.kind != ToolKind::Turning {
        return Err(format!(
            "tool '{name}': a {} assembly is not resolved yet - only a turning insert in its holder is (#3643)",
            current.kind
        ));
    }
    let station = station(name, &nodes)?;
    let lathe: Vec<LatheNode> =
        nodes.into_iter().zip(rows).map(|(node, row)| LatheNode { node, body: row.lathe_body.clone() }).collect();
    let refused = |e: String| format!("tool '{name}': lathe assembly: {e}");
    let r = resolve_lathe(&lathe, station, None).map_err(|e| refused(e.to_string()))?;
    let p = r.params;
    let mut tool = current.clone();
    let at = lathe.iter().position(|n| n.node.component.insert().is_some());
    if let Some(run) = at.and_then(|i| rows.get(i)).and_then(|row| row.running.as_ref()) {
        tool.tool_material = run.tool_material;
        tool.cutting = run.cutting.clone();
    }
    tool.nose_radius = p.nose_radius;
    tool.orientation = (orientation::default_for(ToolKind::Turning) != Some(p.orientation)).then_some(p.orientation);
    if let Some(ic) = p.ic_mm {
        tool.diameter = ic;
    }
    let body = if r.placed.is_empty() {
        None
    } else {
        let pt = |v: [f64; 3]| Point3::new(v[0], v[1], v[2]);
        let tris = r.placed.iter().map(|t| Triangle::new(pt(t[0]), pt(t[1]), pt(t[2]))).collect();
        // #3643: the bands read from the facets grow by the stack's pad
        // (a meshed body's chord deviation), so they are never smaller
        // than the steel - the same bands as the resolver's clearance.
        // The body keeps the pad, so a rebuild and the file keep it too.
        Some(crate::tool_body_lathe::build_padded(ASSEMBLY_SOURCE, r.placement, None, tris, r.pad).map_err(refused)?)
    };
    Ok(Derived { tool, body, gauge_len: 0.0 })
}

/// Where the root sits on the lathe, by its coupling. No root: the
/// resolver names that refusal.
fn station(name: &str, nodes: &[AssemblyNode]) -> Result<LatheStation, String> {
    let Some(root) = nodes.iter().find(|n| n.mount.is_none()) else {
        return Ok(LatheStation::RADIAL_SHANK);
    };
    match root.component.mcs() {
        Some(ConnectionCode::RectShank { .. }) => Ok(LatheStation::RADIAL_SHANK),
        Some(ConnectionCode::Cylindrical { .. } | ConnectionCode::Weldon { .. }) => Ok(LatheStation::AXIAL_SHANK),
        other => {
            let coupling = other.map_or_else(|| "no".to_owned(), ToString::to_string);
            Err(format!(
                "tool '{name}': the root component '{}' has a {coupling} coupling - a turning stack starts at a rectangular shank or a round boring-bar shank (#3643)",
                root.component.name()
            ))
        }
    }
}
