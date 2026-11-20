//! #2901: generate's seam to the first-traverse law (`verify::first_traverse_low`): which operations answer to it and which steel they read; our own reading of the shop rule "never traverse through uncut stock or steel", no vendor material consulted.

use crate::cl::{TiltedFrame, Toolpath};
use crate::fixture::{FixtureBox, MACHINE_TABLE};
use crate::verify::{self, Warning};

use super::OpParams;
use super::kinds::Section;
use super::material::Z_TOL;

/// What the first traverse must ride over: the start stock's top in the
/// path's frame, and the setup's steel in the world (`fixture::
/// named_boxes_for`: the tagged fixtures' boxes, then the machine table),
/// `names` beside `fixtures` in the same order.
pub(super) struct Over<'a> {
    pub stock_top: f64,
    pub fixtures: &'a [FixtureBox],
    pub names: &'a [String],
}

/// Whether the operation's first traverse is a move on a plane in its
/// own frame on a mill: the planar and 3-D milling kinds and the
/// vertical hole kinds, upright or 3+2. The multi-axis, rotary,
/// turning, plasma and EDM kinds read their clearance on other
/// surfaces or axes; the rapid laws judge their moves (decision #2901).
fn answers(params: &OpParams, mill: bool, rotary: bool) -> bool {
    if !mill || rotary {
        return false;
    }
    match params.kind().info().section {
        Section::Mill2D | Section::Mill3D => true,
        Section::Drilling => !matches!(params, OpParams::Drill { cross, .. } if !cross.is_empty()),
        _ => false,
    }
}

/// Add the first-traverse verdict to `warns` when `path` (in `frame`'s
/// local heights when it is tilted) starts under the stock or any steel.
///
/// A 3+2 plane reads the tagged fixtures along its tool axis (they ride
/// the table with the part), but not the machine table: its box is the
/// slab under the whole travel (`MAX_FIXTURE_DIM` wide), so a tilted
/// axis reads a far edge of it, not a top the plane can clear. Upright,
/// the table's top is under the stock and never the higher top.
pub(super) fn judge(path: &Toolpath, params: &OpParams, (mill, rotary): (bool, bool), frame: Option<&TiltedFrame>, over: &Over<'_>, warns: &mut Vec<Warning>) {
    if !answers(params, mill, rotary) {
        return;
    }
    let w = match frame {
        Some(f) => {
            let steel: Vec<FixtureBox> = over
                .fixtures
                .iter()
                .enumerate()
                .filter(|(i, _)| over.names.get(*i).is_none_or(|n| n != MACHINE_TABLE))
                .map(|(_, b)| *b)
                .collect();
            verify::first_traverse_low(path, over.stock_top, &steel, |p| f.z_local(p), Z_TOL)
        }
        None => verify::first_traverse_low(path, over.stock_top, over.fixtures, |p| p.z, Z_TOL),
    };
    if let Some(w) = w
        && !warns.contains(&w)
    {
        warns.push(w);
    }
}
