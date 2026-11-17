//! The overlay lines of each drawn toolpath, made once per input (#2201):
//! `interact::toolpath_lines` asked `path_lines::cache_lines` on every
//! frame for every drawn operation - each arc flattened to chords and
//! each feed given its chevrons again, so a still frame paid for every
//! move of every shown path. The lines of one operation are a pure
//! function of the key below; a still frame copies them.
//!
//! The key holds every input of `cache_lines`:
//! - the path: its stored signature ([`Toolpath::path_sig`], written once
//!   by `Toolpath::new`, the only constructor, over every move field on
//!   the 1 um lattice) and its move count;
//! - the verify warnings, by equality (the gouge colour and the marks);
//! - the palette, the dim factor, the chevron arm, and the viewport
//!   ground colour that `move_colour` fades a dimmed path toward.
//!
//! Entries live per operation entity; a call drops the entries of the
//! operations it did not draw, so a hidden or deleted path holds no
//! lines. Display only: the verify check, the collision checks and the
//! post gate never read this memo.
//! Provenance: our own frame-cost law (docs/decisions/2238-frame-cost-
//! rule.md); no vendor material consulted.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

use camcraft_app::model;
use camcraft_app::schema::Toolpath;
use camcraft_app::verify::Warning;
use flecs_ecs::prelude::Entity;

use crate::egui_app::colorvision::Semantic;
use crate::state::State;

/// Every input of one operation's lines.
struct Key {
    sig: u64,
    moves: usize,
    warnings: Vec<Warning>,
    pal: Semantic,
    dim: f32,
    arm: f64,
    ground: egui::Color32,
}

/// One operation's lines; `complete` is false when the vertex budget cut
/// them short (the caller then draws no further path).
#[derive(Clone)]
pub(crate) struct Lines {
    pub(crate) lines: Rc<[[f32; 6]]>,
    pub(crate) complete: bool,
}

struct Entry {
    entity: Entity,
    key: Key,
    out: Lines,
}

thread_local! {
    static MEMO: RefCell<Vec<Entry>> = const { RefCell::new(Vec::new()) };
    /// Real `cache_lines` builds made through [`lines`] (#2201 pins).
    static BUILDS: Cell<u64> = const { Cell::new(0) };
}

/// The overlay lines of `cache`, the path of operation `entity`, drawn
/// with `pal`, `dim` and `arm`: made once per key, then shared.
pub(crate) fn lines(entity: Entity, cache: &Toolpath, pal: &Semantic, dim: f32, arm: f64) -> Lines {
    lines_on(entity, cache, pal, dim, arm, crate::egui_app::theme::current().viewport)
}

/// [`lines`] with the viewport ground given: `path_lines::move_colour`
/// fades a dimmed path toward the live theme's ground, so the ground is
/// part of the key.
fn lines_on(entity: Entity, cache: &Toolpath, pal: &Semantic, dim: f32, arm: f64, ground: egui::Color32) -> Lines {
    let hit = MEMO.with(|m| {
        m.borrow().iter().find(|e| e.entity == entity && key_is(&e.key, cache, pal, dim, arm, ground)).map(|e| e.out.clone())
    });
    if let Some(out) = hit {
        return out;
    }
    BUILDS.with(|n| n.set(n.get().saturating_add(1)));
    let _span = crate::frame_cost::span("scene/overlay.toolpaths.build");
    let mut v = Vec::new();
    let complete = crate::path_lines::cache_lines(&mut v, cache, pal, dim, arm);
    let out = Lines { lines: Rc::from(v), complete };
    let key = Key {
        sig: cache.path_sig(),
        moves: cache.path.moves.len(),
        warnings: cache.warnings.clone(),
        pal: *pal,
        dim,
        arm,
        ground,
    };
    MEMO.with(|m| {
        let mut m = m.borrow_mut();
        m.retain(|e| e.entity != entity);
        m.push(Entry { entity, key, out: out.clone() });
    });
    out
}

/// The key compare without building a key (no warning clone on a hit).
fn key_is(k: &Key, cache: &Toolpath, pal: &Semantic, dim: f32, arm: f64, ground: egui::Color32) -> bool {
    k.sig == cache.path_sig()
        && k.moves == cache.path.moves.len()
        && k.pal == *pal
        && k.dim.to_bits() == dim.to_bits()
        && k.arm.to_bits() == arm.to_bits()
        && k.ground == ground
        && k.warnings == cache.warnings
}

/// The lines of every drawn path, appended to `out` in program order. The
/// selected operation draws bright; other operations draw at `others`
/// only under "Show all toolpaths"; a path hidden by its browser eye or
/// its setup's eye never draws. A path that does not draw is not read.
/// False when the vertex budget is spent (logged).
pub(crate) fn shown_paths(out: &mut Vec<[f32; 6]>, state: &State, pal: &Semantic, arm: f64, others: f32) -> bool {
    let mut drawn = Vec::new();
    let mut full = true;
    for (name, entity) in camcraft_app::setup::operation_order(&state.world) {
        // Browser-eye hidden: the cache still exists and still posts, it
        // just does not draw (display state, never a geometry change).
        if state.hidden_paths.contains(&name) || state.op_hidden_by_setup(&name) {
            continue;
        }
        let selected = model::is_selected(&state.world, &entity);
        if !selected && !state.show_all_paths {
            continue;
        }
        let dim = if selected { 1.0 } else { others };
        // #261: warned moves wear the gouge colour (crate::path_lines).
        let Some(made) = model::with_toolpath(&state.world, &entity, |c| lines(entity, c, pal, dim, arm)) else {
            continue;
        };
        drawn.push(entity);
        if !append(out, &made) {
            full = false;
            break;
        }
    }
    keep_only(&drawn);
    full
}

/// Append one operation's `lines` to the overlay `out`. False when the
/// vertex budget is spent - `out` is then cut to the budget on a whole
/// line and the cut is logged - or when the lines were already cut short:
/// the caller draws no further path, as `path_lines::cache_lines` did.
pub(crate) fn append(out: &mut Vec<[f32; 6]>, lines: &Lines) -> bool {
    use crate::interact::MAX_OVERLAY_VERTICES;
    out.extend_from_slice(&lines.lines);
    if out.len() > MAX_OVERLAY_VERTICES {
        diag_warn!(
            "overlay: the cached paths outgrew the {MAX_OVERLAY_VERTICES}-vertex \
             display budget; the rest is truncated (the full path still posts)"
        );
        out.truncate(MAX_OVERLAY_VERTICES & !1);
        return false;
    }
    lines.complete
}

/// Drop the entries of every operation not in `drawn`.
fn keep_only(drawn: &[Entity]) {
    MEMO.with(|m| m.borrow_mut().retain(|e| drawn.contains(&e.entity)));
}

/// How many real line builds [`lines`] made on this thread.
#[must_use]
pub fn path_line_builds() -> u64 {
    BUILDS.with(Cell::get)
}

/// How many operations hold lines in the memo now.
#[must_use]
pub fn path_line_entries() -> usize {
    MEMO.with(|m| m.borrow().len())
}

#[cfg(test)]
#[path = "path_line_memo_tests.rs"]
mod tests;
