//! Swept-tool subtraction on the brick SDF field (M7.1 B): the
//! max-law layer, the closed-form tool solids, and the tube walk.
//! Split from `field.rs` by the 400-line law; the storage and the
//! sign live there, the construction in `field_gen.rs`, the solid's
//! own geometry in `field_solid.rs`, the ray-entry companion layer
//! (M7.1 C) in `field_ray.rs`.
//!
//! Provenance: SDF boolean subtraction (the distance-function CSG of
//! the published implicit-surface literature: stock minus tool is
//! `max(phi_stock, -phi_tool)`), the tool solids of RS-274 world
//! machining (capsule, cylinder, truncated cone - the same three the
//! dexel atoms sweep; the bull nose's rounded rectangle, M13.1,
//! specced in `field_bull`), and the chord reduction of this codebase's own
//! `arc_chords`. Nothing from vendor binaries.
//!
//! ## The laws here
//!
//! - **Subtraction is a running max, so it commutes.** One layer of
//!   cut values rides the bricks: `phi_stock(p) = max(phi_mesh(p),
//!   cut(p))`, `cut` = the max over all moves of the negated tool
//!   distance of the move that reached deepest here. Max is
//!   associative and commutative, so a replay in any order answers
//!   identically - the ROADMAP's "sequential cuts accumulate as a
//!   running max, order-independent by construction", made literal.
//!   (The exact engine this mirrors, the dexel, looks order-dependent
//!   only because a column stores one Z; its removal is a min over
//!   bottoms, which commutes the same way.)
//! - **The tool solid is read exactly, never voxelized.** A move
//!   sweeps the solid along a segment; the solid of every instant is
//!   convex and the sweep is a Minkowski sum, so the distance at a
//!   query point is a 1-D CONVEX minimum over the move parameter -
//!   golden section to far below a cell. The ball closes further: its
//!   sweep is a capsule with a closed form. This is the ROADMAP's
//!   inverted-trajectory law: "does the solid, moved BACKWARD along
//!   this segment, contain p", with the containment made exact.
//! - **The conservative direction is the same one everywhere.** The
//!   numerical minimum of a convex function is an upper bound of the
//!   true distance, so the cut never over-claims removal (ghost rest,
//!   the blessed over-report, never a false clean); and the max
//!   layering keeps the field's sign promise: material answers as
//!   material unless the tool demonstrably reached it, whatever the
//!   cell size or band width.
//! - **Tool conventions mirror the dexel atoms value for value.**
//!   Moves carry TIP positions: a ball's center path is the tip path
//!   lifted by `r` along the tool axis (the +Z lift when the move
//!   carries no pose stamp - M5.1's `axis` field IS the inverted
//!   trajectory input); a cone's apex is its tip (virtual sharp); an
//!   upright mill tool's XY arc sweeps the true circle (#413,
//!   `field_arc`, the column engine's #402 law), while a tilted pose
//!   or a lathe tool reduces it to chords with Z interpolated
//!   uniformly (the helix law); and rapids touch nothing (RS-274 G0).
//!   The flat endmill's flank is finite and explicit (`flank`): a
//!   cylinder, not the dexel's infinite column - above the flank the
//!   field shows rest where the column law shows air, and the
//!   reconciliation at declared tolerance (stage C) passes a flank
//!   that spans the stock to agree.
//! - **Total input.** A non-finite move endpoint, a non-positive or
//!   non-finite tool parameter, a nonsense axis, or a move whose
//!   swept window - clipped to the region a cut can matter in (M7.3,
//!   `field_batch`) - would span more than `MAX_CELLS_PER_MOVE` cells
//!   beyond the window of the tool at rest at the chord's start (the
//!   same clipping) refuses by name before a single brick is written.
//!   The ceiling bounds what segmenting can bound: a wide tool's own
//!   footprint can pass the ceiling, and a short move of it cuts (the
//!   column arm cuts it too); a long chord still refuses.
//! - **The simulator segments, not its callers (#4179).**
//!   `subtract_batch_fitted` cuts a chord (or an arc piece) whose
//!   travel exceeds the ceiling as the fewest equal pieces that each
//!   fall under it, and the field arms of `Stock` cut through it. The pieces
//!   sweep the same solid as the whole: a tool swept along the parts
//!   of a path is its sweep along the path. `subtract_batch` keeps the
//!   refusal as the bound of one update.

use super::field::SdfField;
use super::field_arc::arc_sweeps;
use super::field_blade::LatheProfile;
use super::field_lathe::{LatheAxis, LatheSweep};
use super::field_solid::{add, scale, tool_kind, RevKind, ToolKind};
use super::kernel::StockError;
pub(crate) use super::field_sweep::Sweep;

use crate::cl::{Move, MoveKind};
use crate::geo2d::Point2;
use crate::geometry::Point3;

