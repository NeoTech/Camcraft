//! #3645: the wire form of a document tool component - one row of the
//! `components` table of a `.camproj` (v84) and of a user component
//! library (`camcraft-components/1`) - and the content hash that a
//! library link compares. Our own JSON over the public ISO 13399
//! component roles (the #3642 and #3643 records); no vendor library
//! format was read or consulted.
//!
//! The law: strict, all or nothing. Each row is built again through the
//! checked constructors (`ToolComponent::new`, `ComponentGeometry::new`
//! and `with_stop`, `Frame::new`, `ComponentBody::new`), an unknown key
//! is refused, and each refusal names the row by its id. #3646: a
//! multi-seat cutter body's catalog facts (`milling`) and an insert's
//! running data (`running`) are optional keys; a fact on the wrong kind
//! of row is refused by the row law of the component commands. #3643: a
//! lathe component's body (`lathe_body`) is an optional key too.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use camcraft_tools::component::{ComponentKind, CuttingItem, InsertSeat, ToolComponent};
use camcraft_tools::component_geometry::{ComponentBody, ComponentGeometry};
use camcraft_tools::connection_code::ConnectionCode;
use camcraft_tools::frame::Frame;
use camcraft_tools::insert_codes::{Clearance, Hand, InsertShape};
use camcraft_tools::insert_iso1832::{InsertSpec, Thickness};
use camcraft_tools::insert_seat::{Clamping, HolderStyle, InsertSeatSpec};
use camcraft_tools::spun::SpunPoint;

use camcraft_tools::milling_cutter::MillingBody;

use super::{CuttingRowDto, ToolMaterialDto};
#[path = "component_wire_lathe.rs"]
mod lathe;
pub use lathe::LatheBodyDto;
use crate::schema::{DocumentComponent, InsertRunning, LibraryRef};

/// One tool component on file.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ComponentDto {
    pub id: String,
    pub name: String,
    pub item: ItemDto,
    #[serde(default, skip_serializing_if = "GeometryDto::is_empty")]
    pub geometry: GeometryDto,
    /// The user library copy the row was taken from (a project only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub library_ref: Option<LibraryRefDto>,
    /// #3646: a multi-seat cutter body's catalog facts (a tool item only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub milling: Option<MillingDto>,
    /// #3646: an insert's own tool material and cutting rows (an insert only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub running: Option<RunningDto>,
    /// #3643: a lathe component's body (facets in its own frame).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lathe_body: Option<LatheBodyDto>,
}

/// #3646: the catalog facts of a multi-seat cutter body - `DC`, `KAPR`
/// (degrees), `APMX`, centre cutting and an optional seat count.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MillingDto {
    pub dc: f64,
    pub kapr_deg: f64,
    pub apmx: f64,
    pub center_cutting: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seat_count: Option<u32>,
}

/// #3646: what an insert gives each tool that cuts with it.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct RunningDto {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_material: Option<ToolMaterialDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cutting: Vec<CuttingRowDto>,
}

fn milling_dto(m: &MillingBody) -> MillingDto {
    MillingDto { dc: m.dc, kapr_deg: m.kapr_deg, apmx: m.apmx, center_cutting: m.center_cutting, seat_count: m.seat_count }
}

fn running_dto(r: &InsertRunning) -> RunningDto {
    RunningDto {
        tool_material: r.tool_material.map(|m| match m {
            camcraft_tools::cutting::ToolMaterial::Hss => ToolMaterialDto::Hss,
            camcraft_tools::cutting::ToolMaterial::Carbide => ToolMaterialDto::Carbide,
        }),
        cutting: r.cutting.iter().map(CuttingRowDto::from_row).collect(),
    }
}

