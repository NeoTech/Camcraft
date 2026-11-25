## #2984 - Bind each native Agie work to its requested quality

Keep the saved condition IDs. Define one native quality identity for each supported wire, material and complete quality tuple. Reserve CCB030 and CCB035 for the two Berco Cut 0.25 mm selections. A different tuple requires a different identity. This replaces the work-index names QN2 through QN10. Those names could select an existing quality with different parameters. A synthetic job requested Ra 0.30 under QN2 and the controller displayed 0.70. The same request under a fresh name displayed 0.30.

Register the requested quality once for the piece and assign its returned handle to every work. Existing names can still select stale settings even when the call succeeds. An isolated control selected a stale CCB030 with Ra 0.70 for a new piece requesting 0.30. The post therefore leaves every work at zero working steps. The operator must inspect each loaded work and assign its technology on the same control after checking the material, wire, height, numeric targets and both visible options. The quality editor does not expose the third requested flag. Check the displayed values, working steps and offsets on the control at each qualified stock height. A successful formal check alone does not establish the effective quality or an achieved surface finish.

Keep native posting independent of a resident technology database. A new wire, material, parameter tuple or height needs a controller selection check and a measured envelope before it can be enabled.

### Port onto the zero-step setup (2026-11-24)

**What main already has.** The rewrite of #2984 on main (`ee0dafe23`, "prepare every first-writer Agie punch with zero working steps") and the ports of the stack below this PR replaced the old script writer (`crates/post/src/agie/jescript.rs`) that this PR changed. They cover these parts of the PR:

- Zero working steps on import: `camcraft_post::agie_setup` writes one quality object for the piece (`agie_setup/piece.rs`) and arms no technology. The receipt reports `working_steps_generated: false` and the control checks.
- One requested quality for each piece, CCB030 or CCB035, with the receipt offsets and the user-guide procedure for TECCUT.
- The repeat-import text ("a new piece") and the refusal of a stated X or Y travel (`posting_agie.rs`).
- The smoothing refusal, the plasma-stream refusal and the recorded-stream refusal goldens of the single-file dispatcher.

**What this port adds.**

- A group that mixes `AGIE_QN2` and `AGIE_BERCO_RA035` gets its own diagnostic in either work order, before the single-punch limit of CCB035. On main the same group was refused, but with the single-punch text, which did not say that the conditions differ. A group that requests `AGIE_BERCO_RA035` for every work still gets the single-punch diagnostic.
- The machine template text no longer tells the user to set X/Y travel endpoints, because the native post refuses a profile that states them.

**Dropped.** The PR's edits to `jescript.rs`, `quality.rs`, `tests/group.rs` and the old app tests. That code is gone on main, and the frame side holds each rule. No posted byte changes.
