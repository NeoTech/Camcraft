//! #4339: the plan that converts standalone document tools to component
//! stacks. For each tool it proposes the stack that gives the tool - the
//! tool's library holder as the built-in holder component, a collet that
//! takes the shank where the holder needs one, and the tool itself as a
//! solid cutting item, stuck out as the holder reference states - and
//! judges it: the same tool record and gauge length ("same"), the values
//! that differ ("different"), or no stack with the reason by name ("no
//! match"). Nothing is written here; `commands::ConvertToComponents`
//! lands the selected rows in one undo step. Our own design over the ISO
//! 13399 component roles; no vendor source consulted.

use flecs_ecs::prelude::{Entity, World};

use camcraft_tools::component::{ComponentKind, CuttingItem, ToolComponent, ADAPTIVE_PORT};
use camcraft_tools::component_fit::check_fit;
use camcraft_tools::component_geometry::ComponentGeometry;
use camcraft_tools::connection_code::ConnectionCode;

use crate::commands::tool_assembly::ToolUpdate;
use crate::component_library::{builtins, BUILTIN_PREFIX};
use crate::model;
use crate::schema::{AssemblyLink, DocumentComponent, LibraryRef};
use crate::tool_assembly::{self as ta, Mount};
use crate::tool_body::{self, LATHE_HELD};
use crate::tools::Tool;

/// The id space of the per-tool cutting items that a conversion makes.
pub const CUTTING_PREFIX: &str = "user:tool:";
/// Two gauge lengths closer than this (mm) are the same.
const GAUGE_TOL: f64 = 1e-6;

/// The judgement of one tool's proposed stack.
#[derive(Clone, Debug, PartialEq)]
pub enum Verdict {
    /// The stack derives the same tool record and gauge length.
    Same,
    /// The stack derives other values: each difference, in words.
    Different(Vec<String>),
    /// No stack gives the tool: the reason by name.
    NoMatch(String),
}

impl Verdict {
    /// The word of the verdict: "same", "different" or "no match".
    #[must_use]
    pub fn word(&self) -> &'static str {
        match self {
            Self::Same => "same",
            Self::Different(_) => "different",
            Self::NoMatch(_) => "no match",
        }
    }
}

/// One row of the preview.
#[derive(Clone, Debug, PartialEq)]
pub struct ConvertRow {
    pub tool: String,
    /// The proposed stack, root first (empty for "no match").
    pub links: Vec<AssemblyLink>,
    /// The names of the stack's components, root first.
    pub names: Vec<String>,
    /// The components the document must take for this stack.
    pub adds: Vec<DocumentComponent>,
    pub verdict: Verdict,
}

impl ConvertRow {
    fn no(tool: &str, why: impl Into<String>) -> Self {
        Self { tool: tool.to_owned(), links: Vec::new(), names: Vec::new(), adds: Vec::new(), verdict: Verdict::NoMatch(why.into()) }
    }
}

/// The document tools (sorted by name) that have no component stack.
#[must_use]
pub fn standalone_tools(world: &World) -> Vec<String> {
    let mut out: Vec<String> = model::named_entities(world)
        .into_iter()
        .filter(|(_, e)| model::is_tool(world, e) && ta::assembly_of(world, e).is_none())
        .map(|(n, _)| n)
        .collect();
    out.sort();
    out
}

/// The preview rows of `names` (every standalone tool when empty).
/// Refused by name: a name the document does not hold as a tool.
pub fn plan(world: &World, names: &[String]) -> Result<Vec<ConvertRow>, String> {
    let names = if names.is_empty() { standalone_tools(world) } else { names.to_vec() };
    let table = ta::components(world);
    let library = library_rows();
    let mut rows = Vec::with_capacity(names.len());
    for name in &names {
        let entity = model::find_by_name(world, name)
            .filter(|e| model::is_tool(world, e))
            .ok_or_else(|| format!("no tool named '{name}'"))?;
        rows.push(row(world, name, entity, &table, &library));
    }
    Ok(rows)
}

/// The built-in components as document rows linked to their library
/// copy (the link `SyncDocumentComponent` would record).
fn library_rows() -> Vec<DocumentComponent> {
    builtins()
        .into_iter()
        .map(|e| {
            let link = LibraryRef { id: e.component.id().to_owned(), content_hash: e.content_hash() };
            DocumentComponent { library_ref: Some(link), ..DocumentComponent::new(e.component, e.geometry) }
        })
        .collect()
}