fn running(r: &RunningDto) -> InsertRunning {
    InsertRunning {
        tool_material: r.tool_material.map(|m| match m {
            ToolMaterialDto::Hss => camcraft_tools::cutting::ToolMaterial::Hss,
            ToolMaterialDto::Carbide => camcraft_tools::cutting::ToolMaterial::Carbide,
        }),
        cutting: r.cutting.iter().map(|c| c.to_row()).collect(),
    }
}

/// The component kind and its codes. Codes are written as their text
/// (`InsertSpec`, `ConnectionCode`), which parses back to the same value.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ItemDto {
    Insert { code: String },
    /// `tool` is the tool record as a `.camproj` tool states it, without
    /// a body, a holder, a cutter, a number or an assembly.
    Solid { mcs: String, tool: Value },
    ToolItem { mcs: String, seats: Vec<SeatDto> },
    Adaptive { mcs: String, csw: String },
}

/// One insert seat: the ISO 5608 letters of the pocket, its insert size
/// code and the thickness it takes (when the holder states one).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SeatDto {
    pub port: String,
    pub clamping: char,
    pub shape: char,
    pub style: char,
    pub clearance: char,
    pub hand: char,
    pub size: u8,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thickness: Option<String>,
}

/// Where the ports, the cutting point and the body sit.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default)]
#[serde(deny_unknown_fields)]
pub struct GeometryDto {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub ports: Vec<PortDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<BodyDto>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cip: Option<FrameDto>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stops: Vec<StopDto>,
}

