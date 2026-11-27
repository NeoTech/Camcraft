//! The external NC program's `T` numbers mapped to cutter geometry (#2218):
//! an explicit `T -> tool` map, the document's tool numbers, or a `.camtools`
//! library row. Provenance: RS-274/NGC (NISTIR 6556) tool-select semantics;
//! the atoms follow CamCraft's own stock-sim law (`stock_sim::plan_ops`).

use flecs_ecs::prelude::World;

use crate::model;
use crate::tool_library::LibraryTool;
use crate::tools::{Tool, ToolKind};

use super::NcSimOptions;

/// Where a mapped tool's geometry came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NcToolSource {
    /// The caller's explicit `T -> tool` map.
    Explicit,
    /// A document tool that carries this number.
    Document,
    /// A `.camtools` library row that states this number.
    Library,
}

/// One `T` number with the cutter that the simulation sweeps for it.
#[derive(Clone, Debug)]
pub struct NcTool {
    pub number: u32,
    pub name: String,
    pub tool: Tool,
    pub source: NcToolSource,
}

/// The result of the tool map: the mapped tools, the `T` numbers with no
/// cutter geometry, and one reason for each unmapped number.
#[derive(Clone, Debug, Default)]
pub struct NcToolMap {
    pub tools: Vec<NcTool>,
    pub unmapped: Vec<u32>,
    pub reasons: Vec<String>,
}

impl NcToolMap {
    #[must_use]
    pub fn get(&self, number: u32) -> Option<&NcTool> {
        self.tools.iter().find(|t| t.number == number)
    }
}

/// The milling kinds a 3-axis stock atom sweeps. Every other kind (a lathe
/// insert, a probe, a torch, a wire, an electrode, an undercut head) has no
/// atom on this path and is refused by name.
const SWEPT: &[ToolKind] = &[
    ToolKind::EndMill,
    ToolKind::BallNose,
    ToolKind::BullNose,
    ToolKind::Drill,
    ToolKind::Tap,
    ToolKind::Bore,
    ToolKind::Taper,
    ToolKind::ThreadMill,
    ToolKind::ThreadMillSingle,
];

fn document_tool(world: &World, name: &str) -> Option<Tool> {
    let e = model::find_by_name(world, name)?;
    if !model::is_tool(world, &e) {
        return None;
    }
    model::tool_of(world, &e)
}

fn library_named<'a>(library: &'a [LibraryTool], name: &str) -> Option<&'a LibraryTool> {
    library.iter().find(|r| r.name == name)
}

fn resolve(world: &World, number: u32, opts: &NcSimOptions, numbered: &[(String, u32)]) -> Result<NcTool, String> {
    if let Some((_, name)) = opts.tool_map.iter().find(|(t, _)| *t == number) {
        let tool = library_named(&opts.library, name)
            .map(|r| r.tool.clone())
            .or_else(|| document_tool(world, name))
            .ok_or_else(|| format!("T{number}: the tool map names '{name}', which is no document tool and no library row"))?;
        return Ok(NcTool { number, name: name.clone(), tool, source: NcToolSource::Explicit });
    }
    if let Some((name, _)) = numbered.iter().find(|(_, n)| *n == number) {
        let tool = document_tool(world, name).ok_or_else(|| format!("T{number}: document tool '{name}' has no definition"))?;
        return Ok(NcTool { number, name: name.clone(), tool, source: NcToolSource::Document });
    }
    if let Some(row) = opts.library.iter().find(|r| r.number == Some(number)) {
        return Ok(NcTool { number, name: row.name.clone(), tool: row.tool.clone(), source: NcToolSource::Library });
    }
    Err(format!("T{number}: no document tool, library row or map entry carries this number"))
}

/// Why `tool` cannot be swept as a 3-axis cutter, or `None`.
fn geometry_problem(t: &NcTool) -> Option<String> {
    if !SWEPT.contains(&t.tool.kind) {
        return Some(format!(
            "T{}: '{}' is a {} - the NC simulation sweeps only milling cutters",
            t.number, t.name, t.tool.kind
        ));
    }
    if !t.tool.is_sane() {
        return Some(format!("T{}: '{}' has no sane cutter geometry", t.number, t.name));
    }
    None
}

/// Map each distinct `T` number of the program to cutter geometry.
///
/// # Errors
/// Returns the document's tool-number refusal (`tool_numbers::numbers`).
pub fn map_tools(world: &World, numbers: &[u32], opts: &NcSimOptions) -> Result<NcToolMap, String> {
    let numbered = crate::tool_numbers::numbers(world)?;
    let mut out = NcToolMap::default();
    for &n in numbers {
        if out.get(n).is_some() || out.unmapped.contains(&n) {
            continue;
        }
        match resolve(world, n, opts, &numbered) {
            Ok(t) => match geometry_problem(&t) {
                None => out.tools.push(t),
                Some(why) => {
                    out.unmapped.push(n);
                    out.reasons.push(why);
                }
            },
            Err(why) => {
                out.unmapped.push(n);
                out.reasons.push(why);
            }
        }
    }
    Ok(out)
}

/// The stock atom of a mapped cutter, by the same kind law as
/// `stock_sim::plan_ops`.
pub(super) struct Atom {
    pub radius: f64,
    pub ball: bool,
    /// The cone's cotangent of its half-angle (chamfer mill, pointed drill).
    pub cone: Option<f64>,
    /// The bull nose's corner radius.
    pub corner: Option<f64>,
    /// A twist drill's point.
    pub drill: bool,
}

/// The [`Atom`] of a mapped cutter.
///
/// # Errors
/// Refuses a cone or corner that states no sane geometry, by tool name.
pub(super) fn atom(t: &NcTool) -> Result<Atom, String> {
    let tool = &t.tool;
    let cone_of = |what: &str| -> Result<f64, String> {
        let tan_half = (tool.tip_angle.to_radians() * 0.5).tan();
        if tan_half.is_finite() && tan_half > 0.0 && tool.tip_angle > 0.0 && tool.tip_angle < 180.0 {
            Ok(1.0 / tan_half)
        } else {
            Err(format!("T{}: {what} '{}' carries no sane cone (tip angle {})", t.number, t.name, tool.tip_angle))
        }
    };
    let (cone, drill) = match tool.kind {
        ToolKind::Taper => (Some(cone_of("chamfer mill")?), false),
        ToolKind::Drill if (tool.tip_angle - 180.0).abs() > 1e-9 => (Some(cone_of("drill")?), true),
        _ => (None, false),
    };
    let corner = crate::stock_sim::bull_corner(tool).map_err(|e| format!("T{}: {e}", t.number))?;
    let radius = crate::operation::tool_law::cutting_radius(tool).map_err(|e| format!("T{}: {e}", t.number))?;
    Ok(Atom { radius, ball: tool.kind == ToolKind::BallNose, cone, corner, drill })
}
