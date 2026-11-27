//! #2181 / #2192 review: the structural twin of the transport row for
//! the stock removal calculated before playback. The row is drawn and
//! read back through AccessKit: the progress text and a Cancel that
//! works while the calculation runs, the "incomplete" text without
//! Cancel after it stopped, and no row when it is done or not asked.
//! The shell case draws the row from a live playback worker. #3315: a
//! hidden Stock row is said under the transport.
//! Provenance: our own tests over our own transport row; no HSMWorks
//! material involved.

use egui_kittest::kittest::NodeT;

use super::*;

use camcraft_app::stock_play::Precalc;

use crate::play_precalc::{precalc_row, CANCEL};
use crate::state::Sim;

/// The row alone, over a fixed state; the count is Cancel's clicks.
fn row(p: Precalc) -> Harness<'static, (Precalc, usize)> {
    Harness::builder().with_size(Vec2::new(900.0, 60.0)).build_ui_state(
        |ui, st: &mut (Precalc, usize)| {
            if precalc_row(ui, &st.0) {
                st.1 += 1;
            }
        },
        (p, 0),
    )
}

#[test]
fn the_running_row_shows_progress_and_its_cancel_works() {
    let mut h = row(Precalc::Running { done: 50, total: 200 });
    h.run();
    assert!(
        h.query_by_label_contains("Calculating the stock removal: 25%").is_some(),
        "the row says the progress"
    );
    let cancel = h.get_by_role_and_label(egui::accesskit::Role::Button, CANCEL);
    assert!(!cancel.accesskit_node().is_disabled(), "Cancel is enabled while the calculation runs");
    cancel.click();
    h.run();
    assert_eq!(h.state().1, 1, "one click on Cancel is one cancel");
}

#[test]
fn the_incomplete_row_says_so_and_has_no_cancel() {
    let mut h = row(Precalc::Incomplete { done: 100, total: 200, why: "cancelled".into() });
    h.run();
    assert!(
        h.query_by_label_contains("Stock removal incomplete (cancelled at 50%)").is_some(),
        "the row says the calculation is incomplete, and why"
    );
    assert!(h.query_by_label(CANCEL).is_none(), "nothing runs: nothing to cancel");
}

#[test]
fn no_row_when_the_calculation_is_done_or_not_asked() {
    for p in [Precalc::Off, Precalc::Ready { keys: 4, bytes: 1, stride: 64, secs: 0.5 }] {
        let mut h = row(p.clone());
        h.run();
        assert!(h.query_by_label_contains("Calculating").is_none(), "{p:?}: no progress text");
        assert!(h.query_by_label_contains("Stock removal").is_none(), "{p:?}: no incomplete text");
        assert!(h.query_by_label(CANCEL).is_none(), "{p:?}: no Cancel");
    }
}

/// The row the shell draws under the transport: present exactly while
/// the live worker has something to say, and its Cancel ends the run.
#[test]
fn the_shell_draws_the_row_from_the_live_worker() {
    let mut h = shell_harness();
    h.state_mut().state.sim = Some(Sim { t: 0.0, playing: false, speed: 1.0 });
    crate::actions::ensure_field_view(&mut h.state_mut().state);
    assert!(h.state().state.field_play.is_some(), "the transport starts a worker: {}", h.state().state.status);
    h.run_steps(2);
    // The calculation may finish before the first frame: then it is a
    // whole store, and the row is gone.
    let running = h.state().state.field_play.as_ref().is_some_and(|w| matches!(w.precalc(), Precalc::Running { .. }));
    if running && let Some(cancel) = h.query_by_label(CANCEL) {
        cancel.click();
        h.run_steps(2);
    }
    let t0 = std::time::Instant::now();
    let p = loop {
        let p = h.state().state.field_play.as_ref().map_or(Precalc::Off, |w| w.precalc());
        if !matches!(p, Precalc::Running { .. }) || t0.elapsed() > std::time::Duration::from_secs(120) {
            break p;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    };
    h.run_steps(2);
    match &p {
        Precalc::Ready { .. } => {
            assert!(h.query_by_label_contains("Calculating").is_none(), "a finished calculation has no row");
            assert!(h.query_by_label_contains("Stock removal").is_none(), "a finished calculation has no row");
        }
        Precalc::Incomplete { why, .. } => {
            assert_eq!(why, "cancelled", "only the Cancel click stops this calculation");
            assert!(h.query_by_label_contains("Stock removal incomplete (cancelled").is_some(), "the shell says so");
        }
        p => panic!("the calculation ends: {p:?}"),
    }
    assert!(h.query_by_label(CANCEL).is_none(), "{p:?}: no Cancel once the calculation ended");
}

/// #3315: Simulate with the Stock row hidden draws no stock body. The
/// line under the transport says why, in plain words, and goes when the
/// stock is shown again.
#[test]
fn a_hidden_stock_is_said_under_the_transport() {
    use crate::play_precalc::STOCK_HIDDEN_NOTE;
    let mut h = shell_harness();
    h.run_steps(2);
    h.state_mut().state.sim = Some(Sim { t: 0.0, playing: false, speed: 1.0 });
    h.run_steps(2);
    assert!(h.query_by_label(STOCK_HIDDEN_NOTE).is_none(), "a shown stock needs no note");
    h.get_by_label("Hide Stock").click();
    h.run_steps(2);
    assert!(h.state().state.active_stock_hidden());
    assert!(h.query_by_label(STOCK_HIDDEN_NOTE).is_some(), "the transport says the stock is hidden");
    // Plain English: it names the Browser row and no G-code word.
    for word in ["G0", "G1", "M3", "voxel"] {
        assert!(!STOCK_HIDDEN_NOTE.contains(word), "{word} in {STOCK_HIDDEN_NOTE:?}");
    }
    assert!(STOCK_HIDDEN_NOTE.contains("Stock row") && STOCK_HIDDEN_NOTE.contains("Browser"));
    h.get_by_label("Show Stock").click();
    h.run_steps(2);
    assert!(h.query_by_label(STOCK_HIDDEN_NOTE).is_none(), "the note goes with the hidden stock");
    // No transport, no note: the line belongs to the simulation.
    h.get_by_label("Hide Stock").click();
    h.state_mut().state.sim = None;
    h.run_steps(2);
    assert!(h.query_by_label(STOCK_HIDDEN_NOTE).is_none(), "no note outside the simulation");
}
