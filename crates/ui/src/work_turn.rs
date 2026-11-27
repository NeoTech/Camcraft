//! The playhead's work turn as the scene draws it (#4266): one display
//! transform, shared by every work-side scene element - the models, the
//! stock, the cut-region fills, the cached toolpath lines and the
//! playback marker. Machine-side elements (the ground grid, the gizmos,
//! the tool body) do not read it.
//!
//! Membership (the review's item 1): the steel that TURNS with the work
//! is the turning fixtures and the operation's source material. A model
//! whose [`camcraft_app::fixture::FixtureDef`] declares `fixed` is judged
//! in the machine frame by the gate (`fixture::workholding_for`), so the
//! display keeps it there too: no transform.
//!
//! Provenance: our own reading of machine-tool kinematics and our own
//! gate math (`camcraft-sim::machine_view`); no vendor material consulted.

use crate::state::State;

/// One display transform: the turn about the rotary line, as the scene
/// applies it. `bits` is the key the frame-cost caches read (crates/ui/
/// AGENTS.md "Frame cost"): every input's value, no document counter.
#[derive(Clone, Copy, Debug)]
pub(crate) struct TurnView {
    pub origin: [f64; 3],
    pub dir: [f64; 3],
    /// Radians about `dir` through `origin` (signed; the gate's own
    /// reading of the pose standing upright).
    pub angle: f64,
    /// `bits[0]` tags the transform's presence, `1..6` are the origin and
    /// the direction's bit patterns, `7` the angle's.
    pub bits: [u64; 8],
}

impl TurnView {
    fn key(origin: [f64; 3], dir: [f64; 3], angle: f64) -> Self {
        let mut bits = [0u64; 8];
        bits[0] = 1;
        for (i, v) in origin.into_iter().chain(dir).enumerate() {
            bits[i + 1] = v.to_bits();
        }
        bits[7] = angle.to_bits();
        TurnView { origin, dir, angle, bits }
    }
}

/// The bit pattern of "no turn": distinct from every `TurnView` key.
pub(crate) const NO_TURN: [u64; 8] = [0; 8];

/// The playhead's work turn, when the transport is up on this document's
/// timeline and the pose will stand upright under it: the same read
/// every work-side element makes, so the whole display frame shares one
/// transform. `None` is the fallback draw (no turn at all).
#[must_use]
pub(crate) fn play_view(state: &State) -> Option<TurnView> {
    let sim = state.sim.as_ref()?;
    let (version, sc) = state.sim_script_cache.as_ref()?;
    if *version != state.doc_version || sc.is_empty() {
        return None;
    }
    let f = sc.at(sim.t);
    let w = f.work.as_ref()?;
    let t = camcraft_app::sim_machine_view::frame_turn(w, f.axis)?;
    Some(TurnView::key(w.origin, w.dir, t))
}

/// Whether this model's steel turns with the work: not a fixture, or a
/// fixture whose `fixed` flag is false (the default - a chuck). A fixture
/// declared fixed (a table, a tailstock) stays in the machine frame, the
/// same split the gate reads (`camcraft_app::fixture::workholding_for`).
#[must_use]
pub(crate) fn turns_with_work(state: &State, entity: &flecs_ecs::prelude::Entity) -> bool {
    !camcraft_app::fixture::def_of(&state.world, entity).is_some_and(|d| d.fixed)
}

/// The work-side vertices turned in place. `view: None` (no playhead
/// turn, or fixed steel) leaves them as they are.
pub(crate) fn turn_verts(verts: &mut [[f32; 3]], view: Option<&TurnView>) {
    if let Some(v) = view {
        camcraft_app::sim_machine_view::turn_verts(verts, v.origin, v.dir, v.angle);
    }
}

/// The work-side overlay lines turned in place (two `[x, y, z, r, g, b]`
/// vertices per line; the colours ride along untouched).
pub(crate) fn turn_lines(lines: &mut [[f32; 6]], view: Option<&TurnView>) {
    let Some(v) = view else { return };
    let mut pts: Vec<[f32; 3]> = lines.iter().map(|l| [l[0], l[1], l[2]]).collect();
    turn_verts(&mut pts, Some(v));
    for (l, p) in lines.iter_mut().zip(pts) {
        [l[0], l[1], l[2]] = p;
    }
}

#[cfg(test)]
#[path = "work_turn_tests.rs"]
mod tests;
