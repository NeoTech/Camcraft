//! Law measures on a finished toolpath (#4154, K7a of the path kernel, #4027).
//! Provenance: the "laws as checks" table of our own design record (#4024);
//! the radial engagement is the contact angle of the cutter rim with the
//! stock (Altintas, "Manufacturing Automation", 2000, ch. 2), read on a pixel
//! stock model as in the pixel-based engagement methods (Choy and Chan,
//! Computer-Aided Design 35, 2003). No vendor material consulted.
//!
//! Each reading is a count with the list of its places. The measures read
//! only the toolpath, the stock, the part and the reach of the tool centre:
//! they run the same on an old planner and on the new kernel.

mod engage;
mod entries;
mod feeds;
mod lifts;
mod notes;
mod stock;

use camcraft_motion::cl::Toolpath;
use camcraft_motion::geo2d::{Point2, Region};

pub use entries::{Entry, EntryKind, EntryStart};
pub use notes::is_fallback;

/// What the measures need beside the toolpath.
#[derive(Clone)]
pub struct Spec {
    /// The radius of the tool (a flat disc).
    pub tool_r: f64,
    /// The stock outline in XY, as material from `bottom` up to `top`.
    pub stock: Region,
    /// The top of the stock.
    pub top: f64,
    /// The part in XY (never to cut), if any.
    pub part: Option<Region>,
    /// Pre-drilled holes (no material), if any.
    pub holes: Option<Region>,
    /// The area that the tool centre may use on a planar feed, if known.
    pub reach: Option<Region>,
    /// The planned engagement limit in degrees.
    pub limit_deg: f64,
    /// The entry kind that the operation asks for, if it has a choice.
    pub asked_entry: Option<EntryKind>,
    /// The size of a stock grid cell.
    pub cell: f64,
}

impl Spec {
    /// A spec with the usual cell (a quarter of a millimetre).
    pub fn new(tool_r: f64, stock: Region, top: f64, limit_deg: f64) -> Self {
        Spec {
            tool_r,
            stock,
            top,
            part: None,
            holes: None,
            reach: None,
            limit_deg,
            asked_entry: None,
            cell: 0.25,
        }
    }
}

/// One reading: a count and the places where it occurs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Reading {
    pub count: usize,
    pub places: Vec<Point2>,
}

impl Reading {
    pub(crate) fn add(&mut self, at: Point2) {
        self.count = self.count.saturating_add(1);
        self.places.push(at);
    }
}

/// All the readings of one toolpath.
#[derive(Clone, Debug, Default)]
pub struct Readings {
    /// Z-up moves that leave a level while a cut of that level is to come.
    pub lifts: Reading,
    /// The lifts that the link notes name (the sum of their counts).
    pub named_lifts: usize,
    /// Returns to a level after its first cut run.
    pub re_entries: usize,
    /// Pairs of planar feeds that cross (the verifier's Crossing law).
    pub crossings: Reading,
    /// Planar feeds whose centre leaves the reach.
    pub leaves: Reading,
    /// Each descent below the stock top.
    pub entries: Vec<Entry>,
    /// Feed moves with an engagement above the limit.
    pub over: Reading,
    /// The worst engagement on a planar feed, in degrees.
    pub worst_deg: f64,
    /// Feed moves with a full-width cut (180 degrees).
    pub full_width: Reading,
    /// The notes that say that a different plan was used.
    pub fallbacks: Vec<String>,
    /// The entry kind that the operation asked for (from the spec).
    pub asked_entry: Option<EntryKind>,
}

