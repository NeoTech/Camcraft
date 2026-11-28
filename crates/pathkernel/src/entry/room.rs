//! The room of the entry and exit stage, read from the one region (K1):
//! the legal room of the tool centre at the depth of the cut, the air zone
//! where a descent stays clear of the stock by the approach distance, and
//! the context of one entry.
//! Provenance: our own design record (`RECOMMENDED.md`, `## #4024`)
//! "Stage 6: Entry and exit" and `docs/decisions/4025-kernel-region.md`
//! (the air band, "Yes, to the edge", a clamp is a keep-out); the
//! cutter-centre offsets as in Held (1991).

use super::proof::Area;
use super::stage::{ENTRY_GEOMETRY, ENTRY_GEOMETRY_INVALID, EntryStage, NOT_PLANAR};
use super::EntryChoice;
use crate::frame::{PlanePoint, ToolShape};
use crate::plan::Cut;
use crate::region::{KernelRegion, PlanarRegion};
use crate::verdict::{Note, Refusal, Stage};
use crate::{KERNEL_TOL, Point2, Region};

/// The tolerance (mm) of the room tests: the arc tolerance of the polygon
/// offset of `camcraft-geometry` (5 um). A cut on the boundary of the
/// cutter-centre area is in the room within it.
pub const ROOM_TOL: f64 = 0.005;

pub(super) fn geo<T>(r: Result<T, camcraft_geometry::geo2d::GeoError>) -> Result<T, Refusal> {
    r.map_err(|e| crate::geo_refusal(Stage::Entry, &e, ENTRY_GEOMETRY, ENTRY_GEOMETRY_INVALID))
}

pub(super) fn planar(r: &KernelRegion) -> Result<&PlanarRegion, Refusal> {
    r.planar().ok_or_else(|| Refusal::new(Stage::Entry, NOT_PLANAR))
}

/// The unit vector of `d`, if it has a length.
pub(super) fn unit(d: Point2) -> Option<Point2> {
    let len = d.x.hypot(d.y);
    (len.is_finite() && len > KERNEL_TOL).then(|| Point2::new(d.x / len, d.y / len))
}

/// The points that the tool centre walks along `cut`: a closed curve comes
/// back to its first point.
pub(super) fn path_of(cut: &Cut) -> Vec<Point2> {
    let mut pts: Vec<Point2> = cut.curve().pts().iter().map(PlanePoint::at).collect();
    if cut.curve().closed()
        && let Some(&first) = pts.first()
    {
        pts.push(first);
    }
    pts
}

/// The disc of `radius` about `c` as a proved area.
pub(super) fn disc(c: Point2, radius: f64) -> Result<Area, Refusal> {
    Area::new(geo(Region::sweep(&[vec![c]], radius))?)
}

/// The band outside the stock within one tool radius of a stock edge loop:
/// there the disc overhangs the stock edge but touches no part. A loop that
/// is all on the stock outline has no part wall, so the band touches no
/// part (#4025 decision 4). It overlaps the centre and the air by
/// [`ROOM_TOL`], so the union has no seam.
fn strip(p: &PlanarRegion) -> Result<Region, Refusal> {
    let edges: Vec<Vec<Point2>> = p
        .stock_edge()
        .iter()
        .map(|l| {
            let mut v = l.points().to_vec();
            v.extend(l.points().first().copied());
            v
        })
        .collect();
    if edges.is_empty() || p.air().is_empty() {
        return Ok(Region::empty());
    }
    let reach = p.tool().radius() + ROOM_TOL;
    let band = geo(Region::sweep(&edges, reach))?;
    geo(band.intersect(&geo(p.air().offset(reach))?))
}

/// The legal room of the tool centre at the depth of the cut: the centre,
/// the air and the band at the stock edge, less the keep-out grown by the
/// tool radius, grown by [`ROOM_TOL`]. The link stage reads the same room
/// for its links outside the stock (#4241).
pub(crate) fn legal_room(p: &PlanarRegion) -> Result<Region, Refusal> {
    let mut legal = p.center().clone();
    if !p.air().is_empty() {
        legal = geo(legal.union(p.air()))?;
        let s = strip(p)?;
        if !s.is_empty() {
            legal = geo(legal.union(&s))?;
        }
    }
    if !p.keep_out().is_empty() {
        legal = geo(legal.difference(&geo(p.keep_out().offset(p.tool().radius()))?))?;
    }
    geo(legal.offset(ROOM_TOL))
}

