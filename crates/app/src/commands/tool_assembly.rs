//! #3644: the document component table and tool assembly commands -
//! add or replace a shared component, edit it with a live update of
//! every tool built from it, delete an unused one, and set a tool's
//! component stack. Each is checked whole at construction (AGENTS.md:
//! malformed input never enters the history) and lands the table, the
//! stacks and every derived `ToolDef` and `ToolBody` in one undo step.
//! Operations go Stale by their stamps (the tool record and
//! `tool_body::digest`), with no cascade write here. Our own command
//! model, the snapshot-and-restore inverse every sibling uses.

use flecs_ecs::prelude::World;

use camcraft_tools::component::{ComponentKind, ToolComponent};
use camcraft_tools::component_geometry::ComponentGeometry;

use crate::commands::{ChangeTool, Command, CommandError};
use crate::model;
use crate::schema::{AssemblyLink, DocumentComponent, ToolAssembly, ToolBody};
use crate::tool_assembly::{self as ta, components, table_set};
use crate::tool_body;
use crate::tools::ToolKind;

/// #3646: cutter facts only on a tool item, running data only on an
/// insert, and running rows a milling tool can take - checked when the
/// row is made, not when a tool first reads it.
fn row_law(row: &DocumentComponent) -> Result<(), CommandError> {
    let id = row.component.id();
    if row.milling.is_some() && !matches!(row.component.kind(), ComponentKind::ToolItem { .. }) {
        return Err(CommandError::Invalid(format!("the tool component '{id}' states cutter facts, but it is not a cutter body")));
    }
    if let Some(run) = &row.running {
        if row.component.insert().is_none() {
            return Err(CommandError::Invalid(format!("the tool component '{id}' states cutting data, but it is not an insert")));
        }
        if let Some(why) = camcraft_tools::cutting_rows::refusal_for_kind(&run.cutting, ToolKind::EndMill) {
            return Err(CommandError::Invalid(format!("the tool component '{id}': {why}")));
        }
    }
    Ok(())
}

/// The message of a refusal, without the "invalid parameters:" prefix.
fn said(e: CommandError) -> String {
    match e {
        CommandError::Invalid(m) => m,
        other => other.to_string(),
    }
}

/// One tool's derived records, before and after.
struct ToolUpdate {
    name: String,
    change: ChangeTool,
    prev_body: Option<ToolBody>,
    next_body: Option<ToolBody>,
}

impl ToolUpdate {
    /// Derive tool `name` from `links` over `table` and judge it with
    /// every `ChangeTool` law against the body it lands with.
    fn new(world: &World, name: &str, links: &[AssemblyLink], table: &[DocumentComponent]) -> Result<Self, CommandError> {
        let Some(entity) = model::find_by_name(world, name) else {
            return Err(CommandError::NotFound(name.to_owned()));
        };
        let current = model::tool_of(world, &entity).ok_or_else(|| CommandError::Invalid(format!("'{name}' is not a tool")))?;
        let derived = ta::derive(name, &current, links, table).map_err(CommandError::Invalid)?;
        let change = ChangeTool::assembled(world, name, derived.tool, derived.body.as_ref())?;
        Ok(Self { name: name.to_owned(), change, prev_body: tool_body::body_of(world, &entity), next_body: derived.body })
    }

    fn apply(&self, world: &World) {
        self.change.apply(world);
        if let Some(entity) = model::find_by_name(world, &self.name) {
            tool_body::set_body(world, &entity, self.next_body.as_ref());
        }
    }

    fn invert(&self, world: &World) {
        if let Some(entity) = model::find_by_name(world, &self.name) {
            tool_body::set_body(world, &entity, self.prev_body.as_ref());
        }
        self.change.invert(world);
    }
}

/// Add a new component to the table, or (its id held already) replace
/// it with [`EditDocumentComponent`]'s live update.
pub struct UpsertDocumentComponent(Upsert);

enum Upsert {
    Add { prev: Vec<DocumentComponent>, next: Vec<DocumentComponent> },
    Edit(EditDocumentComponent),
}

