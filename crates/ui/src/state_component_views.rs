//! #3898 (U7 of #3663): what the tool library window's component pages
//! derive from the document and the component library - each document row
//! with its library status and its users, the ids that fit the
//! "compatible" filter, and for the stack draft: what "Add component"
//! offers, the resolver's values (gauge length, cutting diameter, corner
//! radius; for a lathe tool the nose radius, the orientation and the
//! insert's inscribed circle) or its refusal, and for each row the parts
//! that "Swap" offers (only the replacements with which the whole stack
//! still resolves). A lathe tool's empty stack offers only the roots that
//! the lathe resolver takes (a rectangular shank or a round bar shank,
//! #3643); a spindle tool's empty stack leaves those out. Frame cost (`crates/ui/AGENTS.md`): State owns the caches, with
//! their input. The component table and the stacks that use each row are
//! read once for each document version; a set is built again only when
//! that read differs (`rev`), the library read changes, or the filter,
//! the open tool's record or the stack draft changes. A still frame reads and builds nothing
//! (`ViewBuilds`).
//! Provenance: our own caches over our own component records
//! (`component_fit`, `component_library::sync_status`,
//! `tool_assembly::derive`); no HSMWorks
//! material involved.

use std::rc::Rc;

use camcraft_app::component::{ComponentKind, CuttingItem, ToolComponent};
use camcraft_app::component_fit::{compatible_children, compatible_parents};
use camcraft_app::component_library::{SyncStatus, sync_status};
use camcraft_app::connection_code::ConnectionCode;
use camcraft_app::orientation;
use camcraft_app::schema::{AssemblyLink, DocumentComponent};
use camcraft_app::tool_assembly::{self as ta, Mount};
use camcraft_app::tool_body::LATHE_HELD;
use camcraft_app::tools::Tool;

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

/// The values a resolved stack gives its tool.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StackValues {
    /// From the cutting tip to the spindle gauge line (mm).
    pub gauge_len: f64,
    /// The cutting diameter (mm).
    pub diameter: f64,
    /// The corner radius (mm).
    pub corner_r: f64,
    /// For a lathe-held tool, what the lathe resolver gives (#3643).
    pub lathe: Option<LatheValues>,
}

/// The values a resolved lathe stack gives its tool: `diameter` above
/// holds the insert's inscribed circle and the gauge length is 0 (#4278).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LatheValues {
    /// The insert's nose radius (mm).
    pub nose_radius: f64,
    /// The tip orientation code (1-8), stated or derived.
    pub orientation: Option<u8>,
}

/// One replacement that "Swap" offers: its label and the whole new stack.
pub type SwapOffer = (String, Vec<AssemblyLink>);

