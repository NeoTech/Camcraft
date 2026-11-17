## #1966 - G93 inverse-time feed solver and explicit SectionMode markers

- **What was decided**:
  1. Emit `PARAM SectionMode <mode>` (`"3-axis"`, `"3+2 indexed"`, `"simultaneous 5-axis"`) per generated milling operation stream via [`post::set_section_mode`] (called from `crates/app/src/posting.rs` -> `posting_section_mode::insert`), leaving `Dump::default()` and `post::dump` untouched and keeping turning/plasma/EDM/live-tool operations unannotated.
  2. Implement the shared `G93` inverse-time feed solver in `crates/post/src/post/g93.rs` (`dt_cut = ||tip_1 - tip_0|| / v_prog`, `dt_lim = max(|d_k| / v_k_max)` over linear `[X, Y, Z]` mm/min and rotary `[A, B, C]` deg/min limits, `dt_min = max(dt_cut, dt_lim)`, `F = clamp(1 / dt_min, f_min, f_max)` with `f_min = 0.001`, `f_max = 60_000.0`, floor-formatted to 4 decimals).
  3. For pure rotary reorientations (`ds_tip <= eps`, `max_d_rot > eps`), honor `rotary_only_deg_per_min`, else stated rotary/linear axis `max_normal_speed` limits, else programmed `v_prog`, and refuse cleanly via `G93Error::MissingFeedAndLimits` if neither feed nor limit exists; return `G93Decision::Skip` for duplicate zero-motion moves.

- **Why**:
  - Adding `set_section_mode` as a helper invoked by `app::posting` (matching `ask_reverse_spindle`) avoids modifying `Dump`/`dump()`, `js.rs`, or `engine.rs` (preventing merge conflicts with open PRs `#468`, `#1681`, `#2115`, `#2136`) while preserving byte-identical legacy `.cl` / `Dump::default()` streams and 3-axis `.nc` programs.
  - Extracting `AxisSpeedLimits` from `Machine::axes` (`max_normal_speed` scaled by `parse_mch`) and computing `dt_cut` strictly in part tip coordinates ensures `G93` `F` words (`min^-1`) remain identical under `G20` (inch) and `G21` (mm) and prevent pivot-compensation lever arms or slow rotary axes from overspeeding.
  - Flooring the formatted `F` word to 4 decimals prevents rounding upward into a machine speed violation.

- **What was considered and dropped**:
  - Adding a `section_mode` field to `post::Dump` — dropped because several open PRs modify `Dump`, and direct `post::dump(&tp, &Dump::default())` streams used by `.cl` and recorded golden suites must remain byte-identical.
  - Re-wrapping rotary deltas `(rot_1 - rot_0)` mod 360 inside `g93.rs` — dropped because `kinematics::solve_path` outputs unwrapped continuous joint targets where `|to - from|` is the true physical travel (a 270 deg rewind must be timed for 270 deg, not 90 deg).

- **Cost to change later**:
  - Low: `SectionMode` values and `G93Options` defaults are centralized in `crates/post/src/post/{section_mode,g93}.rs`, and downstream `.cps` wiring (#1968/#1969) consumes them directly without breaking existing callers.
