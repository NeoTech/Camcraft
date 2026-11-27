//! #3644: the document's tool component table and each tool's component
//! stack in the ECS world, and the one derivation from them: a stack
//! resolved by the #3643 resolver (`camcraft_tools::tool_assembly`) and
//! landed as the tool's `ToolDef` numbers and `ToolBody`. Our own design
//! over the public ISO 13399 component roles; no vendor source consulted.
//!
//! The laws here:
//! - **One row for each id.** The table ([`DocumentComponents`]) holds
//!   each component id once. A tool's stack ([`ToolAssembly`]) names
//!   rows by id; a name the table does not hold is refused by name.
//! - **Derived whole or refused.** [`derive`] resolves the stack and
//!   gives the tool record and the body that the laws read, or the
//!   resolver's named refusal. Nothing is written here: the commands
//!   (`crate::commands::tool_assembly`) land the result.
//! - **The spindle side only.** A lathe-held tool is refused: the lathe
//!   side of the resolver is still open on #3643.
//! - **A multi-seat cutter is derived whole** (#3646). When the insert
//!   sits on a body row with cutter facts, the whole milling record
//!   comes from `camcraft_tools::milling_cutter` (diameter, corner,
//!   flutes, flute length, centre cutting, the enclosed holder), and
//!   every seat must take the insert. An insert row's running data
//!   (tool material, cutting rows) goes on each tool that cuts with it.

use flecs_ecs::core::{EntityViewGet, WorldGet}; // try_get is not re-exported in prelude (0.2.2)
use flecs_ecs::prelude::{Entity, World};

use camcraft_tools::milling_cutter;
use camcraft_tools::tool_assembly::{resolve, AssemblyNode};

use crate::schema::{AssemblyLink, DocumentComponent, DocumentComponents, ToolAssembly, ToolBody};
use crate::tool_body::{build_spun, BodyFrame, LATHE_HELD};
use crate::tools::Tool;

/// What an assembly's body is called where a file name would be.
pub const ASSEMBLY_SOURCE: &str = "tool assembly";

/// The document's component rows, in table order.
#[must_use]
pub fn components(world: &World) -> Vec<DocumentComponent> {
    world.try_get::<&DocumentComponents>(|t| t.0.clone()).unwrap_or_default()
}

/// The row of component `id`, if the document holds it.
#[must_use]
pub fn component(world: &World, id: &str) -> Option<DocumentComponent> {
    components(world).into_iter().find(|r| r.component.id() == id.trim())
}

/// Replace the table - the one writer, for the component commands and
/// the project loader.
pub fn table_set(world: &World, rows: Vec<DocumentComponent>) {
    world.set(DocumentComponents(rows));
}

/// The component stack of a tool entity, if it has one.
#[must_use]
pub fn assembly_of(world: &World, entity: &Entity) -> Option<ToolAssembly> {
    entity.entity_view(world).try_get::<&ToolAssembly>(|a| a.clone())
}

/// Land (`Some`) or clear (`None`) a tool's stack - the one writer.
pub fn set_assembly(world: &World, entity: &Entity, assembly: Option<&ToolAssembly>) {
    let view = entity.entity_view(world);
    match assembly {
        Some(a) => {
            view.set(a.clone());
        }
        None => {
            view.remove(world.component::<ToolAssembly>());
        }
    }
}

/// The names of the document tools (sorted) whose stack names
/// component `id`.
#[must_use]
pub fn users_of(world: &World, id: &str) -> Vec<String> {
    let id = id.trim();
    crate::model::named_entities(world)
        .into_iter()
        .filter(|(_, e)| crate::model::is_tool(world, e))
        .filter(|(_, e)| assembly_of(world, e).is_some_and(|a| a.0.iter().any(|l| l.component == id)))
        .map(|(n, _)| n)
        .collect()
}

/// What a stack lands on its tool.
#[derive(Debug, Clone, PartialEq)]
pub struct Derived {
    /// The tool record: a solid tool's own, or the current record with
    /// the insert's cutting diameter and corner radius.
    pub tool: Tool,
    /// The judged holder body; `None` when no component has a body.
    pub body: Option<ToolBody>,
    /// From the cutting tip to the spindle gauge line.
    pub gauge_len: f64,
}

/// Resolve `links` over `table` for the tool `name` (now `current`).
/// Refused by name: a lathe-held tool, a component the table does not
/// hold, each resolver refusal, and a united body that cannot be drawn.
pub fn derive(name: &str, current: &Tool, links: &[AssemblyLink], table: &[DocumentComponent]) -> Result<Derived, String> {
    if LATHE_HELD.contains(&current.kind) {
        return Err(format!(
            "tool '{name}': a {} is lathe-held - a lathe assembly is not resolved yet (#3643)",
            current.kind
        ));
    }
    let mut nodes = Vec::with_capacity(links.len());
    let mut rows = Vec::with_capacity(links.len());
    for link in links {
        let Some(row) = table.iter().find(|r| r.component.id() == link.component) else {
            return Err(format!(
                "tool '{name}': the assembly names the component '{}', which the document does not hold",
                link.component
            ));
        };
        nodes.push(AssemblyNode { component: row.component.clone(), geometry: row.geometry.clone(), mount: link.mount.clone() });
        rows.push(row);
    }
    let r = resolve(&nodes).map_err(|e| format!("tool '{name}': assembly: {e}"))?;
    let tool = match r.tool {
        Some(solid) => solid,
        None => cut_with_insert(name, current, &nodes, &rows, r.cutting.diameter, r.cutting.corner_r)?,
    };
    let body = match r.body {
        None => None,
        Some(b) => {
            let Some(spun) = b.spun else {
                return Err(format!(
                    "tool '{name}': assembly: the united holder bodies cannot be drawn as one outline"
                ));
            };
            let mut body = build_spun(spun);
            body.source = ASSEMBLY_SOURCE.to_owned();
            // The bands the resolver judged are the ones the laws read.
            if let BodyFrame::Spindle { profile, .. } = &mut body.frame {
                *profile = b.profile;
            }
            Some(body)
        }
    };
    Ok(Derived { tool, body, gauge_len: r.gauge_len })
}

/// The record of a tool that cuts with an insert: the insert row's
/// running data over `current`, then either the whole multi-seat
/// record (the insert's parent row states cutter facts) or the
/// resolver's cutting diameter and corner radius.
fn cut_with_insert(
    name: &str,
    current: &Tool,
    nodes: &[AssemblyNode],
    rows: &[&DocumentComponent],
    diameter: f64,
    corner_r: f64,
) -> Result<Tool, String> {
    let at = nodes.iter().position(|n| n.component.insert().is_some());
    let mut base = current.clone();
    if let Some(run) = at.and_then(|i| rows.get(i)).and_then(|r| r.running.as_ref()) {
        base.tool_material = run.tool_material;
        base.cutting = run.cutting.clone();
    }
    let parent = at.and_then(|i| nodes[i].mount.as_ref()).and_then(|m| rows.get(m.parent).map(|r| (m.parent, *r)));
    let (Some(i), Some((p, body))) = (at, parent) else {
        return Ok(Tool { diameter, corner_r, ..base });
    };
    let Some(spec) = &body.milling else {
        return Ok(Tool { diameter, corner_r, ..base });
    };
    let refused = |e: milling_cutter::MillingError| format!("tool '{name}': {e}");
    let cutter = milling_cutter::derive(&nodes[p].component, spec, &nodes[i].component).map_err(refused)?;
    cutter.assembled(&base, nodes).map_err(refused)
}