/// The proposal and the verdict for one tool.
fn row(world: &World, name: &str, entity: Entity, table: &[DocumentComponent], library: &[DocumentComponent]) -> ConvertRow {
    let Some(current) = model::tool_of(world, &entity) else {
        return ConvertRow::no(name, "it is not a tool");
    };
    if ta::assembly_of(world, &entity).is_some() {
        return ConvertRow::no(name, "it has a component stack already");
    }
    if LATHE_HELD.contains(&current.kind) {
        return ConvertRow::no(name, format!("no holder: a {} sits in a lathe holder, which has no built-in component", current.kind));
    }
    let Some(href) = tool_body::body_of(world, &entity).and_then(|b| b.library) else {
        return ConvertRow::no(name, "no holder: the tool names no library holder");
    };
    let holder_id = format!("{BUILTIN_PREFIX}holder:{}", href.entry);
    // The document's copy first, then the built-in one.
    let pick = |id: &str| table.iter().chain(library).find(|r| r.component.id() == id).cloned();
    let Some(holder) = pick(&holder_id) else {
        return ConvertRow::no(name, format!("no holder: '{}' has no built-in holder component", href.entry));
    };
    // A record whose holder nose sits at or above the shank's end holds
    // no shank in the holder, which a stack refuses: the cutting item
    // takes one shank diameter more of shank, in the bore, and the gauge
    // length stays. The longer shank shows as a difference.
    let full = current.flute_len + current.shank_len;
    let item = if href.projection >= full {
        Tool { shank_len: href.projection - current.flute_len + current.shank_d.max(1.0), ..current.clone() }
    } else {
        current.clone()
    };
    let cutting = match cutting_row(name, &item, table) {
        Ok(c) => c,
        Err(why) => return ConvertRow::no(name, why),
    };
    let mut chain = vec![holder.clone()];
    if check_fit(&holder.component, ADAPTIVE_PORT, &cutting.component).is_err() {
        let takes = |r: &&DocumentComponent| {
            matches!(r.component.kind(), ComponentKind::AdaptiveItem { .. })
                && r.component.id() != holder_id
                && check_fit(&holder.component, ADAPTIVE_PORT, &r.component).is_ok()
        };
        let collets: Vec<&DocumentComponent> = table.iter().chain(library).filter(takes).collect();
        let Some(collet) = collets.iter().find(|c| check_fit(&c.component, ADAPTIVE_PORT, &cutting.component).is_ok()) else {
            let d = current.shank_d;
            return ConvertRow::no(
                name,
                if collets.is_empty() {
                    format!("the holder '{}' does not take the shank {d} mm", href.entry)
                } else {
                    format!("no collet for the shank {d} mm in the holder '{}'", href.entry)
                },
            );
        };
        chain.push((*collet).clone());
    }
    chain.push(cutting);
    let last = chain.len().saturating_sub(1);
    let links: Vec<AssemblyLink> = chain
        .iter()
        .enumerate()
        .map(|(i, r)| AssemblyLink {
            component: r.component.id().to_owned(),
            mount: (i > 0).then(|| Mount {
                parent: i - 1,
                port: Some(ADAPTIVE_PORT.to_owned()),
                z_rot_deg: 0.0,
                projection_mm: (i == last && href.projection.is_finite() && href.projection > 0.0).then_some(href.projection),
            }),
        })
        .collect();
    let names = chain.iter().map(|r| r.component.name().to_owned()).collect();
    let adds: Vec<DocumentComponent> =
        chain.into_iter().filter(|r| !table.iter().any(|t| t.component.id() == r.component.id())).collect();
    let mut next = table.to_vec();
    next.extend(adds.iter().cloned());
    let derived = match ta::derive(name, &current, &links, &next) {
        Ok(d) => d,
        Err(why) => return ConvertRow::no(name, why),
    };
    // Every law the command judges (a body in the flutes, an operation
    // the record strands) - a refused stack is no match, by its reason.
    if let Err(e) = ToolUpdate::new(world, name, &links, &next) {
        return ConvertRow::no(name, e.to_string());
    }
    let diffs = differences(&current, &derived.tool, href.gauge_len + href.projection, derived.gauge_len);
    let verdict = if diffs.is_empty() { Verdict::Same } else { Verdict::Different(diffs) };
    ConvertRow { tool: name.to_owned(), links, names, adds, verdict }
}

/// The tool itself as a solid cutting item on a plain shank of its own
/// shank diameter, under `user:tool:<name>`. Refused by name: a record
/// the component law refuses, and an id the document holds for another
/// component.
fn cutting_row(name: &str, tool: &Tool, table: &[DocumentComponent]) -> Result<DocumentComponent, String> {
    let id = format!("{CUTTING_PREFIX}{name}");
    let kind = ComponentKind::CuttingItem(CuttingItem::Solid {
        tool: Box::new(tool.clone()),
        mcs: ConnectionCode::Cylindrical { d: tool.shank_d },
    });
    let component = ToolComponent::new(&id, name, kind).map_err(|e| format!("the tool is not a cutting item: {e}"))?;
    let row = DocumentComponent::new(component, ComponentGeometry::default());
    match table.iter().find(|r| r.component.id() == id) {
        Some(held) if *held != row => Err(format!("the document holds another tool component '{id}'")),
        _ => Ok(row),
    }
}

/// The differences between the tool now and the derived tool, in words.
#[must_use]
pub fn differences(now: &Tool, next: &Tool, gauge_now: f64, gauge_next: f64) -> Vec<String> {
    let mut out = Vec::new();
    if !(gauge_now - gauge_next).abs().le(&GAUGE_TOL) {
        out.push(format!("gauge length {gauge_now:.3} -> {gauge_next:.3} mm"));
    }
    if now == next {
        return out;
    }
    if now.kind != next.kind {
        out.push(format!("type {} -> {}", now.kind, next.kind));
    }
    let lengths = [
        ("diameter", now.diameter, next.diameter),
        ("corner radius", now.corner_r, next.corner_r),
        ("flute length", now.flute_len, next.flute_len),
        ("shank diameter", now.shank_d, next.shank_d),
        ("shank length", now.shank_len, next.shank_len),
        ("holder diameter", now.holder_d, next.holder_d),
        ("holder length", now.holder_len, next.holder_len),
    ];
    let before = out.len();
    for (what, a, b) in lengths {
        if a.to_bits() != b.to_bits() {
            out.push(format!("{what} {a:.3} -> {b:.3} mm"));
        }
    }
    if out.len() == before && now.kind == next.kind {
        out.push("other values of the tool record".to_owned());
    }
    out
}
