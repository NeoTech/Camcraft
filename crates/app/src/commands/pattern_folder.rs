//! Pattern folders (#3012) as undoable commands: create one around
//! member operations, edit its pattern, set its members, turn one copy
//! on or off, and delete it (the members stay, ungrouped). Reorder and
//! rename are `ReorderOperation` and `RenameOperation`.
//! Provenance: our own document model; linear and grid part patterns
//! are general CAM practice (see `pattern_folder`).

use flecs_ecs::prelude::World;

use crate::commands::{Command, CommandError};
use crate::model;
use crate::pattern_folder::{checked_kind, copy_count, folder_of_member, gather, member_problem, pattern_of};
use crate::program_entry::{declared_setup, effective_setup, sequence};
use crate::schema::{OperationMeta, PatternDef, PatternKind, PatternOrder};
use crate::setup;

/// `(key, before, after)` for each list row whose position a command pins.
pub(crate) type Moved = Vec<(String, OperationMeta, OperationMeta)>;

pub(crate) fn set_metas(world: &World, rows: &Moved, forward: bool) {
    for (name, before, next) in rows {
        if let Some(entity) = model::find_by_name(world, name) {
            entity.entity_view(world).set(if forward { next.clone() } else { before.clone() });
        }
    }
}

/// The rows of one setup's operation list, in list order.
pub(crate) fn rows_of(world: &World, effective: Option<&str>) -> Vec<String> {
    sequence(world)
        .into_iter()
        .filter(|(_, e)| effective_setup(world, e).as_deref() == effective)
        .map(|(n, _)| n)
        .collect()
}

/// Pin each row at its index (0..n). `new` names a row not yet in the
/// world: its position comes back as the second value.
pub(crate) fn pin(world: &World, rows: Vec<String>, new: Option<&str>) -> Result<(Moved, OperationMeta), CommandError> {
    let mut moved = Vec::with_capacity(rows.len());
    let mut meta = OperationMeta::default();
    for (index, key) in rows.into_iter().enumerate() {
        let order = u32::try_from(index).map_err(|_| CommandError::Invalid("too many operations".into()))?;
        if new == Some(key.as_str()) {
            meta.order = Some(order);
            continue;
        }
        let e = model::find_by_name(world, &key).ok_or_else(|| CommandError::NotFound(key.clone()))?;
        let before = model::operation_meta_of(world, &e);
        let mut next = before.clone();
        next.order = Some(order);
        moved.push((key, before, next));
    }
    Ok((moved, meta))
}

/// A row's `(declared, effective)` setup.
type Setups = (Option<String>, Option<String>);

/// The member list checked: operations that exist, once each, milling
/// work, all in one setup, and in no other folder. The answer is the
/// members' `(declared, effective)` setup, or `None` for no members.
fn checked_members(
    world: &World,
    folder: Option<&str>,
    members: &[String],
) -> Result<Option<Setups>, CommandError> {
    let mut setups: Option<Setups> = None;
    for (i, key) in members.iter().enumerate() {
        if members[..i].contains(key) {
            return Err(CommandError::Invalid(format!("'{key}' is named twice in the member list")));
        }
        let e = model::find_by_name(world, key).ok_or_else(|| CommandError::NotFound(key.clone()))?;
        if let Some(why) = member_problem(world, key, &e) {
            return Err(CommandError::Invalid(why));
        }
        if let Some((other, _)) = folder_of_member(world, key).filter(|(f, _)| Some(f.as_str()) != folder) {
            return Err(CommandError::Invalid(format!(
                "'{key}' is already in pattern folder '{other}' - an operation belongs to one folder only"
            )));
        }
        let here = (declared_setup(world, &e), effective_setup(world, &e));
        match &setups {
            Some((_, first)) if *first != here.1 => {
                return Err(CommandError::Invalid(format!(
                    "'{key}' is in another setup - a pattern folder holds operations of one setup"
                )));
            }
            Some(_) => {}
            None => setups = Some(here),
        }
    }
    Ok(setups)
}

fn folder(world: &World, key: &str) -> Result<PatternDef, CommandError> {
    model::find_by_name(world, key)
        .and_then(|e| pattern_of(world, &e))
        .ok_or_else(|| CommandError::NotFound(key.to_owned()))
}