/// Refusal ceiling for the cells one chord's travel adds to the
/// tool's window at rest (the seed law's `MAX_CELLS_PER_FACET` in the
/// cutter's clothing).
const MAX_CELLS_PER_MOVE: u64 = 1 << 22;

/// The tool solids of the field's vocabulary - the same ones the
/// dexel sweeps (`Dexel::subtract`, `subtract_ball`, `subtract_cone`,
/// `subtract_bull`).
#[derive(Clone, Copy, Debug)]
pub enum SdfTool {
    /// Ball nose of radius `r`; moves carry tip positions and the
    /// center rides `r` above along the axis. The bare sphere.
    Ball { r: f64 },
    /// Ball nose on a shank of its own radius, `shank` long above the
    /// center (M7.3): the capsule the column's ball atom implies -
    /// it removes everything above the ball's bottom - made finite.
    Shanked { r: f64, shank: f64 },
    /// Flat endmill: disc of `r`, flank of `flank` along the axis.
    Flat { r: f64, flank: f64 },
    /// Taper: cone of maximum cutting radius `r`, apex at the tip
    /// (virtual sharp), side slope `cot` = cotangent of the half
    /// angle from the axis, as the dexel's cone atom reads it.
    Taper { r: f64, cot: f64 },
    /// Twist drill (#2441): cone of maximum radius `r` and side slope
    /// `cot` from the apex up to `r * cot`, transitioning into a
    /// cylindrical flute of radius `r` up to `flank` (`flank >= r * cot`).
    Drill { r: f64, cot: f64, flank: f64 },
    /// Bull nose (M13.1): a cylinder of `r` to `flank` whose bottom
    /// edge is rounded by `corner` - the rounded-rectangle profile
    /// revolved (`field_bull`), the dexel's `cut_bull` atom made a
    /// solid. `0 < corner <= r`, `flank >= corner`.
    BullNose { r: f64, corner: f64, flank: f64 },
    /// Dovetail head (#1444): a frustum widest at the tip - radius `r`
    /// there, falling by `cot` per mm of height up to `flank`, where it
    /// must still be over 0. The neck above it is its own solid
    /// (`field_undercut`).
    Dovetail { r: f64, cot: f64, flank: f64 },
    /// Turning insert (M13.1 owed, the annular-sector sweep): moves
    /// are the nose CENTER's path (M5.2 E) in the mill world of a
    /// lathe whose spindle is `axis` (`LatheAxis::Z` for +Z through
    /// the origin; the document's fitted axis otherwise, #19); the
    /// part spins a full revolution at every instant, so a move
    /// removes the solid of revolution of the half-plane stadium of
    /// radius `nose` about its (radius, station) chord. A pose stamp
    /// means nothing to a spinning part and is ignored. A blade (#21)
    /// sweeps its own footprint instead of the nose disc
    /// ([`LatheProfile`], `field_blade`).
    Lathe { profile: LatheProfile, axis: LatheAxis },
}

impl SdfField {
    /// Subtract one CL move: the tool rides the move's path (its
    /// chords, arcs reduced under the uniform-Z helix law) and the
    /// cut layer takes the running max. Rapids touch nothing; invalid
    /// input refuses before any write.
    pub fn subtract(
        &mut self,
        m: &Move,
        tool: &SdfTool,
        chord_tol: f64,
    ) -> Result<(), StockError> {
        self.subtract_batch(std::slice::from_ref(m), tool, chord_tol)
    }

    /// Subtract a batch of moves cut by one tool - the same answer as
    /// one `subtract` per move, bit for bit (both layers commute), at
    /// one visit per dirty brick for the whole batch (M7.3's cost law,
    /// `field_batch`). Every move is validated and every chord's
    /// window proved payable BEFORE any write, so a refusal leaves the
    /// field exactly as it stood.
    pub fn subtract_batch(
        &mut self,
        moves: &[Move],
        tool: &SdfTool,
        chord_tol: f64,
    ) -> Result<(), StockError> {
        self.subtract_moves(moves, tool, chord_tol, false)
    }

    /// `subtract_batch`, with every chord or arc piece whose travel
    /// exceeds the ceiling cut as the fewest equal pieces that each fall
    /// under it (#4179, the module law): the segmenting the refusal names,
    /// applied once here. A piece no longer than one cell that still
    /// exceeds the ceiling (a footprint too large for one update)
    /// refuses by name, before any write.
    pub fn subtract_batch_fitted(
        &mut self,
        moves: &[Move],
        tool: &SdfTool,
        chord_tol: f64,
    ) -> Result<(), StockError> {
        self.subtract_moves(moves, tool, chord_tol, true)
    }

