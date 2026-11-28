//! #3642: does one tool component fit another? An insert in a holder's
//! seat (shape, clearance, size, thickness, clamping against the
//! fixing hole, hand), and a child's machine side on a parent's
//! workpiece side (the connection codes). Plus the two library
//! queries a browser asks: what this component can sit in, and what
//! can sit in it. Our own rules from general machining practice over
//! the ISO 1832, ISO 5608 and ISO 13399 codes; pure functions.
//!
//! The law: each mismatch is refused by name - both components, the
//! property, the expected value and the actual value.

use crate::component::{ComponentKind, CuttingItem, ToolComponent, ADAPTIVE_PORT};
use crate::insert_codes::{FixingHole, Hand};

/// The property a fit check found wrong.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FitProperty {
    Kind,
    Port,
    Shape,
    Clearance,
    Size,
    Thickness,
    Fixing,
    Hand,
    Coupling,
    /// An insert or seat spec that its own code would refuse.
    Code,
}

impl FitProperty {
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Kind => "kind",
            Self::Port => "port",
            Self::Shape => "shape",
            Self::Clearance => "clearance",
            Self::Size => "size",
            Self::Thickness => "thickness",
            Self::Fixing => "clamping and fixing",
            Self::Hand => "hand",
            Self::Coupling => "coupling",
            Self::Code => "designation code",
        }
    }
}

/// One refused fit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FitError {
    pub parent: String,
    pub child: String,
    pub property: FitProperty,
    pub expected: String,
    pub actual: String,
}

impl core::fmt::Display for FitError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "'{}' does not fit '{}': {} - expected {}, actual {}",
            self.child,
            self.parent,
            self.property.name(),
            self.expected,
            self.actual
        )
    }
}

impl std::error::Error for FitError {}

/// A match a library query found: the component, and the parent's
/// port the pair meets at.
#[derive(Clone, Debug, PartialEq)]
pub struct Fit<'a> {
    pub component: &'a ToolComponent,
    pub port: String,
}

fn refuse(
    parent: &ToolComponent,
    child: &ToolComponent,
    property: FitProperty,
    expected: String,
    actual: String,
) -> FitError {
    FitError { parent: parent.name().to_owned(), child: child.name().to_owned(), property, expected, actual }
}

fn kind_name(c: &ToolComponent) -> &'static str {
    match c.kind() {
        ComponentKind::CuttingItem(CuttingItem::Insert(_)) => "an insert",
        ComponentKind::CuttingItem(CuttingItem::Solid { .. }) => "a solid tool",
        ComponentKind::ToolItem { .. } => "a tool item",
        ComponentKind::AdaptiveItem { .. } => "an adaptive item",
    }
}

fn hole_name(h: FixingHole) -> &'static str {
    match h {
        FixingHole::None => "no hole",
        FixingHole::Cylindrical => "a cylindrical hole",
        FixingHole::Countersunk => "a countersunk hole",
        FixingHole::Special => "a special fixing",
    }
}