/// Measure `path` against `spec`.
pub fn measure(path: &Toolpath, spec: &Spec) -> Result<Readings, String> {
    if !(spec.tool_r.is_finite() && spec.tool_r > 0.0) {
        return Err(format!("tool radius must be positive, got {}", spec.tool_r));
    }
    if !spec.top.is_finite() || !spec.limit_deg.is_finite() {
        return Err("stock top and limit must be finite".into());
    }
    let mut out = Readings::default();
    let (lifts, re_entries) = lifts::level_lifts(path, spec.top);
    out.lifts = lifts;
    out.re_entries = re_entries;
    out.named_lifts = notes::named_lifts(path);
    out.crossings = feeds::crossings(path);
    if let Some(reach) = &spec.reach {
        out.leaves = feeds::leaves(path, reach, spec.cell)?;
    }
    let mut grid = stock::Stock::new(spec)?;
    let walk = engage::walk(path, spec, &mut grid);
    out.entries = walk.entries;
    out.over = walk.over;
    out.worst_deg = walk.worst_deg;
    out.full_width = walk.full_width;
    out.asked_entry = spec.asked_entry;
    out.fallbacks = path.notes.iter().filter(|n| is_fallback(n)).cloned().collect();
    Ok(out)
}

impl Readings {
    /// The count of entries that start at `at`.
    pub fn entries_at(&self, at: EntryStart) -> usize {
        self.entries.iter().filter(|e| e.start == at).count()
    }

    /// The count of entries into solid material with the kind `kind`.
    pub fn solid(&self, kind: EntryKind) -> usize {
        self.entries
            .iter()
            .filter(|e| e.start == EntryStart::Solid && e.kind == kind)
            .count()
    }

    /// The lifts that no note names.
    pub fn unnamed_lifts(&self) -> usize {
        self.lifts.count.saturating_sub(self.named_lifts)
    }

    /// One line that a test pins: each count in a fixed order.
    pub fn row(&self) -> String {
        format!(
            "lifts {} named {} re {} | cross {} leave {} | entry air {} hole {} col {} solid {} \
             (plunge {} ramp {} helix {} rapid {}) | over {} worst {:.0} full {} | fallback {}",
            self.lifts.count,
            self.named_lifts,
            self.re_entries,
            self.crossings.count,
            self.leaves.count,
            self.entries_at(EntryStart::Air),
            self.entries_at(EntryStart::Hole),
            self.entries_at(EntryStart::Column),
            self.entries_at(EntryStart::Solid),
            self.solid(EntryKind::Plunge),
            self.solid(EntryKind::Ramp),
            self.solid(EntryKind::Helix),
            self.solid(EntryKind::Rapid),
            self.over.count,
            self.worst_deg,
            self.full_width.count,
            self.fallbacks.len(),
        )
    }

    /// The entries into solid material of another kind than the one that
    /// the operation asked for.
    pub fn substituted(&self) -> usize {
        let Some(asked) = self.asked_entry else {
            return 0;
        };
        self.entries
            .iter()
            .filter(|e| e.start == EntryStart::Solid && e.kind != asked)
            .count()
    }

    /// The laws that these readings break, by the checks of the design
    /// record (#4024):
    /// - link: a lift with no named reason, a crossing, or a feed out of
    ///   the reach;
    /// - L4: an entry of another kind than the one asked, with no note;
    /// - L5: a straight plunge or a rapid into solid material;
    /// - L6: a cut above the limit, or a full-width cut outside a named
    ///   slot. `slot` says that the kind is a named slot, where a
    ///   full-width cut is the plan.
    pub fn breaks(&self, slot: bool) -> String {
        let mut out = Vec::new();
        if self.unnamed_lifts() > 0 || self.crossings.count > 0 || self.leaves.count > 0 {
            out.push("link");
        }
        if self.substituted() > 0 && self.fallbacks.is_empty() {
            out.push("L4");
        }
        if self.solid(EntryKind::Plunge) > 0 || self.solid(EntryKind::Rapid) > 0 {
            out.push("L5");
        }
        let planned_full = if slot { self.full_width.count } else { 0 };
        if self.over.count > planned_full || (!slot && self.full_width.count > 0) {
            out.push("L6");
        }
        if out.is_empty() {
            "-".into()
        } else {
            out.join(" ")
        }
    }
}
