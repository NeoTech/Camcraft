//! #3898 (U7 of #3663): what the tool library window's component pages
//! derive from the document and the component library - each document row
//! with its library status and its users, the ids that fit the
//! "compatible" filter, and what "Add component" offers for the stack
//! draft. Frame cost (`crates/ui/AGENTS.md`): State owns the caches, with
//! their input. The component table and the stacks that use each row are
//! read once for each document version; a set is built again only when
//! that read differs (`rev`), the library read changes, or the filter or
//! the stack draft changes. A still frame reads and builds nothing
//! (`ViewBuilds`).
//! Provenance: our own caches over our own component records
//! (`component_fit`, `component_library::sync_status`); no HSMWorks
//! material involved.

use std::rc::Rc;

use camcraft_app::component::{ComponentKind, CuttingItem, ToolComponent};
use camcraft_app::component_fit::{compatible_children, compatible_parents};
use camcraft_app::component_library::{SyncStatus, sync_status};
use camcraft_app::schema::{AssemblyLink, DocumentComponent};
use camcraft_app::tool_assembly::{self as ta, Mount};

use super::State;

/// One document row with its library status and the tools that use it.
pub type ComponentRow = (DocumentComponent, SyncStatus, Vec<String>);

/// What the pages derive from the document and the library read.
pub struct Derived {
    /// The ids that fit the compatible filter's component (`None`: no filter).
    pub compatible: Option<Rc<Vec<String>>>,
    /// Each document row with its library status and its users.
    pub rows: Rc<Vec<ComponentRow>>,
}

/// What the stack group derives from the stack draft.
pub struct StackView {
    /// The parts that can go on the stack next ([`candidates`]).
    pub offer: Vec<(String, AssemblyLink)>,
}

/// The document's inputs of the sets: the component table and, for each
/// row, the tools whose stack uses it.
#[derive(Default, PartialEq)]
struct DocInputs {
    table: Vec<DocumentComponent>,
    users: Vec<Vec<String>>,
}

/// The reads and builds of the caches (the counter tests read them).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ViewBuilds {
    /// Reads of the component table and the stacks.
    pub reads: u64,
    pub rows: u64,
    pub compatible: u64,
    pub stack: u64,
}

type RowsKey = (u64, u64);
type CompatibleKey = (u64, u64, String);
type StackKey = (u64, Vec<AssemblyLink>);

/// The caches. `rev` changes only when a read of the document gives other
/// inputs; the keys hold it with the library read and the page's input.
#[derive(Default)]
pub struct ComponentViews {
    read_at: Option<u64>,
    rev: u64,
    inputs: Rc<DocInputs>,
    rows: Option<(RowsKey, Rc<Vec<ComponentRow>>)>,
    compatible: Option<(CompatibleKey, Rc<Vec<String>>)>,
    stack: Option<(StackKey, Rc<StackView>)>,
    builds: ViewBuilds,
}

impl ComponentViews {
    /// The reads and builds so far.
    #[must_use]
    pub fn builds(&self) -> ViewBuilds {
        self.builds
    }
}

/// The parts that can go on the stack next, with the link each one adds:
/// the holding parts for an empty stack, else the parts that fit the last
/// holding part (a cutting item holds nothing).
#[must_use]
pub fn candidates(links: &[AssemblyLink], table: &[DocumentComponent]) -> Vec<(String, AssemblyLink)> {
    let comps: Vec<&ToolComponent> = table.iter().map(|r| &r.component).collect();
    if links.is_empty() {
        return comps
            .into_iter()
            .filter(|c| !matches!(c.kind(), ComponentKind::CuttingItem(CuttingItem::Insert(_))))
            .map(|c| (c.name().to_owned(), AssemblyLink { component: c.id().to_owned(), mount: None }))
            .collect();
    }
    let parent = links.iter().enumerate().rev().find_map(|(i, l)| {
        comps.iter().copied().find(|c| c.id() == l.component).filter(|c| !matches!(c.kind(), ComponentKind::CuttingItem(_))).map(|c| (i, c))
    });
    let Some((at, parent)) = parent else {
        return Vec::new();
    };
    compatible_children(parent, comps.iter().copied())
        .into_iter()
        .filter(|f| f.component.insert().is_some() || !links.iter().any(|l| l.component == f.component.id()))
        .map(|f| {
            // A solid tool on a shank mount needs a stated stick-out: it
            // starts at the tool's own (flute plus shank length).
            let projection_mm = match f.component.kind() {
                ComponentKind::CuttingItem(CuttingItem::Solid { tool, .. }) => Some(tool.flute_len + tool.shank_len),
                _ => None,
            };
            let mount = Mount { parent: at, port: Some(f.port.clone()), z_rot_deg: 0.0, projection_mm };
            (format!("{} in {} {}", f.component.name(), parent.name(), f.port), AssemblyLink { component: f.component.id().to_owned(), mount: Some(mount) })
        })
        .collect()
}