    fn subtract_moves(&mut self, moves: &[Move], tool: &SdfTool, chord_tol: f64, fit: bool) -> Result<(), StockError> {
        let kind = tool_kind(tool)?;
        if !chord_tol.is_finite() || chord_tol < 0.0 {
            return Err(StockError::Invalid(format!(
                "field subtract: chord tolerance {chord_tol} is not a finite non-negative number"
            )));
        }
        let mut sweeps: Vec<([f64; 3], Sweep)> = Vec::new();
        // #637 E1: a solid that can matter and does not span the world
        // above its tip suspends the floor read's monotone law.
        let z_hi = super::march::world(self).0;
        let mut bounded = false;
        for m in moves {
            if !m.is_finite() {
                return Err(StockError::Invalid(
                    "field subtract: the move has a non-finite endpoint".into(),
                ));
            }
            if m.kind == MoveKind::Rapid {
                continue; // rapids touch nothing (RS-274 G0, the dexel's law)
            }
            let ax = match m.axis {
                None => [0.0, 0.0, 1.0],
                Some(a) => {
                    let nn = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
                    if !a[0].is_finite() || !a[1].is_finite() || !a[2].is_finite() || nn == 0.0 {
                        return Err(StockError::Invalid(format!(
                            "field subtract: axis {:?} is not a finite nonzero direction",
                            a
                        )));
                    }
                    [a[0] / nn, a[1] / nn, a[2] / nn]
                }
            };
            // An upright mill tool sweeps an XY arc on the true circle
            // (#413, `field_arc`); everything else rides its chords.
            let arc = match m.arc {
                Some(arc) if arc.plane.is_xy() && ax == [0.0, 0.0, 1.0] => arc_sweeps(m, arc, &kind, chord_tol),
                _ => None,
            };
            // A chord keeps its ends, so that it can be cut in pieces.
            let built: Vec<(Sweep, Option<(Point3, Point3)>)> = match arc {
                Some(v) => v.into_iter().map(|sw| (sw, None)).collect(),
                None => chords(m, chord_tol)
                    .into_iter()
                    .map(|(a, b)| chord_sweep(&kind, ax, a, b).map(|sw| (sw, Some((a, b)))))
                    .collect::<Result<Vec<_>, _>>()?,
            };
            for (sw, ends) in built {
                let fitted = if self.travel_work(&sw, ax) <= MAX_CELLS_PER_MOVE {
                    vec![sw]
                } else if fit {
                    self.fit(&kind, ax, &sw, ends)?
                } else {
                    return Err(StockError::Invalid(format!(
                        "field subtract: a chord's swept window spans more than {MAX_CELLS_PER_MOVE} cells beyond the tool at rest at cell size {} - the move is too long for one field update; segment it",
                        self.cell
                    )));
                };
                for sw in fitted {
                    if self.clipped(sw.window(self.cell, self.cap, ax)).is_some() && !sw.spans_above(ax, z_hi) {
                        bounded = true;
                    }
                    sweeps.push((ax, sw));
                }
            }
        }
        // Set only past every gate: a refusal leaves the field as it stood.
        self.bounded_cuts |= bounded;
        self.cut_batch(&sweeps);
        Ok(())
    }
}

impl SdfField {
    /// The WORK a sweep's travel adds, which the ceiling bounds: the
    /// window as clipped to the cut region (the only cells the walk can
    /// visit), beyond the tool's own window at rest. Segmenting shrinks
    /// the first, never the second.
    fn travel_work(&self, sw: &Sweep, ax: [f64; 3]) -> u64 {
        let window = self.clipped(sw.window(self.cell, self.cap, ax));
        let rest = sw.rest_window(self.cell, self.cap, ax).and_then(|w| self.clipped(w));
        window.map_or(0, window_cells).saturating_sub(rest.map_or(0, window_cells))
    }