impl UpsertDocumentComponent {
    /// Refused by name: every [`EditDocumentComponent`] refusal when the
    /// id is held. The component and its geometry were checked when made.
    pub fn new(world: &World, component: ToolComponent, geometry: ComponentGeometry) -> Result<Self, CommandError> {
        if ta::component(world, component.id()).is_some() {
            return EditDocumentComponent::new(world, component, geometry).map(|e| Self(Upsert::Edit(e)));
        }
        Self::with_row(world, DocumentComponent::new(component, geometry))
    }

    /// #3646: add or replace the whole row, its cutter facts and its
    /// running data included. Refused by name: a fact on the wrong kind
    /// of component, and every [`EditDocumentComponent`] refusal.
    pub fn with_row(world: &World, row: DocumentComponent) -> Result<Self, CommandError> {
        if ta::component(world, row.component.id()).is_some() {
            return EditDocumentComponent::with_row(world, row).map(|e| Self(Upsert::Edit(e)));
        }
        row_law(&row)?;
        let prev = components(world);
        let mut next = prev.clone();
        next.push(row);
        Ok(Self(Upsert::Add { prev, next }))
    }

    /// The tools whose records this command changes (sorted).
    #[must_use]
    pub fn updated_tools(&self) -> Vec<String> {
        match &self.0 {
            Upsert::Add { .. } => Vec::new(),
            Upsert::Edit(e) => e.updated_tools(),
        }
    }
}

impl Command for UpsertDocumentComponent {
    fn apply(&self, world: &World) {
        match &self.0 {
            Upsert::Add { next, .. } => table_set(world, next.clone()),
            Upsert::Edit(e) => e.apply(world),
        }
    }

    fn invert(&self, world: &World) {
        match &self.0 {
            Upsert::Add { prev, .. } => table_set(world, prev.clone()),
            Upsert::Edit(e) => e.invert(world),
        }
    }
}

/// Replace a held component and re-derive every tool built from it, in
/// one undo step. An edit that any of those tools cannot take is refused
/// whole, by the component's and the tool's names, and nothing changes.
pub struct EditDocumentComponent {
    prev: Vec<DocumentComponent>,
    next: Vec<DocumentComponent>,
    updates: Vec<ToolUpdate>,
}

impl EditDocumentComponent {
    /// Replace the catalog fact and the geometry of a held row; its
    /// cutter facts, running data and lathe body stay.
    pub fn new(world: &World, component: ToolComponent, geometry: ComponentGeometry) -> Result<Self, CommandError> {
        let held = ta::component(world, component.id());
        let row = DocumentComponent {
            milling: held.as_ref().and_then(|r| r.milling),
            lathe_body: held.as_ref().and_then(|r| r.lathe_body.clone()),
            running: held.and_then(|r| r.running),
            ..DocumentComponent::new(component, geometry)
        };
        Self::with_row(world, row)
    }

    /// #3646: replace a held row whole - the shared insert's running
    /// data or a cutter body's facts too. Every tool built from it is
    /// derived again in the same undo step.
    pub fn with_row(world: &World, row: DocumentComponent) -> Result<Self, CommandError> {
        row_law(&row)?;
        let id = row.component.id().to_owned();
        let prev = components(world);
        let Some(at) = prev.iter().position(|r| r.component.id() == id) else {
            return Err(CommandError::Invalid(format!("the document holds no tool component '{id}'")));
        };
        if prev[at] == row {
            return Err(CommandError::Invalid(format!("the tool component '{id}' is already so")));
        }
        let mut next = prev.clone();
        next[at] = row;
        let mut updates = Vec::new();
        for user in ta::users_of(world, &id) {
            let links = model::find_by_name(world, &user).and_then(|e| ta::assembly_of(world, &e)).map(|a| a.0).unwrap_or_default();
            let update = ToolUpdate::new(world, &user, &links, &next).map_err(|e| {
                CommandError::Invalid(format!("the edit of tool component '{id}' is refused: {}", said(e)))
            })?;
            updates.push(update);
        }
        Ok(Self { prev, next, updates })
    }