impl State {
    /// Read the table and the stacks again when the document changed;
    /// `rev` moves only when they differ from the last read.
    fn read_component_inputs(&mut self) {
        if self.component_views.read_at == Some(self.doc_version) {
            return;
        }
        let table = ta::components(&self.world);
        let users = table.iter().map(|r| ta::users_of(&self.world, r.component.id())).collect();
        let inputs = DocInputs { table, users };
        let v = &mut self.component_views;
        v.builds.reads = v.builds.reads.saturating_add(1);
        v.read_at = Some(self.doc_version);
        if *v.inputs != inputs {
            v.inputs = Rc::new(inputs);
            v.rev = v.rev.saturating_add(1);
        }
    }

    /// The document's component rows with their status and users, and
    /// (`compatible`) the ids that fit that component.
    pub(crate) fn component_derived(&mut self, compatible: Option<&str>) -> Derived {
        let lib = self.component_library();
        self.read_component_inputs();
        let v = &mut self.component_views;
        let key = (v.rev, lib.generation);
        let rows = match &v.rows {
            Some((k, r)) if *k == key => Rc::clone(r),
            _ => {
                v.builds.rows = v.builds.rows.saturating_add(1);
                let r: Rc<Vec<ComponentRow>> = Rc::new(
                    v.inputs.table.iter().zip(&v.inputs.users).map(|(r, u)| (r.clone(), sync_status(r, &lib.all), u.clone())).collect(),
                );
                v.rows = Some((key, Rc::clone(&r)));
                r
            }
        };
        let compatible = compatible.map(|id| {
            let key = (v.rev, lib.generation, id.to_owned());
            if let Some((k, ids)) = &v.compatible
                && *k == key
            {
                return Rc::clone(ids);
            }
            v.builds.compatible = v.builds.compatible.saturating_add(1);
            let table = &v.inputs.table;
            let mut pool: Vec<&ToolComponent> = lib.all.iter().map(|c| &c.component).collect();
            pool.extend(table.iter().map(|r| &r.component).filter(|c| !lib.all.iter().any(|l| l.component.id() == c.id())));
            let ids = match pool.iter().copied().find(|c| c.id() == id) {
                Some(sel) => {
                    let mut ids: Vec<String> = compatible_parents(sel, pool.iter().copied())
                        .into_iter()
                        .chain(compatible_children(sel, pool.iter().copied()))
                        .map(|f| f.component.id().to_owned())
                        .collect();
                    ids.dedup();
                    ids
                }
                None => Vec::new(),
            };
            let ids = Rc::new(ids);
            v.compatible = Some((key, Rc::clone(&ids)));
            ids
        });
        Derived { compatible, rows }
    }

    /// What the stack group offers for the stack draft `links`.
    pub(crate) fn component_stack_view(&mut self, links: &[AssemblyLink]) -> Rc<StackView> {
        self.read_component_inputs();
        let v = &mut self.component_views;
        if let Some(((rev, kept), view)) = &v.stack
            && *rev == v.rev
            && kept.as_slice() == links
        {
            return Rc::clone(view);
        }
        v.builds.stack = v.builds.stack.saturating_add(1);
        let view = Rc::new(StackView { offer: candidates(links, &v.inputs.table) });
        v.stack = Some(((v.rev, links.to_vec()), Rc::clone(&view)));
        view
    }
}
