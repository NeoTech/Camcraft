//! #300 / #899 / #2566: import a tool library or controller tool table as
//! ONE undo unit. The file was already checked whole by `tool_library` /
//! `tool_table`; here each tool meets the document under an explicit
//! [`ImportMode`] keyed by tool name.
//!
//! Provenance: our own command model (`AddTool` / `ChangeTool` /
//! `DeleteTool`, batched into one undo unit); no vendor material consulted.
//! #3904: a library tool's drawn cutter lands with it and undoes with it.
//! #3645: a library tool's component stack lands with it, and the rows
//! the stacks name land in the component table, in the same undo step
//! (`components`: the id law).

use flecs_ecs::prelude::World;

use crate::commands::tool::reserved;
use crate::commands::{Command, CommandError, DeleteTool};
use crate::model;
use crate::project::ToolBodyDto;
use crate::schema::{ToolAssembly, ToolBody, ToolDef};
use crate::spun::SpunProfile;
use crate::tool_body;
use crate::tool_cutter;
use crate::tool_library::LibraryTool;
use crate::tools::Tool;

pub use report::{ImportMode, ImportReport};
mod components;
mod numbers;
use numbers::{claim_number, validate_library_numbers};
mod report;

/// One planned change of an import, applied in order, inverted in reverse.
enum Step {
    /// A new tool, its body (#889), its assigned T/H number (#2442,
    /// #2566), its drawn cutter (#3904) and its stack (#3645).
    Add(String, Tool, Option<ToolBody>, u32, Option<SpunProfile>, Option<ToolAssembly>),
    /// Same entity, new definition and optional number change.
    Change {
        name: String,
        prev: Tool,
        next: Tool,
        prev_number: Option<u32>,
        next_number: Option<u32>,
        body: Option<Box<(Option<ToolBody>, ToolBody)>>,
        /// #3904: (the cutter before, the library's cutter).
        cutter: Option<Box<(Option<SpunProfile>, SpunProfile)>>,
        /// #3645: (the stack before, the library's stack) when they differ.
        assembly: Option<Box<(Option<ToolAssembly>, Option<ToolAssembly>)>>,
    },
    Remove(DeleteTool),
}

fn body_key(b: Option<&ToolBody>) -> Option<ToolBodyDto> {
    b.map(ToolBodyDto::from_body)
}

/// Import a library in one step; undo restores every changed tool exactly.
pub struct ImportTools {
    steps: Vec<Step>,
    /// #3645: the component table before and after, when rows land.
    table: Option<components::TableChange>,
    report: ImportReport,
}

fn change_refusal(
    world: &World,
    name: &str,
    prev: &Tool,
    next: &Tool,
    landing: Option<&ToolBody>,
    landing_cutter: Option<&SpunProfile>,
) -> Option<String> {
    if let Some(why) = next.torch_refusal() {
        return Some(format!("tool '{name}': {why}"));
    }
    if let Some(msg) = crate::operation::tool_gate::retype_refusal(world, name, prev.kind, next.kind) {
        return Some(msg);
    }
    let entity = model::find_by_name(world, name)?;
    if let Some(body) = landing.cloned().or_else(|| tool_body::body_of(world, &entity))
        && let Err(e) = tool_body::fits(next, &body)
    {
        return Some(format!("tool '{name}': {e}"));
    }
    if let Some(c) = landing_cutter.cloned().or_else(|| tool_cutter::cutter_of(world, &entity))
        && let Err(e) = camcraft_tools::cutter_profile::agrees(next, &c)
    {
        return Some(format!("tool '{name}': the drawn cutter profile no longer fits: {e}"));
    }
    None
}