/// The air zone of a descent: the air of the region less the places within
/// the approach distance of the stock band, and the distance used when the
/// asked one is larger. The air band of the region is one tool radius wide
/// (#4025), so the distance is limited to half the tool radius.
pub(super) fn air_zone(p: &PlanarRegion, approach: f64) -> Result<(Region, Option<f64>), Refusal> {
    let a = approach.min(0.5 * p.tool().radius()).max(0.0);
    let capped = (approach > a + KERNEL_TOL).then_some(a);
    if a <= KERNEL_TOL || p.air().is_empty() {
        return Ok((p.air().clone(), capped));
    }
    let near = geo(p.center().union(&strip(p)?))?;
    Ok((geo(p.air().difference(&geo(near.offset(a))?))?, capped))
}

/// The context of one entry: the stage, the region, the user's choice, the
/// cut start and direction, and the legal room.
pub(super) struct Ctx<'a> {
    pub stage: &'a EntryStage,
    pub region: &'a PlanarRegion,
    pub choice: &'a EntryChoice,
    pub cleared: &'a Region,
    pub cut: &'a Cut,
    pub start: PlanePoint,
    pub dir: Point2,
    pub legal: Region,
    pub legal_area: Area,
    pub notes: Vec<Note>,
    /// The tries that a checked constructor refused, with the kind and the
    /// named reason (#4407): never dropped, they go to the notes of the
    /// entry or to the final refusal.
    pub rejected: Vec<(super::EntryKind, &'static str)>,
}

/// Refusal reason: a first cut with no direction at its start.
pub const NO_DIRECTION: &str = "the first cut has no direction at its start";

impl<'a> Ctx<'a> {
    pub fn new(stage: &'a EntryStage, cut: &'a Cut, r: &'a KernelRegion, cleared: &'a Region, choice: &'a EntryChoice) -> Result<Self, Refusal> {
        let region = planar(r)?;
        let pts = cut.curve().pts();
        let start = *pts.first().ok_or_else(|| Refusal::new(Stage::Entry, NO_DIRECTION))?;
        let dir = path_of(cut)
            .iter()
            .skip(1)
            .find_map(|q| unit(Point2::new(q.x - start.at().x, q.y - start.at().y)))
            .ok_or_else(|| Refusal::new(Stage::Entry, NO_DIRECTION).at(start.at()))?;
        let legal = legal_room(region)?;
        let legal_area = Area::new(legal.clone())?;
        Ok(Self { stage, region, choice, cleared, cut, start, dir, legal, legal_area, notes: Vec::new(), rejected: Vec::new() })
    }
    pub fn tool(&self) -> ToolShape {
        self.region.tool()
    }
    pub fn r(&self) -> f64 {
        self.region.tool().radius()
    }
    /// The depth of the cut start.
    pub fn depth(&self) -> f64 {
        self.start.depth()
    }
    /// The point above `at` where the descent starts: the material top plus
    /// the clearance (never below the cut).
    pub fn above(&self, at: Point2) -> Result<PlanePoint, Refusal> {
        PlanePoint::new(at, self.stage.heights().start().max(self.depth()))
    }
    pub fn at_depth(&self, at: Point2) -> Result<PlanePoint, Refusal> {
        PlanePoint::new(at, self.depth())
    }
    /// True when no stretch of `path` leaves the legal room. A geometry
    /// error is a refusal, not a "does not fit" (#4407).
    pub fn holds(&self, path: &[Point2]) -> Result<bool, Refusal> {
        geo(self.legal.holds_path(path))
    }
    /// The straight lead at depth from `from` to the cut start, or none
    /// when `from` is the cut start; `Err` when it leaves the legal room.
    pub fn straight_lead(&self, from: Point2) -> Result<Option<super::Lead>, Refusal> {
        let s = self.start.at();
        if crate::near(from, s) {
            return Ok(None);
        }
        super::Lead::new(vec![self.at_depth(from)?, self.at_depth(s)?], &self.legal_area).map(Some)
    }
    /// Adds a note for the status line.
    pub fn note(&mut self, text: String) -> Result<(), Refusal> {
        self.notes.push(Note::new(Stage::Entry, text)?);
        Ok(())
    }
    /// The value of a checked constructor of a `kind` try, or `None` with
    /// its named reason kept in [`Ctx::rejected`] (#4407): the stage may
    /// try the next place or kind, but the reason is never dropped.
    pub fn checked<T>(&mut self, kind: super::EntryKind, made: Result<T, Refusal>) -> Option<T> {
        match made {
            Ok(v) => Some(v),
            Err(r) => {
                if !self.rejected.contains(&(kind, r.reason())) {
                    self.rejected.push((kind, r.reason()));
                }
                None
            }
        }
    }
}
