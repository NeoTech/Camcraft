//! The entries of a toolpath into the stock, and where each one starts
//! (law L5 of #4023).
//! Provenance: our own design record (#4024, L5 "read the posted CL on an
//! independent stock grid; classify the XY as air, a pre-drilled hole, a
//! cleared column, or material"), on the grid of `stock.rs`.

use camcraft_motion::cl::{Move, MoveKind};
use camcraft_motion::geo2d::Point2;

use super::stock::{Stock, points};

/// How the tool goes down.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryKind {
    /// A straight vertical feed.
    Plunge,
    /// A straight feed with an XY travel.
    Ramp,
    /// An arc in XY with a Z travel.
    Helix,
    /// A rapid move.
    Rapid,
}

/// Where an entry starts, read on the stock that is left at that time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntryStart {
    /// No stock under the tool at the start.
    Air,
    /// A pre-drilled hole under the tool.
    Hole,
    /// Stock that earlier moves cleared to below the entry depth.
    Column,
    /// The tool goes down into material.
    Solid,
}

/// One descent below the stock top.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    /// The index of the first move of the descent.
    pub at_move: usize,
    pub kind: EntryKind,
    pub start: EntryStart,
    /// The XY where the descent ends (or first meets material).
    pub place: Point2,
}

pub(super) fn kind_of(m: &Move) -> EntryKind {
    if m.kind == MoveKind::Rapid {
        EntryKind::Rapid
    } else if m.arc.is_some() {
        EntryKind::Helix
    } else if (m.to.x - m.from.x).hypot(m.to.y - m.from.y) > 1e-6 {
        EntryKind::Ramp
    } else {
        EntryKind::Plunge
    }
}

/// True if `m` goes down and ends below the stock top.
pub(super) fn descends(m: &Move, top: f64) -> bool {
    m.to.z < m.from.z - 1e-9 && m.to.z < top - super::stock::Z_EPS
}

/// The open entry of a walk: one run of descending moves.
pub(super) struct Open {
    pub entry: Entry,
}

impl Open {
    /// Read one descending move before the stock is cut by it. Material
    /// under any of its points makes the entry an entry into solid.
    pub fn read(open: Option<Open>, i: usize, m: &Move, stock: &Stock) -> Open {
        let mut open = open.unwrap_or(Open {
            entry: Entry {
                at_move: i,
                kind: kind_of(m),
                start: EntryStart::Air,
                place: Point2::new(m.to.x, m.to.y),
            },
        });
        if open.entry.start == EntryStart::Solid {
            return open;
        }
        let hit = points(m, stock.grid.cell)
            .into_iter()
            .skip(1)
            .find(|p| stock.material(Point2::new(p.x, p.y), p.z));
        match hit {
            Some(p) => {
                open.entry.kind = kind_of(m);
                open.entry.start = EntryStart::Solid;
                open.entry.place = Point2::new(p.x, p.y);
            }
            None => {
                open.entry.kind = kind_of(m);
                open.entry.place = Point2::new(m.to.x, m.to.y);
            }
        }
        open
    }

    /// Close the entry: a descent that met no material starts in a hole,
    /// in a cleared column or in air.
    pub fn close(mut self, stock: &Stock) -> Entry {
        if self.entry.start != EntryStart::Solid {
            let p = self.entry.place;
            self.entry.start = if stock.in_hole(p) {
                EntryStart::Hole
            } else if stock.was_stock(p) {
                EntryStart::Column
            } else {
                EntryStart::Air
            };
        }
        self.entry
    }
}
