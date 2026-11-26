//! #3642: a reusable tool component, in the three kinds of the
//! ISO 13399 tool model - a cutting item (an insert, or a solid
//! tool), a tool item (a holder or cutter body with insert seats) and
//! an adaptive item (a spindle holder, chuck, collet, adapter or
//! extension). Each kind states its machine-side connection (MCS) and
//! its workpiece-side ports (CSW). Our own record over the public
//! ISO 13399 concepts; pure data, no ECS and no file I/O. Assemblies
//! (#3643), the document table (#3644) and the stores (#3645) build on
//! this.
//!
//! The law: a component is checked when it is made
//! ([`ToolComponent::new`]), and each refusal names the component. The
//! check covers the specs that it is given (an insert, a seat: their
//! fields are public), and the connections are stored in their
//! canonical form.

use crate::connection_code::{ConnectionCode, ConnectionError};
use crate::holder_library::BORE_SLACK;
use crate::insert_iso1832::{DesignationError, InsertSpec};
use crate::insert_seat::InsertSeatSpec;
use crate::tools::Tool;

/// The one workpiece-side port of an adaptive item.
pub const ADAPTIVE_PORT: &str = "CSW";

/// What cuts.
#[derive(Clone, Debug, PartialEq)]
pub enum CuttingItem {
    /// An indexable insert. It has no machine side: it sits in a seat.
    Insert(InsertSpec),
    /// A solid tool (an end mill, a drill) and its shank connection.
    Solid { tool: Box<Tool>, mcs: ConnectionCode },
}

/// One insert seat of a tool item, under its port name ("CSW1",
/// "CSW1_2").
#[derive(Clone, Debug, PartialEq)]
pub struct InsertSeat {
    pub port: String,
    pub spec: InsertSeatSpec,
}

/// The three component kinds.
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentKind {
    CuttingItem(CuttingItem),
    /// A turning holder, a boring bar body or an indexable cutter
    /// body: one machine side and one or more insert seats.
    ToolItem { mcs: ConnectionCode, seats: Vec<InsertSeat> },
    /// A spindle holder, collet chuck, collet, adapter, extension or
    /// clamping unit: one machine side and one workpiece side.
    AdaptiveItem { mcs: ConnectionCode, csw: ConnectionCode },
}

/// Why a component is refused when it is made.
#[derive(Clone, Debug, PartialEq)]
pub enum ComponentError {
    EmptyId,
    EmptyName { id: String },
    NoSeat { name: String },
    BadPort { name: String, port: String },
    DuplicatePort { name: String, port: String },
    Connection { name: String, side: &'static str, err: ConnectionError },
    Tool { name: String, why: String },
    ShankMismatch { name: String, shank_d: f64, mcs: String },
    /// An insert spec that its ISO 1832 code would refuse.
    Insert { name: String, err: Box<DesignationError> },
    /// A seat spec that its ISO 5608 code would refuse.
    Seat { name: String, port: String, err: Box<DesignationError> },
}

impl core::fmt::Display for ComponentError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::EmptyId => write!(f, "a tool component needs an id"),
            Self::EmptyName { id } => write!(f, "the tool component '{id}' needs a name"),
            Self::NoSeat { name } => write!(f, "the tool item '{name}' has no insert seat - it needs one at least"),
            Self::BadPort { name, port } => write!(
                f,
                "the tool item '{name}' has a seat port '{port}' - a seat port is CSW and a number (CSW1, CSW1_2)"
            ),
            Self::DuplicatePort { name, port } => write!(f, "the tool item '{name}' has two seats named '{port}'"),
            Self::Connection { name, side, err } => write!(f, "the component '{name}' {side}: {err}"),
            Self::Tool { name, why } => write!(f, "the solid tool '{name}' is refused: {why}"),
            Self::ShankMismatch { name, shank_d, mcs } => write!(
                f,
                "the solid tool '{name}' has a Ø{shank_d} mm shank, but its machine side says {mcs}"
            ),
            Self::Insert { name, err } => write!(f, "the insert '{name}' is refused: {err}"),
            Self::Seat { name, port, err } => write!(f, "the tool item '{name}' seat '{port}' is refused: {err}"),
        }
    }
}

impl std::error::Error for ComponentError {}

/// One reusable tool component.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolComponent {
    id: String,
    name: String,
    kind: ComponentKind,
}