/// Does `insert` sit in the seat `port` of `holder`?
pub fn check_insert_fit(holder: &ToolComponent, port: &str, insert: &ToolComponent) -> Result<(), FitError> {
    let no = |p, e: String, a: String| Err(refuse(holder, insert, p, e, a));
    let Some(ins) = insert.insert() else {
        return no(FitProperty::Kind, "an insert".into(), kind_name(insert).into());
    };
    let Some(seat) = holder.seat(port) else {
        return if matches!(holder.kind(), ComponentKind::ToolItem { .. }) {
            no(FitProperty::Port, format!("a seat of {}", holder.ports().join(", ")), port.into())
        } else {
            no(FitProperty::Kind, "a tool item with insert seats".into(), kind_name(holder).into())
        };
    };
    // A component checks its specs when it is made; this guards the
    // rule here too.
    if let Err(e) = ins.check() {
        return no(FitProperty::Code, "a valid ISO 1832 insert".into(), e.to_string());
    }
    if let Err(e) = seat.check() {
        return no(FitProperty::Code, "a valid ISO 5608 seat".into(), e.to_string());
    }
    if seat.shape != ins.shape {
        return no(FitProperty::Shape, seat.shape.letter().into(), ins.shape.letter().into());
    }
    if seat.clearance != ins.clearance {
        return no(FitProperty::Clearance, seat.clearance.letter().into(), ins.clearance.letter().into());
    }
    if seat.size != ins.size {
        return no(FitProperty::Size, format!("{:02}", seat.size), format!("{:02}", ins.size));
    }
    if let Some(t) = seat.thickness
        && t != ins.thickness
    {
        return no(FitProperty::Thickness, t.code(), ins.thickness.code());
    }
    let takes = seat.clamping.takes();
    let has = ins.fixing.hole();
    if takes != has {
        return no(
            FitProperty::Fixing,
            format!("{} (clamping {})", hole_name(takes), seat.clamping.letter()),
            format!("{} (fixing {})", hole_name(has), ins.fixing.letter()),
        );
    }
    if let Some(h @ (Hand::R | Hand::L)) = ins.hand
        && h != seat.hand
    {
        return no(FitProperty::Hand, seat.hand.letter().into(), h.letter().into());
    }
    Ok(())
}

const COUPLING_PARENT: &str = "a component with a workpiece-side coupling";

/// Does `child`'s machine side mate with the workpiece side `port` of
/// `parent`? An insert seat is not a coupling: an insert fits by
/// [`check_insert_fit`].
pub fn check_coupling_fit(parent: &ToolComponent, port: &str, child: &ToolComponent) -> Result<(), FitError> {
    let no = |p, e: String, a: String| Err(refuse(parent, child, p, e, a));
    let csw = match parent.kind() {
        ComponentKind::AdaptiveItem { csw, .. } if port == ADAPTIVE_PORT => csw,
        ComponentKind::AdaptiveItem { .. } => return no(FitProperty::Port, ADAPTIVE_PORT.into(), port.into()),
        ComponentKind::ToolItem { .. } => {
            let actual = "a tool item (its ports are insert seats)";
            return no(FitProperty::Kind, COUPLING_PARENT.into(), actual.into());
        }
        ComponentKind::CuttingItem(_) => return no(FitProperty::Kind, COUPLING_PARENT.into(), kind_name(parent).into()),
    };
    let Some(mcs) = child.mcs() else {
        return no(FitProperty::Kind, "a component with a machine side".into(), kind_name(child).into());
    };
    if csw.takes(mcs) { Ok(()) } else { no(FitProperty::Coupling, csw.to_string(), mcs.to_string()) }
}

/// Either check, by what the child is.
pub fn check_fit(parent: &ToolComponent, port: &str, child: &ToolComponent) -> Result<(), FitError> {
    if child.insert().is_some() { check_insert_fit(parent, port, child) } else { check_coupling_fit(parent, port, child) }
}

/// The components of `library` that `child` fits in, with the port:
/// "the holders that take this insert". The child itself (by id) is
/// left out.
pub fn compatible_parents<'a>(
    child: &ToolComponent,
    library: impl IntoIterator<Item = &'a ToolComponent>,
) -> Vec<Fit<'a>> {
    let mut out = Vec::new();
    for parent in library {
        if parent.id() == child.id() {
            continue;
        }
        for port in parent.ports() {
            if check_fit(parent, port, child).is_ok() {
                out.push(Fit { component: parent, port: port.to_owned() });
            }
        }
    }
    out
}

/// The components of `library` that fit in `parent`, port by port:
/// "the inserts this holder takes". The parent itself (by id) is left
/// out.
pub fn compatible_children<'a>(
    parent: &ToolComponent,
    library: impl IntoIterator<Item = &'a ToolComponent>,
) -> Vec<Fit<'a>> {
    let library: Vec<&ToolComponent> = library.into_iter().filter(|c| c.id() != parent.id()).collect();
    let mut out = Vec::new();
    for port in parent.ports() {
        for &child in &library {
            if check_fit(parent, port, child).is_ok() {
                out.push(Fit { component: child, port: port.to_owned() });
            }
        }
    }
    out
}