/// Make a pattern folder around `members` (folder order). It takes the
/// row of the first member in list order, and the members follow it.
/// With no members it stands at the end of the active setup's list.
pub struct CreatePatternFolder {
    name: String,
    def: PatternDef,
    meta: OperationMeta,
    moved: Moved,
}

impl CreatePatternFolder {
    pub fn new(world: &World, kind: &PatternKind, order_mode: PatternOrder, members: &[String]) -> Result<Self, CommandError> {
        let kind = checked_kind(kind).map_err(CommandError::Invalid)?;
        let (declared, effective) = match checked_members(world, None, members)? {
            Some(s) => s,
            None => {
                let active = setup::active(world);
                let effective = setup::effective(world, active.as_deref());
                (active, effective)
            }
        };
        let name = (1..=u32::MAX)
            .map(|n| format!("Pattern{n}"))
            .find(|n| model::find_by_name(world, n).is_none())
            .ok_or_else(|| CommandError::Invalid("no free pattern folder name".into()))?;
        let mut rows = rows_of(world, effective.as_deref());
        let at = rows.iter().position(|r| members.contains(r)).unwrap_or(rows.len());
        rows.insert(at, name.clone());
        let rows = gather(world, &rows, Some((&name, members)));
        let (moved, meta) = pin(world, rows, Some(&name))?;
        let def = PatternDef {
            setup: declared,
            suppressed: false,
            order_mode,
            kind,
            members: members.to_vec(),
            disabled_instances: Vec::new(),
        };
        Ok(Self { name, def, meta, moved })
    }

    /// The new folder's key.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Command for CreatePatternFolder {
    fn apply(&self, world: &World) {
        set_metas(world, &self.moved, true);
        let entity = model::find_by_name(world, &self.name).unwrap_or_else(|| model::spawn(world, &self.name));
        entity.entity_view(world).set(self.def.clone());
        entity.entity_view(world).set(self.meta.clone());
    }

    fn invert(&self, world: &World) {
        if let Some(entity) = model::find_by_name(world, &self.name) {
            model::remove(world, &entity);
        }
        set_metas(world, &self.moved, false);
    }
}

/// One folder's definition before and after, with the rows it pins.
pub struct PatternChange {
    name: String,
    before: PatternDef,
    after: PatternDef,
    moved: Moved,
}

impl PatternChange {
    fn new(name: &str, before: PatternDef, after: PatternDef, moved: Moved) -> Result<Self, CommandError> {
        if after == before {
            return Err(CommandError::Invalid(format!("'{name}' already reads that way")));
        }
        let copies = copy_count(&after.kind);
        let off = after.disabled_instances.iter().filter(|n| (1..=copies).contains(*n)).count();
        if off >= usize::try_from(copies).unwrap_or(usize::MAX) {
            return Err(CommandError::Invalid(format!("'{name}': at least one copy must stay enabled")));
        }
        Ok(Self { name: name.to_owned(), before, after, moved })
    }
}

impl Command for PatternChange {
    fn apply(&self, world: &World) {
        set_metas(world, &self.moved, true);
        if let Some(e) = model::find_by_name(world, &self.name) {
            e.entity_view(world).set(self.after.clone());
        }
    }

    fn invert(&self, world: &World) {
        if let Some(e) = model::find_by_name(world, &self.name) {
            e.entity_view(world).set(self.before.clone());
        }
        set_metas(world, &self.moved, false);
    }
}

/// What an edit changes; `None` keeps the field.
#[derive(Debug, Clone, Default)]
pub struct PatternEdit {
    pub kind: Option<PatternKind>,
    pub order_mode: Option<PatternOrder>,
    pub suppressed: Option<bool>,
}

/// Change a folder's pattern, its order mode or its suppression. A
/// smaller pattern drops the off-switches of copies it no longer makes.
pub struct EditPatternFolder(PatternChange);

impl EditPatternFolder {
    pub fn new(world: &World, key: &str, edit: &PatternEdit) -> Result<Self, CommandError> {
        let before = folder(world, key)?;
        let mut after = before.clone();
        if let Some(kind) = &edit.kind {
            after.kind = checked_kind(kind).map_err(CommandError::Invalid)?;
            let copies = copy_count(&after.kind);
            after.disabled_instances.retain(|n| *n <= copies);
        }
        if let Some(mode) = edit.order_mode {
            after.order_mode = mode;
        }
        if let Some(s) = edit.suppressed {
            after.suppressed = s;
        }
        PatternChange::new(key, before, after, Vec::new()).map(Self)
    }
}