fn collect_replace_removals(
    world: &World,
    names: &[String],
) -> Result<(Vec<String>, Vec<DeleteTool>), CommandError> {
    let mut removed = Vec::new();
    let mut dels = Vec::new();
    for (name, entity) in model::named_entities(world) {
        if model::tool_of(world, &entity).is_none() || names.contains(&name) {
            continue;
        }
        let users: Vec<String> = model::named_entities(world)
            .into_iter()
            .filter(|(_, e)| model::operation_of(world, e).is_some_and(|op| op.tool == name))
            .map(|(n, _)| n)
            .collect();
        if let Some(first) = users.first() {
            let more = users.len().saturating_sub(1);
            let more = if more == 0 { String::new() } else { format!(" (and {more} more)") };
            return Err(CommandError::Invalid(format!(
                "replace would remove tool '{name}', but operation {first}{more} uses it - keep it in the library or give the operation another tool; nothing imported"
            )));
        }
        dels.push(DeleteTool::new(world, &name)?);
        removed.push(name);
    }
    Ok((removed, dels))
}

impl ImportTools {
    /// The original import: [`ImportMode::Add`], holders taken.
    pub fn new(world: &World, library: Vec<LibraryTool>) -> Result<Self, CommandError> {
        Self::with_options(world, library, ImportMode::Add, true)
    }

    /// `mode`, with the file's holders taken (#899's door).
    pub fn with_mode(world: &World, library: Vec<LibraryTool>, mode: ImportMode) -> Result<Self, CommandError> {
        Self::with_options(world, library, mode, true)
    }

    /// [`ImportMode::Add`] with the holders choice (#889's door).
    pub fn with_holders(world: &World, library: Vec<LibraryTool>, holders: bool) -> Result<Self, CommandError> {
        Self::with_options(world, library, ImportMode::Add, holders)
    }

