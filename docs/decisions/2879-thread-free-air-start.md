## #2879 - Threading: allow free-air starts in front of the end face and keep the meridian lattice inside the bar

- **Decided:**
  1. Remove the post-filter re-insertion of `window`'s endpoints `(w0, w1)` in `revolve::lattice` (`crates/motion/src/turning/revolve.rs`). Every endpoint inside the bar domain `[d_lo, d_hi]` is already in `lat` from each move's `raw_band`; dropping the re-insertion keeps `lat` strictly inside `[d_lo, d_hi]` as the module's domain law states.
  2. In `thread_planned_in` (`crates/motion/src/turning/thread.rs`), compute `k_hi` from the unclamped `f.station_of(p.top_z)` instead of `blade::setup`'s clamped `s_hi`, while keeping `s_lo..=s_hi` clamped for `ref_top`, valley phase discovery, and the empty-window refusal. The same change landed on `main` first in #2892 (#2878) in the form `let s_hi_pass = f.station_of(p.top_z).max(s_hi);`. Because `blade::setup` sets `s_hi = f.station_of(top_z).min(d_hi)`, `s_hi <= station_of(top_z)` always and the two forms give the same value. The merge of `main` into this branch kept main's `s_hi_pass` form; this PR now adds the pinning tests of `turning_thread_end_face`, which also guard the #2892 fix.

- **Why:**
  - Re-inserting `w0` and `w1` after `lat.retain(|s| (d_lo..=d_hi).contains(s))` added intervals outside the bar whenever a move's band extended past an end face (by 0.1 mm on a free-end groove, by full pitches on a free-air lead-in, or by half a blade width on a free-end groove). In those air intervals `E(s) = 0`, so `revolve::scan` reported the capped wave floor (or blade tip) as rest and blocked the post.
  - Because `blade::setup` clamps `s_hi` to `d_hi`, setting `top_z` in front of the end face could never place lead-in lattice points in free air; every pass still started inside the part. Using `f.station_of(p.top_z)` for `k_hi` preserves exact bit-for-bit behaviour whenever `top_z <= d_hi`, keeps `k_hi - k_lo > blade::MAX_RUNGS` intact, and lets passes start in free air in front of the end face so spindle and feed synchronise (and so G76 cycles can start in free air, see #2878).

- **Considered and dropped:**
  - Skipping `e_u == 0.0 && e_v == 0.0` intervals inside `revolve::scan` while leaving `w0`/`w1` in `lat`: dropped because `revolve.rs` already documents that `lat` never lives outside `[d_lo, d_hi]` and keeping the invariant in `lattice` avoids special-casing air intervals in `scan`.
  - Unclamping `s_hi` inside `blade::setup`: dropped because `blade::setup` is shared with grooving and form turning, where `win` must stay clamped to the bar domain `[d_lo, d_hi]`.
  - Unclamping `s_lo` (the chuck end) in `thread_planned_in`: dropped because passes travel toward `-Z` (the chuck) and must not run past the chuck-end face.

- **Cost to change later:** Low. Both changes are self-contained inside `crates/motion/src/turning/` (`revolve.rs` and `thread.rs`) with no API or file-format change.