/// Replace a folder's members (folder order). The members move to
/// directly after the folder; operations taken out keep their rows.
pub struct SetPatternMembers(PatternChange);

impl SetPatternMembers {
    pub fn new(world: &World, key: &str, members: &[String]) -> Result<Self, CommandError> {
        let before = folder(world, key)?;
        let entity = model::find_by_name(world, key).ok_or_else(|| CommandError::NotFound(key.to_owned()))?;
        let effective = effective_setup(world, &entity);
        if let Some((_, theirs)) = checked_members(world, Some(key), members)?
            && theirs != effective
        {
            return Err(CommandError::Invalid(format!(
                "the members must be in the folder's setup ({})",
                effective.as_deref().unwrap_or("the first setup")
            )));
        }
        let rows = gather(world, &rows_of(world, effective.as_deref()), Some((key, members)));
        let (moved, _) = pin(world, rows, None)?;
        let after = PatternDef { members: members.to_vec(), ..before.clone() };
        PatternChange::new(key, before, after, moved).map(Self)
    }
}

/// Turn one copy (1-based) of a folder on or off.
pub struct SetPatternInstanceEnabled(PatternChange);

impl SetPatternInstanceEnabled {
    pub fn new(world: &World, key: &str, instance: u32, enabled: bool) -> Result<Self, CommandError> {
        let before = folder(world, key)?;
        let copies = copy_count(&before.kind);
        if instance == 0 || instance > copies {
            return Err(CommandError::Invalid(format!("'{key}' has copies 1 to {copies}, not {instance}")));
        }
        let mut after = before.clone();
        after.disabled_instances.retain(|n| *n != instance);
        if !enabled {
            after.disabled_instances.push(instance);
            after.disabled_instances.sort_unstable();
        }
        PatternChange::new(key, before, after, Vec::new()).map(Self)
    }
}

macro_rules! delegate {
    ($($t:ty),*) => {$(
        impl Command for $t {
            fn apply(&self, world: &World) {
                self.0.apply(world);
            }
            fn invert(&self, world: &World) {
                self.0.invert(world);
            }
        }
    )*};
}
delegate!(EditPatternFolder, SetPatternMembers, SetPatternInstanceEnabled);

/// Delete a folder; its members stay where they stand, ungrouped.
pub struct DeletePatternFolder {
    name: String,
    def: PatternDef,
    meta: OperationMeta,
}

impl DeletePatternFolder {
    pub fn new(world: &World, key: &str) -> Result<Self, CommandError> {
        let def = folder(world, key)?;
        let entity = model::find_by_name(world, key).ok_or_else(|| CommandError::NotFound(key.to_owned()))?;
        Ok(Self { name: key.to_owned(), def, meta: model::operation_meta_of(world, &entity) })
    }
}

impl Command for DeletePatternFolder {
    fn apply(&self, world: &World) {
        if let Some(entity) = model::find_by_name(world, &self.name) {
            model::remove(world, &entity);
        }
    }

    fn invert(&self, world: &World) {
        let entity = model::find_by_name(world, &self.name).unwrap_or_else(|| model::spawn(world, &self.name));
        entity.entity_view(world).set(self.def.clone());
        entity.entity_view(world).set(self.meta.clone());
    }
}

/// A deleted operation leaves its folder (`DeleteEntity`), and its undo
/// puts it back in the folder at the same place.
pub(crate) struct MemberDrop {
    folder: String,
    before: PatternDef,
    after: PatternDef,
}

impl MemberDrop {
    pub(crate) fn of(world: &World, op: &str) -> Option<Self> {
        let (folder, before) = folder_of_member(world, op)?;
        let after = PatternDef { members: before.members.iter().filter(|m| *m != op).cloned().collect(), ..before.clone() };
        Some(Self { folder, before, after })
    }

    pub(crate) fn write(&self, world: &World, forward: bool) {
        if let Some(e) = model::find_by_name(world, &self.folder) {
            e.entity_view(world).set(if forward { self.after.clone() } else { self.before.clone() });
        }
    }
}