    /// The tools whose records this edit changes (sorted).
    #[must_use]
    pub fn updated_tools(&self) -> Vec<String> {
        self.updates.iter().map(|u| u.name.clone()).collect()
    }
}

impl Command for EditDocumentComponent {
    fn apply(&self, world: &World) {
        table_set(world, self.next.clone());
        for u in &self.updates {
            u.apply(world);
        }
    }

    fn invert(&self, world: &World) {
        for u in self.updates.iter().rev() {
            u.invert(world);
        }
        table_set(world, self.prev.clone());
    }
}

/// Delete an unused component from the table.
pub struct DeleteDocumentComponent {
    prev: Vec<DocumentComponent>,
    next: Vec<DocumentComponent>,
}

impl DeleteDocumentComponent {
    /// Refused by name: an id the document does not hold, and a
    /// component that a tool's stack still names (the tools are named).
    pub fn new(world: &World, id: &str) -> Result<Self, CommandError> {
        let id = id.trim();
        let prev = components(world);
        if !prev.iter().any(|r| r.component.id() == id) {
            return Err(CommandError::Invalid(format!("the document holds no tool component '{id}'")));
        }
        let users = ta::users_of(world, id);
        if !users.is_empty() {
            return Err(CommandError::Invalid(format!(
                "the tool component '{id}' is used by the tool assemblies of {} - take it out of them first",
                users.join(", ")
            )));
        }
        let next = prev.iter().filter(|r| r.component.id() != id).cloned().collect();
        Ok(Self { prev, next })
    }
}

impl Command for DeleteDocumentComponent {
    fn apply(&self, world: &World) {
        table_set(world, self.next.clone());
    }

    fn invert(&self, world: &World) {
        table_set(world, self.prev.clone());
    }
}

/// Set a tool's component stack and land its derived `ToolDef` and
/// `ToolBody` in the same undo step, or clear the stack (the records
/// stay as they are).
pub struct SetToolAssembly {
    name: String,
    prev: Option<ToolAssembly>,
    next: Option<ToolAssembly>,
    update: Option<ToolUpdate>,
}

impl SetToolAssembly {
    /// Refused by name: no such tool, and every `derive` and `ChangeTool`
    /// refusal (a fit, a frame, a body in the flutes, an operation the
    /// new numbers strand).
    pub fn new(world: &World, name: &str, links: Vec<AssemblyLink>) -> Result<Self, CommandError> {
        let links: Vec<AssemblyLink> =
            links.into_iter().map(|l| AssemblyLink { component: l.component.trim().to_owned(), ..l }).collect();
        let update = ToolUpdate::new(world, name, &links, &components(world))?;
        let prev = model::find_by_name(world, name).and_then(|e| ta::assembly_of(world, &e));
        Ok(Self { name: name.to_owned(), prev, next: Some(ToolAssembly(links)), update: Some(update) })
    }

    /// Clear the stack of tool `name`; its records stay as they are.
    pub fn clear(world: &World, name: &str) -> Result<Self, CommandError> {
        let Some(entity) = model::find_by_name(world, name) else {
            return Err(CommandError::NotFound(name.to_owned()));
        };
        let Some(prev) = ta::assembly_of(world, &entity) else {
            return Err(CommandError::Invalid(format!("tool '{name}' has no tool assembly to clear")));
        };
        Ok(Self { name: name.to_owned(), prev: Some(prev), next: None, update: None })
    }
}

impl Command for SetToolAssembly {
    fn apply(&self, world: &World) {
        if let Some(u) = &self.update {
            u.apply(world);
        }
        if let Some(entity) = model::find_by_name(world, &self.name) {
            ta::set_assembly(world, &entity, self.next.as_ref());
        }
    }

    fn invert(&self, world: &World) {
        if let Some(entity) = model::find_by_name(world, &self.name) {
            ta::set_assembly(world, &entity, self.prev.as_ref());
        }
        if let Some(u) = &self.update {
            u.invert(world);
        }
    }
}