    /// Plan the import against the document.
    pub fn with_options(
        world: &World,
        library: Vec<LibraryTool>,
        mode: ImportMode,
        holders: bool,
    ) -> Result<Self, CommandError> {
        validate_library_numbers(&library)?;
        let names: Vec<String> = library.iter().map(|l| l.name.clone()).collect();
        let (removed_names, remove_steps) = if mode == ImportMode::Replace {
            collect_replace_removals(world, &names)?
        } else {
            (Vec::new(), Vec::new())
        };
        // Track surviving tool-number owners; entries being renumbered in
        // Update/Replace release their old slot first so swaps succeed.
        let reassigned: Vec<&str> = if mode != ImportMode::Add {
            library
                .iter()
                .filter(|l| l.number.is_some())
                .map(|l| l.name.as_str())
                .collect()
        } else {
            Vec::new()
        };
        let mut number_owners: Vec<(String, u32)> = crate::tool_numbers::numbers(world)
            .map_err(CommandError::Invalid)?
            .into_iter()
            .filter(|(n, _)| !removed_names.contains(n) && !reassigned.contains(&n.as_str()))
            .collect();

        let mut steps: Vec<Step> = Vec::with_capacity(library.len() + remove_steps.len());
        let mut added: Vec<(String, Tool)> = Vec::new();
        let mut report = ImportReport::default();

        let mut landing: Vec<crate::tool_library::LibraryAssembly> = Vec::new();
        for LibraryTool { name, tool, body, number: explicit_num, cutter, assembly } in library {
            // #3645: holders off leaves the stack out with the body.
            let assembly = assembly.filter(|_| holders);
            if let Some(err) = reserved(world, &name) {
                return Err(err);
            }
            if let Some(why) = tool.why_not_sane() {
                return Err(CommandError::Invalid(format!("tool '{name}': {why}")));
            }
            if let Some(c) = &cutter
                && let Err(e) = camcraft_tools::cutter_profile::agrees(&tool, c)
            {
                return Err(CommandError::Invalid(format!("tool '{name}': the drawn cutter profile does not fit: {e}")));
            }
            if steps.iter().any(|s| matches!(s, Step::Add(n, ..) if *n == name)) || report_has(&report, &name) {
                return Err(CommandError::Invalid(format!("tool '{name}' appears twice in the library")));
            }
            let body = match body {
                Some(b) if holders => {
                    tool_body::fits(&tool, &b).map_err(|e| CommandError::Invalid(format!("tool '{name}': {e}")))?;
                    Some(b)
                }
                Some(_) => {
                    report.holders_left_out = report.holders_left_out.saturating_add(1);
                    None
                }
                None => None,
            };
            if body.is_some() {
                report.holders = report.holders.saturating_add(1);
            }
            let taken = |n: &str| model::find_by_name(world, n).is_some() || added.iter().any(|(t, _)| t == n);
            let existing = model::find_by_name(world, &name);
            let current = existing.as_ref().and_then(|e| model::tool_of(world, e));
            let prev_number = existing.as_ref().and_then(|e| model::tool_number_of(world, e));
            if let Some(n) = explicit_num.or(prev_number) {
                crate::tool_reservations::for_tool(world, &name, tool.kind, n).map_err(CommandError::Invalid)?;
            }
            let doc_body = existing.as_ref().and_then(|e| tool_body::body_of(world, e));
            let body_same = body.is_none() || body_key(doc_body.as_ref()) == body_key(body.as_ref());
            let num_same = explicit_num.is_none() || explicit_num == prev_number;
            let doc_cutter = existing.as_ref().and_then(|e| tool_cutter::cutter_of(world, e));
            let cutter_same = cutter.is_none() || doc_cutter == cutter;
            let doc_stack = existing.as_ref().and_then(|e| crate::tool_assembly::assembly_of(world, e));
            let lib_stack = assembly.as_ref().map(|a| a.stack.clone());
            let stack_same = lib_stack.is_none() || doc_stack == lib_stack;
            match (existing, current) {
                (None, _) => {
                    let assigned = claim_number(world, tool.kind, &mut number_owners, &name, explicit_num)?;
                    report.added.push(name.clone());
                    added.push((name.clone(), tool.clone()));
                    landing.extend(assembly);
                    steps.push(Step::Add(name, tool, body, assigned, cutter, lib_stack));
                }
                (Some(_), Some(prev)) if prev == tool && body_same && num_same && cutter_same && stack_same => {
                    // #3645: the same stack can name rows that differ.
                    if let Some(a) = &assembly {
                        components::check_skipped(world, a)?;
                    }
                    if body.is_some() {
                        report.holders = report.holders.saturating_sub(1);
                    }
                    report.skipped.push(name);
                }
                (Some(_), Some(prev)) if mode != ImportMode::Add => {
                    if let Some(why) = change_refusal(world, &name, &prev, &tool, body.as_ref(), cutter.as_ref()) {
                        return Err(CommandError::Invalid(why));
                    }
                    let next_number = match explicit_num {
                        Some(n) => Some(claim_number(world, tool.kind, &mut number_owners, &name, Some(n))?),
                        None => prev_number,
                    };
                    match mode {
                        ImportMode::Replace => report.replaced.push(name.clone()),
                        _ => report.updated.push(name.clone()),
                    }
                    let body = body.map(|b| Box::new((doc_body, b)));
                    let cutter = cutter.map(|c| Box::new((doc_cutter, c)));
                    // #3645: the library's tool takes its own stack (none
                    // clears a stack its record no longer comes from).
                    landing.extend(assembly);
                    let assembly = (doc_stack != lib_stack).then(|| Box::new((doc_stack, lib_stack)));
                    steps.push(Step::Change { name, prev, next: tool, prev_number, next_number, body, cutter, assembly });
                }
                (Some(_), None) if mode != ImportMode::Add => {
                    return Err(CommandError::Invalid(format!(
                        "'{name}' in the library names something in the document that is not a tool; nothing imported"
                    )));
                }
                _ => {
                    let Some(free) = (2..=9999u32).map(|i| format!("{name} ({i})")).find(|c| !taken(c)) else {
                        return Err(CommandError::NameConflict(name));
                    };
                    let assigned = claim_number(world, tool.kind, &mut number_owners, &free, explicit_num)?;
                    report.renamed.push((name, free.clone()));
                    added.push((free.clone(), tool.clone()));
                    landing.extend(assembly);
                    steps.push(Step::Add(free, tool, body, assigned, cutter, lib_stack));
                }
            }
        }
        for (name, del) in removed_names.into_iter().zip(remove_steps) {
            steps.push(Step::Remove(del));
            report.removed.push(name);
        }
        if steps.is_empty() {
            return Err(CommandError::Invalid(
                "nothing to import: every tool in the library is already in the document".into(),
            ));
        }
        let (table, rows_added, rows_reused) = components::plan(world, &landing.iter().collect::<Vec<_>>())?;
        report.components_added = rows_added;
        report.components_reused = rows_reused;
        Ok(ImportTools { steps, table, report })
    }