    /// `sw` (a chord from `ends`, or an arc piece) as the fewest equal
    /// pieces whose travel each falls under the ceiling (#4179). A piece's
    /// work falls as it shortens, so doubling finds a count that falls under
    /// and bisection the fewest below it. Pieces shorter than a cell
    /// shrink no window further: past that count it refuses. A sweep
    /// that is neither a chord nor an arc piece has no travel to split,
    /// so it refuses by name rather than be cut as nothing.
    fn fit(&self, kind: &ToolKind, ax: [f64; 3], sw: &Sweep, ends: Option<(Point3, Point3)>) -> Result<Vec<Sweep>, StockError> {
        let unsplittable = || {
            StockError::Invalid(
                "field subtract: a sweep past the ceiling is neither a chord nor an arc piece, so it cannot be split - segment it".into(),
            )
        };
        let travel = match (sw, ends) {
            (_, Some((a, b))) => (b.x - a.x).hypot(b.y - a.y).hypot(b.z - a.z),
            (Sweep::Arc(p), None) => p.travel(),
            _ => return Err(unsplittable()),
        };
        let most = (travel / self.cell).ceil().clamp(1.0, f64::from(u32::MAX)) as usize;
        let pieces = |k: usize| -> Result<Vec<Sweep>, StockError> {
            match (sw, ends) {
                (_, Some((a, b))) => {
                    let at = |j: usize| {
                        let t = j as f64 / k as f64;
                        if j == k { b } else { Point3::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t) }
                    };
                    (0..k).map(|j| chord_sweep(kind, ax, at(j), at(j + 1))).collect()
                }
                (Sweep::Arc(p), None) => Ok(p.split(k).into_iter().map(Sweep::Arc).collect()),
                _ => Err(unsplittable()),
            }
        };
        let under = |v: &[Sweep]| v.iter().all(|s| self.travel_work(s, ax) <= MAX_CELLS_PER_MOVE);
        let (mut lo, mut hi) = (1usize, 2usize);
        let mut best = loop {
            let v = pieces(hi.min(most))?;
            if under(&v) {
                hi = hi.min(most);
                break v;
            }
            if hi >= most {
                return Err(StockError::Invalid(format!(
                    "field subtract: a chord's swept window spans more than {MAX_CELLS_PER_MOVE} cells beyond the tool at rest at cell size {} even in pieces of one cell - the tool's footprint is too large for one field update",
                    self.cell
                )));
            }
            lo = hi;
            hi = hi.saturating_mul(2);
        };
        while hi - lo > 1 {
            let mid = lo + (hi - lo) / 2;
            let v = pieces(mid)?;
            if under(&v) {
                hi = mid;
                best = v;
            } else {
                lo = mid;
            }
        }
        Ok(best)
    }
}

/// One straight chord's swept solid for `kind`.
pub(super) fn chord_sweep(kind: &ToolKind, ax: [f64; 3], a: Point3, b: Point3) -> Result<Sweep, StockError> {
    Ok(match kind {
        ToolKind::Ball(r) => {
            let off = scale(ax, *r);
            Sweep::Ball { c0: add(a, off), c1: add(b, off), r: *r }
        }
        ToolKind::Shanked { r, shank } => {
            let off = scale(ax, *r);
            Sweep::Shanked { c0: add(a, off), c1: add(b, off), up: scale(ax, *shank), r: *r }
        }
        ToolKind::Flat { r, flank } => Sweep::Revolve { a, b, kind: RevKind::Flat { r: *r, flank: *flank } },
        ToolKind::Taper { r, cot } => Sweep::Revolve { a, b, kind: RevKind::Taper { r: *r, cot: *cot } },
        ToolKind::Drill { r, cot, flank } => {
            Sweep::Revolve { a, b, kind: RevKind::Drill { r: *r, cot: *cot, flank: *flank } }
        }
        ToolKind::Bull { r, corner, flank } => {
            Sweep::Revolve { a, b, kind: RevKind::Bull { r: *r, corner: *corner, flank: *flank } }
        }
        ToolKind::Dove { r, cot, flank } => {
            Sweep::Revolve { a, b, kind: RevKind::Dove { r: *r, cot: *cot, flank: *flank } }
        }
        ToolKind::Lathe { foot, axis } => Sweep::Lathe(LatheSweep::of(*axis, a, b, foot)?),
    })
}

/// Cells in a chord's window: saturating, so an absurd dilation
/// answers absurd (and refuses), never an overflowed count.
fn window_cells(w: [(i32, i32); 3]) -> u64 {
    let cnt = |s: (i32, i32)| (s.1 as i64 - s.0 as i64 + 1).max(0) as u64;
    cnt(w[0])
        .saturating_mul(cnt(w[1]))
        .saturating_mul(cnt(w[2]))
}

/// The move's path as straight chords: one segment for a linear move,
/// the sagitta chords with Z interpolated uniformly for an arc the
/// exact sweep does not take (a tilted pose, a lathe tool).
pub(super) fn chords(m: &Move, chord_tol: f64) -> Vec<(Point3, Point3)> {
    match m.arc {
        None => vec![(m.from, m.to)],
        // XZ / YZ arcs (M11.2 A): their own 3-D chords.
        Some(arc) if !arc.plane.is_xy() => m.segments3(chord_tol),
        Some(arc) => {
            let a2 = Point2::new(m.from.x, m.from.y);
            let b2 = Point2::new(m.to.x, m.to.y);
            let cs = crate::cl::arc_chords(a2, b2, arc, chord_tol);
            let n = cs.len().max(1) as f64;
            let dz = m.to.z - m.from.z;
            cs.into_iter()
                .enumerate()
                .map(|(j, (ca, cb))| {
                    (
                        Point3::new(ca.x, ca.y, m.from.z + dz * (j as f64 / n)),
                        Point3::new(cb.x, cb.y, m.from.z + dz * ((j + 1) as f64 / n)),
                    )
                })
                .collect()
        }
    }
}
