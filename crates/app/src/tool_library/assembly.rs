//! #3645: the tool components and the tool stacks of a `.camtools` file
//! (version 8). The file holds a top-level `components` array - the rows
//! that the exported tools' stacks name, in the one wire form of
//! `project::component_wire` - and each definition may hold its stack as
//! `assembly` (the `project::assembly_wire` links). Our own format over
//! our own project DTOs; no vendor library format consulted.
//!
//! The law is the project loader's: the table is read whole and strict
//! (each row built again through the checked constructors, refused by its
//! id), each stack must name rows of the table and resolve, and a row that
//! no stack names is refused, so nothing in the file is dropped without a
//! word. The tool record and body stay as the file states them.

use flecs_ecs::prelude::{Entity, World};
use serde_json::Value;

use crate::project::{self, ComponentDto, LinkDto};
use crate::schema::{DocumentComponent, ToolAssembly, ToolBody};
use crate::tool_assembly as ta;

use super::ASSEMBLY_VERSION;

/// A library tool's component stack and the rows it names, already
/// checked to resolve. Each row is there once, with its index in the
/// file's table, so an import adds rows in the file's order.
#[derive(Clone, Debug, PartialEq)]
pub struct LibraryAssembly {
    pub stack: ToolAssembly,
    pub rows: Vec<(usize, DocumentComponent)>,
}

/// Export: the stack of tool entity `e` as its definition's `assembly`
/// value, with the ids it names added to `used`. `None`: no stack.
pub(super) fn stack_value(world: &World, e: &Entity, name: &str, used: &mut Vec<String>) -> Result<Option<Value>, String> {
    let Some(a) = ta::assembly_of(world, e) else { return Ok(None) };
    for l in &a.0 {
        if !used.contains(&l.component) {
            used.push(l.component.clone());
        }
    }
    serde_json::to_value(project::assembly_to_dto(&a))
        .map(Some)
        .map_err(|err| format!("tool '{name}': assembly not written: {err}"))
}

/// Export: the document rows of `used`, in table order, as the file's
/// `components` value. A stack that names a row the table lacks is
/// refused by the row's id (the document laws never leave one).
pub(super) fn table_value(world: &World, used: &[String]) -> Result<Value, String> {
    let table = ta::components(world);
    if let Some(id) = used.iter().find(|id| !table.iter().any(|r| r.component.id() == id.as_str())) {
        return Err(format!("a tool assembly names the tool component '{id}', which the document does not hold"));
    }
    let rows: Vec<ComponentDto> = table
        .iter()
        .filter(|r| used.iter().any(|id| id == r.component.id()))
        .map(project::component_to_dto)
        .collect();
    serde_json::to_value(rows).map_err(|err| format!("tool components not written: {err}"))
}

/// Import: the file's table. `version` is `None` for a bare array (no
/// table). A file older than version 8 that states a table is refused:
/// it says it is a file that cannot hold one.
pub(super) fn read_table(root: &Value, version: Option<u64>) -> Result<Vec<DocumentComponent>, String> {
    let Some(raw) = root.get("components").filter(|v| !v.is_null()) else {
        return Ok(Vec::new());
    };
    if let Some(v) = version
        && v < ASSEMBLY_VERSION
    {
        return Err(format!(
            "tool library version {v} cannot hold tool components (version {ASSEMBLY_VERSION} does)"
        ));
    }
    let dtos: Vec<ComponentDto> =
        serde_json::from_value(raw.clone()).map_err(|e| format!("the tool components are malformed: {e}"))?;
    project::components_from_dtos(&dtos)
}

/// Import: a definition's `assembly` over `table`, and the tool's body
/// as the project loader lands it. Refused by the tool's name.
pub(super) fn read_assembly(
    def: &Value,
    name: &str,
    table: &[DocumentComponent],
    version: Option<u64>,
    body: Option<ToolBody>,
) -> Result<(Option<LibraryAssembly>, Option<ToolBody>), String> {
    let Some(raw) = def.get("assembly").filter(|v| !v.is_null()) else {
        return Ok((None, body));
    };
    if let Some(v) = version
        && v < ASSEMBLY_VERSION
    {
        return Err(format!(
            "tool '{name}' states an assembly, which a version {v} tool library cannot hold (version {ASSEMBLY_VERSION} does)"
        ));
    }
    let links: Vec<LinkDto> =
        serde_json::from_value(raw.clone()).map_err(|e| format!("tool '{name}': assembly is malformed: {e}"))?;
    let (stack, body) = project::stack_from_dto(name, &links, table, body)?;
    let rows = table
        .iter()
        .enumerate()
        .filter(|(_, r)| stack.0.iter().any(|l| l.component == r.component.id()))
        .map(|(i, r)| (i, r.clone()))
        .collect();
    Ok((Some(LibraryAssembly { stack, rows }), body))
}

/// Import: a row of the table that no tool's stack names refuses the
/// file by its id - an export never writes one.
pub(super) fn refuse_unused(table: &[DocumentComponent], used: &[&LibraryAssembly]) -> Result<(), String> {
    match table.iter().find(|r| !used.iter().any(|a| a.stack.0.iter().any(|l| l.component == r.component.id()))) {
        Some(r) => Err(format!(
            "tool component '{}' is in the tool library, but no tool's assembly names it",
            r.component.id()
        )),
        None => Ok(()),
    }
}
