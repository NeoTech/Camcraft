//! #3645: a tool's component stack on file (`assembly` on a `.camproj`
//! tool, v84) and the load law of the component table and the stacks.
//! Our own JSON over the #3644 records (`ToolAssembly`, `AssemblyLink`,
//! the #3643 `Mount`); no vendor format consulted.
//!
//! The load law: the table is read whole first (`component_wire`), and
//! each stack must name rows of that table and resolve with the #3643
//! resolver, or the file is refused by the tool's name. The derived
//! `ToolDef` and `ToolBody` stay as the file states them: they are what
//! the document judged and posted when it was saved, so an open never
//! shifts a posted program. The next component edit derives them again.

use serde::{Deserialize, Serialize};

use camcraft_tools::tool_assembly::{resolve, AssemblyBody, AssemblyNode, Mount};

use crate::schema::{AssemblyLink, DocumentComponent, ToolAssembly, ToolBody};
use crate::tool_body::BodyFrame;

/// A stack read from file and the body its resolver judges.
pub(super) type ReadStack = (ToolAssembly, Option<AssemblyBody>);

/// One node of a stack on file.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LinkDto {
    pub component: String,
    /// Absent: the root, on the spindle.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mount: Option<MountDto>,
}

/// How a node sits on its parent (`Mount`).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MountDto {
    pub parent: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub z_rot_deg: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_mm: Option<f64>,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// The wire form of a stack.
#[must_use]
pub fn assembly_to_dto(a: &ToolAssembly) -> Vec<LinkDto> {
    a.0.iter()
        .map(|l| LinkDto {
            component: l.component.clone(),
            mount: l.mount.as_ref().map(|m| MountDto {
                parent: m.parent,
                port: m.port.clone(),
                z_rot_deg: m.z_rot_deg,
                projection_mm: m.projection_mm,
            }),
        })
        .collect()
}

/// The stack of tool `tool` from `links`, checked against `table`:
/// each id names a row, and the stack resolves. Refused by the tool's
/// name and the resolver's reason.
pub fn assembly_from_dto(tool: &str, links: &[LinkDto], table: &[DocumentComponent]) -> Result<ToolAssembly, String> {
    read_stack(tool, links, table).map(|(a, _)| a)
}

/// #3645: a `.camtools` tool's stack over the file's `table`, and its
/// stated body as the project loader lands it ([`as_derived`]), so an
/// imported assembled tool is the same as the one a project opens.
pub fn stack_from_dto(
    tool: &str,
    links: &[LinkDto],
    table: &[DocumentComponent],
    body: Option<ToolBody>,
) -> Result<(ToolAssembly, Option<ToolBody>), String> {
    let (a, resolved) = read_stack(tool, links, table)?;
    Ok((a, body.map(|b| as_derived(b, resolved.as_ref()))))
}

fn read_stack(tool: &str, links: &[LinkDto], table: &[DocumentComponent]) -> Result<ReadStack, String> {
    let mut out = Vec::with_capacity(links.len());
    let mut nodes = Vec::with_capacity(links.len());
    for l in links {
        let id = l.component.trim();
        let Some(row) = table.iter().find(|r| r.component.id() == id) else {
            return Err(format!(
                "tool '{tool}': its assembly names the tool component '{id}', which the file does not hold"
            ));
        };
        let mount = l.mount.as_ref().map(|m| Mount {
            parent: m.parent,
            port: m.port.clone(),
            z_rot_deg: m.z_rot_deg,
            projection_mm: m.projection_mm,
        });
        nodes.push(AssemblyNode { component: row.component.clone(), geometry: row.geometry.clone(), mount: mount.clone() });
        out.push(AssemblyLink { component: id.to_owned(), mount });
    }
    let r = resolve(&nodes).map_err(|e| format!("tool '{tool}': its assembly is refused: {e}"))?;
    Ok((ToolAssembly(out), r.body))
}

/// The file's body of an assembled tool as `tool_assembly::derive` lands
/// it: when the file's drawn outline is the one the stack resolves to,
/// the body takes the assembly's source and the judged bands of the
/// resolver (a united body is judged by its bands, not by its drawn
/// outline). Any other body (a hand edit, a newer resolver) stays as the
/// file states it.
pub(super) fn as_derived(mut body: ToolBody, resolved: Option<&AssemblyBody>) -> ToolBody {
    let Some(r) = resolved else { return body };
    if r.spun.is_none() || body.spun != r.spun || body.library.is_some() || body.machine_default {
        return body;
    }
    body.source = crate::tool_assembly::ASSEMBLY_SOURCE.to_owned();
    if let BodyFrame::Spindle { profile, .. } = &mut body.frame {
        *profile = r.profile.clone();
    }
    body
}

/// The table and each tool's stack of `file` (by tool name), read whole
/// before any world write: the first refusal refuses the file.
pub(super) fn read_all(
    file: &super::ProjectFile,
) -> Result<(Vec<DocumentComponent>, std::collections::HashMap<String, ReadStack>), String> {
    let table = super::component_wire::components_from_dtos(&file.components)?;
    let mut stacks = std::collections::HashMap::new();
    for dto in &file.entities {
        if let Some(links) = dto.tool.as_ref().and_then(|t| t.assembly.as_ref()) {
            stacks.insert(dto.name.clone(), read_stack(&dto.name, links, &table)?);
        }
    }
    Ok((table, stacks))
}