impl ToolComponent {
    /// A checked component: a non-empty id and name, valid connection
    /// codes (stored in their canonical form), a valid insert spec, at
    /// least one seat on a tool item with unique `CSW<n>` port names
    /// and valid seat specs, and a sane solid tool whose shank matches
    /// its machine side.
    pub fn new(id: &str, name: &str, mut kind: ComponentKind) -> Result<Self, ComponentError> {
        let id = id.trim();
        let name = name.trim();
        if id.is_empty() {
            return Err(ComponentError::EmptyId);
        }
        if name.is_empty() {
            return Err(ComponentError::EmptyName { id: id.to_owned() });
        }
        let named = name.to_owned();
        let conn = |side: &'static str, code: &mut ConnectionCode| -> Result<(), ComponentError> {
            *code = code.canonical().map_err(|err| ComponentError::Connection { name: named.clone(), side, err })?;
            Ok(())
        };
        match &mut kind {
            ComponentKind::CuttingItem(CuttingItem::Insert(spec)) => {
                spec.check().map_err(|err| ComponentError::Insert { name: named.clone(), err: Box::new(err) })?;
            }
            ComponentKind::CuttingItem(CuttingItem::Solid { tool, mcs }) => {
                conn("machine side", mcs)?;
                if let Some(why) = tool.why_not_sane() {
                    return Err(ComponentError::Tool { name: named, why });
                }
                if let ConnectionCode::Cylindrical { d } | ConnectionCode::Weldon { d } = &*mcs
                    && (d - tool.shank_d).abs() > BORE_SLACK
                {
                    return Err(ComponentError::ShankMismatch {
                        name: named,
                        shank_d: tool.shank_d,
                        mcs: mcs.to_string(),
                    });
                }
            }
            ComponentKind::ToolItem { mcs, seats } => {
                conn("machine side", mcs)?;
                if seats.is_empty() {
                    return Err(ComponentError::NoSeat { name: named });
                }
                for (i, seat) in seats.iter().enumerate() {
                    if !is_seat_port(&seat.port) {
                        return Err(ComponentError::BadPort { name: named, port: seat.port.clone() });
                    }
                    if seats[..i].iter().any(|s| s.port == seat.port) {
                        return Err(ComponentError::DuplicatePort { name: named, port: seat.port.clone() });
                    }
                    let port = || seat.port.clone();
                    seat.spec.check().map_err(|err| ComponentError::Seat { name: named.clone(), port: port(), err: Box::new(err) })?;
                }
            }
            ComponentKind::AdaptiveItem { mcs, csw } => {
                conn("machine side", mcs)?;
                conn("workpiece side", csw)?;
            }
        }
        Ok(Self { id: id.to_owned(), name: name.to_owned(), kind })
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn kind(&self) -> &ComponentKind {
        &self.kind
    }

    /// The machine-side connection; `None` for an insert.
    #[must_use]
    pub fn mcs(&self) -> Option<&ConnectionCode> {
        match &self.kind {
            ComponentKind::CuttingItem(CuttingItem::Insert(_)) => None,
            ComponentKind::CuttingItem(CuttingItem::Solid { mcs, .. })
            | ComponentKind::ToolItem { mcs, .. }
            | ComponentKind::AdaptiveItem { mcs, .. } => Some(mcs),
        }
    }

    /// The insert, when this is one.
    #[must_use]
    pub fn insert(&self) -> Option<&InsertSpec> {
        match &self.kind {
            ComponentKind::CuttingItem(CuttingItem::Insert(spec)) => Some(spec),
            _ => None,
        }
    }

    /// The seat behind a port name, when this is a tool item that has
    /// it.
    #[must_use]
    pub fn seat(&self, port: &str) -> Option<&InsertSeatSpec> {
        match &self.kind {
            ComponentKind::ToolItem { seats, .. } => seats.iter().find(|s| s.port == port).map(|s| &s.spec),
            _ => None,
        }
    }

    /// The workpiece-side port names, in order: the seats of a tool
    /// item, the one `CSW` of an adaptive item, none for a cutting item.
    #[must_use]
    pub fn ports(&self) -> Vec<&str> {
        match &self.kind {
            ComponentKind::CuttingItem(_) => Vec::new(),
            ComponentKind::ToolItem { seats, .. } => seats.iter().map(|s| s.port.as_str()).collect(),
            ComponentKind::AdaptiveItem { .. } => vec![ADAPTIVE_PORT],
        }
    }
}

/// "CSW", then a number, then optionally "_" and a number.
fn is_seat_port(port: &str) -> bool {
    let Some(rest) = port.strip_prefix(ADAPTIVE_PORT) else { return false };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    match rest.split_once('_') {
        Some((a, b)) => digits(a) && digits(b),
        None => digits(rest),
    }
}