    #[must_use]
    pub fn report(&self) -> &ImportReport {
        &self.report
    }
}

fn report_has(report: &ImportReport, name: &str) -> bool {
    [&report.skipped, &report.updated, &report.replaced]
        .iter()
        .any(|v| v.iter().any(|n| n == name))
        || report.renamed.iter().any(|(from, _)| from == name)
}

fn put(world: &World, name: &str, tool: &Tool, number: Option<u32>) {
    if let Some(entity) = model::find_by_name(world, name) {
        let number = number.or_else(|| model::tool_number_of(world, &entity));
        entity.entity_view(world).set(ToolDef { tool: tool.clone(), number });
    }
}

fn put_body(world: &World, name: &str, body: Option<&ToolBody>) {
    if let Some(entity) = model::find_by_name(world, name) {
        tool_body::set_body(world, &entity, body);
    }
}

fn put_cutter(world: &World, name: &str, cutter: Option<&SpunProfile>) {
    if let Some(entity) = model::find_by_name(world, name) {
        tool_cutter::set_cutter(world, &entity, cutter);
    }
}

impl Command for ImportTools {
    fn apply(&self, world: &World) {
        if let Some((_, next)) = &self.table {
            crate::tool_assembly::table_set(world, next.clone());
        }
        for step in &self.steps {
            match step {
                Step::Add(name, tool, body, number, cutter, stack) => {
                    let entity = match model::find_by_name(world, name) {
                        Some(entity) => entity,
                        None => model::spawn(world, name),
                    };
                    entity
                        .entity_view(world)
                        .set(ToolDef { tool: tool.clone(), number: Some(*number) });
                    tool_body::set_body(world, &entity, body.as_ref());
                    tool_cutter::set_cutter(world, &entity, cutter.as_ref());
                    crate::tool_assembly::set_assembly(world, &entity, stack.as_ref());
                }
                Step::Change { name, next, next_number, body, cutter, assembly, .. } => {
                    put(world, name, next, *next_number);
                    if let Some((_, b)) = body.as_deref() {
                        put_body(world, name, Some(b));
                    }
                    if let Some((_, c)) = cutter.as_deref() {
                        put_cutter(world, name, Some(c));
                    }
                    if let Some((_, a)) = assembly.as_deref() {
                        components::put_assembly(world, name, a.as_ref());
                    }
                }
                Step::Remove(del) => del.apply(world),
            }
        }
    }

    fn invert(&self, world: &World) {
        for step in self.steps.iter().rev() {
            match step {
                Step::Add(name, ..) => {
                    if let Some(entity) = model::find_by_name(world, name) {
                        model::remove(world, &entity);
                    }
                }
                Step::Change { name, prev, prev_number, body, cutter, assembly, .. } => {
                    put(world, name, prev, *prev_number);
                    if let Some((before, _)) = body.as_deref() {
                        put_body(world, name, before.as_ref());
                    }
                    if let Some((before, _)) = cutter.as_deref() {
                        put_cutter(world, name, before.as_ref());
                    }
                    if let Some((before, _)) = assembly.as_deref() {
                        components::put_assembly(world, name, before.as_ref());
                    }
                }
                Step::Remove(del) => del.invert(world),
            }
        }
        if let Some((prev, _)) = &self.table {
            crate::tool_assembly::table_set(world, prev.clone());
        }
    }
}