/// What the stack group derives from the stack draft.
pub struct StackView {
    /// The parts that can go on the stack next ([`candidates`]).
    pub offer: Vec<(String, AssemblyLink)>,
    /// The resolver's values, or its named refusal; `None` for no stack.
    pub values: Option<Result<StackValues, String>>,
    /// For each row, the replacements with which the stack resolves.
    pub swaps: Vec<Vec<SwapOffer>>,
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
type StackKey = (u64, String, Option<Tool>, Vec<AssemblyLink>);

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

/// The parts that can go on a spindle tool's stack next ([`candidates_for`]).
#[must_use]
pub fn candidates(links: &[AssemblyLink], table: &[DocumentComponent]) -> Vec<(String, AssemblyLink)> {
    candidates_for(links, table, false)
}

/// Whether row `r` can start the stack of a lathe tool (`lathe`) or of
/// a spindle tool. The lathe resolver takes a rectangular shank or a round
/// bar shank as its root and needs an insert after it; a spindle stack
/// refuses a row with a lathe body, and a rectangular shank is a lathe
/// holder's.
fn starts(r: &DocumentComponent, lathe: bool) -> bool {
    let rect = matches!(r.component.mcs(), Some(ConnectionCode::RectShank { .. }));
    if lathe {
        let bar = matches!(r.component.mcs(), Some(ConnectionCode::Cylindrical { .. } | ConnectionCode::Weldon { .. }));
        (rect || bar) && !matches!(r.component.kind(), ComponentKind::CuttingItem(_))
    } else {
        !rect && r.lathe_body.is_none() && !matches!(r.component.kind(), ComponentKind::CuttingItem(CuttingItem::Insert(_)))
    }
}

/// The parts that can go on the stack next, with the link each one adds:
/// the holding parts for an empty stack (the lathe roots for a lathe
/// tool), else the parts that fit the last holding part (a cutting item
/// holds nothing).
#[must_use]
pub fn candidates_for(links: &[AssemblyLink], table: &[DocumentComponent], lathe: bool) -> Vec<(String, AssemblyLink)> {
    let comps: Vec<&ToolComponent> = table.iter().map(|r| &r.component).collect();
    if links.is_empty() {
        return table
            .iter()
            .filter(|r| starts(r, lathe))
            .map(|r| (r.component.name().to_owned(), AssemblyLink { component: r.component.id().to_owned(), mount: None }))
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

fn values(name: &str, tool: &Tool, links: &[AssemblyLink], table: &[DocumentComponent]) -> Result<StackValues, String> {
    ta::derive(name, tool, links, table).map(|d| {
        let lathe = is_lathe(tool).then(|| LatheValues {
            nose_radius: d.tool.nose_radius,
            orientation: d.tool.orientation.or_else(|| orientation::default_for(d.tool.kind)),
        });
        StackValues { gauge_len: d.gauge_len, diameter: d.tool.diameter, corner_r: d.tool.corner_r, lathe }
    })
}

/// Whether the lathe resolver builds `tool`'s stack (`tool_assembly::derive`).
fn is_lathe(tool: &Tool) -> bool {
    LATHE_HELD.contains(&tool.kind)
}

/// The replacements for row `at`: each part that fits where the row's
/// part sits, put in its place with the row's turn and (when stated) its
/// stick-out kept, and kept only when the whole stack then resolves.
fn swaps_at(name: &str, tool: &Tool, links: &[AssemblyLink], at: usize, table: &[DocumentComponent]) -> Vec<SwapOffer> {
    let (Some(prefix), Some(old)) = (links.get(..at), links.get(at)) else {
        return Vec::new();
    };
    candidates_for(prefix, table, is_lathe(tool))
        .into_iter()
        .filter(|(_, c)| c.component != old.component)
        .filter_map(|(label, mut c)| {
            if let (Some(m), Some(was)) = (c.mount.as_mut(), old.mount.as_ref()) {
                m.z_rot_deg = was.z_rot_deg;
                m.projection_mm = was.projection_mm.or(m.projection_mm);
            }
            let mut next = links.to_vec();
            if let Some(slot) = next.get_mut(at) {
                *slot = c;
            }
            values(name, tool, &next, table).is_ok().then_some((label, next))
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

    /// What the stack group derives for the stack draft `links` of tool
    /// `name`, whose record is now `tool` (`None`: not in the document).
    pub(crate) fn component_stack_view(&mut self, name: &str, tool: Option<&Tool>, links: &[AssemblyLink]) -> Rc<StackView> {
        self.read_component_inputs();
        let v = &mut self.component_views;
        if let Some(((rev, kept_name, kept_tool, kept), view)) = &v.stack
            && *rev == v.rev
            && kept_name == name
            && kept_tool.as_ref() == tool
            && kept.as_slice() == links
        {
            return Rc::clone(view);
        }
        v.builds.stack = v.builds.stack.saturating_add(1);
        let table = &v.inputs.table;
        let (values, swaps) = match tool {
            Some(t) if !links.is_empty() => (
                Some(values(name, t, links, table)),
                (0..links.len()).map(|i| swaps_at(name, t, links, i, table)).collect(),
            ),
            Some(_) => (None, Vec::new()),
            None => (Some(Err(format!("tool '{name}' is not in the document"))), Vec::new()),
        };
        let lathe = tool.is_some_and(is_lathe);
        let view = Rc::new(StackView { offer: candidates_for(links, table, lathe), values, swaps });
        v.stack = Some(((v.rev, name.to_owned(), tool.cloned(), links.to_vec()), Rc::clone(&view)));
        view
    }
}