impl GeometryDto {
    fn is_empty(&self) -> bool {
        *self == Self::default()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FrameDto {
    pub origin: [f64; 3],
    pub z: [f64; 3],
    pub x: [f64; 3],
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PortDto {
    pub name: String,
    pub frame: FrameDto,
}

/// A revolved body: `[z, r, arc]` per point tip-up, its height 0 at
/// `base` along the component's Z.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BodyDto {
    pub base: f64,
    pub points: Vec<[f64; 3]>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct StopDto {
    pub port: String,
    pub depth: f64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LibraryRefDto {
    pub id: String,
    pub content_hash: String,
}

fn frame_dto(f: &Frame) -> FrameDto {
    FrameDto { origin: f.origin(), z: f.z(), x: f.x() }
}

fn seat_dto(s: &InsertSeat) -> SeatDto {
    let p = &s.spec;
    SeatDto {
        port: s.port.clone(),
        clamping: p.clamping.letter(),
        shape: p.shape.letter(),
        style: p.style.letter(),
        clearance: p.clearance.letter(),
        hand: p.hand.letter(),
        size: p.size,
        thickness: p.thickness.map(Thickness::code),
    }
}

fn item_dto(kind: &ComponentKind) -> ItemDto {
    match kind {
        ComponentKind::CuttingItem(CuttingItem::Insert(spec)) => ItemDto::Insert { code: spec.to_string() },
        ComponentKind::CuttingItem(CuttingItem::Solid { tool, mcs }) => {
            ItemDto::Solid { mcs: mcs.to_string(), tool: super::tool_to_json(tool) }
        }
        ComponentKind::ToolItem { mcs, seats } => {
            ItemDto::ToolItem { mcs: mcs.to_string(), seats: seats.iter().map(seat_dto).collect() }
        }
        ComponentKind::AdaptiveItem { mcs, csw } => ItemDto::Adaptive { mcs: mcs.to_string(), csw: csw.to_string() },
    }
}

fn geometry_dto(g: &ComponentGeometry) -> GeometryDto {
    GeometryDto {
        ports: g.ports().iter().map(|(name, f)| PortDto { name: name.clone(), frame: frame_dto(f) }).collect(),
        body: g.body().map(|b| BodyDto {
            base: b.base(),
            points: b.profile().points().iter().map(|p| [p.z, p.r, p.arc]).collect(),
        }),
        cip: g.cip().map(frame_dto),
        stops: g
            .ports()
            .iter()
            .filter_map(|(name, _)| g.stop(name).map(|depth| StopDto { port: name.clone(), depth }))
            .collect(),
    }
}

/// The wire form of a row.
#[must_use]
pub fn component_to_dto(row: &DocumentComponent) -> ComponentDto {
    ComponentDto {
        id: row.component.id().to_owned(),
        name: row.component.name().to_owned(),
        item: item_dto(row.component.kind()),
        geometry: geometry_dto(&row.geometry),
        library_ref: row
            .library_ref
            .as_ref()
            .map(|r| LibraryRefDto { id: r.id.clone(), content_hash: r.content_hash.clone() }),
        milling: row.milling.as_ref().map(milling_dto),
        running: row.running.as_ref().map(running_dto),
        lathe_body: row.lathe_body.as_ref().map(lathe::lathe_body_dto),
    }
}

/// The content hash of a component and its geometry: FNV-1a 64 over
/// the compact wire JSON of its name, item and geometry, so the same
/// content gives the same hash in each build and on each machine. The
/// id and the library link are not content: a document row and its
/// library copy can have different ids. The #3646 cutter facts and
/// running data and the #3643 lathe body are not in the hash either: a
/// library copy does not hold them, and a sync keeps the document row's
/// own.
#[must_use]
pub fn content_hash(component: &ToolComponent, geometry: &ComponentGeometry) -> String {
    let content = (component.name(), item_dto(component.kind()), geometry_dto(geometry));
    // The write cannot fail, so the default is never taken: the value is
    // our own DTOs and a `Value`, every map key is a string, and
    // serde_json writes a non-finite number as null, not as an error.
    let bytes = serde_json::to_vec(&content).unwrap_or_default();
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h = (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("fnv1a64:{h:016x}")
}

fn code(text: &str, side: &str) -> Result<ConnectionCode, String> {
    ConnectionCode::parse(text).map_err(|e| format!("the {side} code: {e}"))
}

fn letter<T>(c: char, what: &str, f: impl Fn(char) -> Option<T>) -> Result<T, String> {
    f(c).ok_or_else(|| format!("the {what} letter '{c}' is not a known letter"))
}

fn seat(s: &SeatDto) -> Result<InsertSeat, String> {
    let at = |e: String| format!("seat '{}': {e}", s.port);
    let thickness = match &s.thickness {
        None => None,
        Some(t) => Some(Thickness::from_code(t).ok_or_else(|| at(format!("the thickness code '{t}' is not a known code")))?),
    };
    let spec = InsertSeatSpec {
        clamping: letter(s.clamping, "clamping", Clamping::from_letter).map_err(at)?,
        shape: letter(s.shape, "insert shape", InsertShape::from_letter).map_err(at)?,
        style: letter(s.style, "holder style", HolderStyle::from_letter).map_err(at)?,
        clearance: letter(s.clearance, "clearance", Clearance::from_letter).map_err(at)?,
        hand: letter(s.hand, "hand", Hand::from_letter).map_err(at)?,
        size: s.size,
        thickness,
    };
    Ok(InsertSeat { port: s.port.clone(), spec })
}

/// The keys a component's tool record must not state: they belong to a
/// document tool, not to a catalog component.
const TOOL_ONLY_KEYS: [&str; 5] = ["body", "holder", "cutter", "number", "assembly"];

fn kind(item: &ItemDto) -> Result<ComponentKind, String> {
    Ok(match item {
        ItemDto::Insert { code } => ComponentKind::CuttingItem(CuttingItem::Insert(
            InsertSpec::parse(code).map_err(|e| format!("the insert code: {e}"))?,
        )),
        ItemDto::Solid { mcs, tool } => {
            if let Some(k) = TOOL_ONLY_KEYS.iter().find(|k| tool.get(**k).is_some_and(|v| !v.is_null())) {
                return Err(format!("the solid tool states \"{k}\", which a component does not hold"));
            }
            let tool = super::tool_from_json(tool).map_err(|e| format!("the solid tool is malformed: {e}"))?;
            ComponentKind::CuttingItem(CuttingItem::Solid { tool: Box::new(tool), mcs: code(mcs, "machine side")? })
        }
        ItemDto::ToolItem { mcs, seats } => ComponentKind::ToolItem {
            mcs: code(mcs, "machine side")?,
            seats: seats.iter().map(seat).collect::<Result<_, _>>()?,
        },
        ItemDto::Adaptive { mcs, csw } => {
            ComponentKind::AdaptiveItem { mcs: code(mcs, "machine side")?, csw: code(csw, "workpiece side")? }
        }
    })
}

fn frame(f: &FrameDto, what: &str) -> Result<Frame, String> {
    Frame::new(f.origin, f.z, f.x).map_err(|e| format!("the {what} frame: {e}"))
}

fn geometry(g: &GeometryDto) -> Result<ComponentGeometry, String> {
    let ports = g
        .ports
        .iter()
        .map(|p| frame(&p.frame, &format!("port '{}'", p.name)).map(|f| (p.name.clone(), f)))
        .collect::<Result<Vec<_>, _>>()?;
    let body = match &g.body {
        None => None,
        Some(b) => {
            let points = b.points.iter().map(|p| SpunPoint { z: p[0], r: p[1], arc: p[2] }).collect();
            Some(ComponentBody::new(points, b.base).map_err(|e| e.to_string())?)
        }
    };
    let cip = g.cip.as_ref().map(|f| frame(f, "cutting point")).transpose()?;
    let mut out = ComponentGeometry::new(ports, body, cip).map_err(|e| e.to_string())?;
    for (i, s) in g.stops.iter().enumerate() {
        if g.stops[..i].iter().any(|o| o.port == s.port) {
            return Err(format!("the port '{}' has two bore stops", s.port));
        }
        out = out.with_stop(&s.port, s.depth).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// The checked row of `dto`, or the refusal, naming the row.
pub fn component_from_dto(dto: &ComponentDto) -> Result<DocumentComponent, String> {
    let named = |e: String| format!("tool component '{}': {e}", dto.id.trim());
    let component = ToolComponent::new(&dto.id, &dto.name, kind(&dto.item).map_err(named)?).map_err(|e| named(e.to_string()))?;
    let geometry = geometry(&dto.geometry).map_err(named)?;
    let library_ref = match &dto.library_ref {
        None => None,
        Some(r) if r.id.trim().is_empty() || r.content_hash.trim().is_empty() => {
            return Err(named("its library link needs an id and a content hash".to_owned()));
        }
        Some(r) => Some(LibraryRef { id: r.id.trim().to_owned(), content_hash: r.content_hash.trim().to_owned() }),
    };
    let row = DocumentComponent {
        component,
        geometry,
        library_ref,
        milling: dto.milling.as_ref().map(|m| MillingBody {
            dc: m.dc,
            kapr_deg: m.kapr_deg,
            apmx: m.apmx,
            center_cutting: m.center_cutting,
            seat_count: m.seat_count,
        }),
        running: dto.running.as_ref().map(running),
        lathe_body: dto.lathe_body.as_ref().map(lathe::lathe_body).transpose().map_err(named)?,
    };
    if let Some(why) = crate::commands::tool_assembly::row_refusal(&row) {
        return Err(why);
    }
    Ok(row)
}

/// Every row of `rows`, checked whole: the first refused row, or an id
/// that is there twice, refuses all of them by name.
pub fn components_from_dtos(rows: &[ComponentDto]) -> Result<Vec<DocumentComponent>, String> {
    let mut out: Vec<DocumentComponent> = Vec::with_capacity(rows.len());
    for dto in rows {
        let row = component_from_dto(dto)?;
        if out.iter().any(|r| r.component.id() == row.component.id()) {
            return Err(format!("tool component '{}' is there twice", row.component.id()));
        }
        out.push(row);
    }
    Ok(out)
}
