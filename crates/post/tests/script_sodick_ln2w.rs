//! The `sodick-ln2w.cps` script suite (#4045, the per-script-suite law of
//! #926): the post binary on every recorded edge-case stream
//! (`recorded/streams/*.cnc`), its program or its named refusal compared
//! with the golden in `recorded/golden/sodick-ln2w/`. The recorded
//! streams are cutting streams posted machine-less (or on a mill), so the
//! Sodick LN2W wire post refuses each by name - it posts only wire EDM work on
//! a machine declared `wire_edm`; its wire program is pinned by app's
//! `sodick_aq600l`.
//!
//! Regenerate after an intentional change, then read the diff:
//! `WRITE_GOLDEN=1 cargo test -p camcraft-post --test script_sodick_ln2w`
//!
//! Gate map (M4.4): covers assets/posts/sodick-ln2w.cps, camcraft_post::post::engine,
//! camcraft_post::post::cnc, the post binary.

#[macro_use]
mod recorded;

post_suite!("sodick-ln2w.cps");

// The post's own words on synthetic wire streams (start order, taper, skims, refusals).
#[path = "sodick_ln2w/pins.rs"]
mod pins;

// The control-side wire offset (#4076): G41 / G42 H, G40 on the lead-out.
#[path = "sodick_ln2w/offset.rs"]
mod offset;

// A constant taper as a 4-axis program (#4075): TP / TN, G74, U V, G75.
#[path = "sodick_ln2w/taper.rs"]
mod taper;
