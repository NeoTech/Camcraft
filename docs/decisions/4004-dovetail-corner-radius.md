## #4004 - Dovetail cutters read `corner_r` across tools, verification, stock simulation, and UI

**What was decided**

- `Head::Dove` carries `{ cot, corner_r }` and `Undercut::rho` reads `corner_r` (`c`) for `ToolKind::Dovetail`. Up to `s_tan = c * (1 + sin t)` (`t = 90 - tip_angle`), `rho(s)` follows the corner arc centred at `(r - c, c)`. Above `s_tan`, `rho(s)` follows the tangent flank `r - c + c * (1 + sin t) / cos t - s * tan t`.
- `Undercut::contact()` returns `(r, corner_r)` for a dovetail, which preserves `SlotUndercut` XY shift `width / 2 - r` at the widest diameter while reporting the true height of the widest point.
- `undercut::refusal` validates `corner_r` in `[0, diameter / 2]`, requires `s_tan <= flute_len + NECK_MEETS`, and checks neck clearance at `flute_len` against the widened rounded flank.
- `undercut::head_outline`, `Tool::meridian`, and `Cutter::atoms` emit the exact corner arc plus tangent flank. For `corner_r > 0`, `Cutter::atoms` uses `Prof::Zone` over `[0, s_tan]` (`cap: false`) and `Prof::Dove` over `[s_tan, fl]` (`cap: true`), or a single `Prof::Zone` (`cap: true`) when `fl - s_tan < meridian::MIN_LEN`.
- `SdfTool::Dovetail` and `RevKind::Dove` add `corner: f64`, and `field_dove::dove_dist` computes the exact Euclidean SDF of the rounded dovetail half-plane profile.
- The tool editor, Properties summary, and Draw cutter pane expose `Corner radius` / `corner R` for `ToolKind::Dovetail`, while `TSlot`, `Keyseat`, and `Lollipop` keep `corner_r` hidden and zeroed.
- `Frame` moved from `probe.rs` to `probe_frame.rs` and `dove_dist` moved from `field_solid.rs` to `field_dove.rs` to stay under the 400-line file limit.

**Why**

- A two-atom split (`Prof::Zone` plus `Prof::Dove`) matches `Tool::meridian().atoms()` one-for-one (`probe_meridian_tests`) and limits the `Atom::cap` roof allowance strictly to the top face at `flute_len`.
- Using exact `dove_dist` in `stock_sim` ensures both gouge verification (`solid_hits`) and field stock simulation reflect the wider rounded flank and rounded bottom corner.
- Preserving valid `corner_r` in `tool_undercut_row::seed` (and clearing only negative, non-finite, or oversized values) avoids discarding a typed radius when retyping a bull-nose draft to a dovetail.

**What was considered and dropped**

- Refusing `ToolKind::Dovetail` with `corner_r > 0` at `why_not_sane` or `SlotUndercut`: dropped because `.camtools` libraries already store `corner_r > 0` for dovetail cutters, and reading the true radius directly fixes the safety gap without breaking imports.
- Extending `Prof::Dove` into a single combined arc-plus-frustum atom: dropped because `Tool::meridian().atoms()` emits separate `Zone` and `Frustum` atoms, so a single atom would break meridian-to-probe parity and grant `cap` across the arc boundary.
- Reading `corner_r` on `TSlot` or `Keyseat`: dropped because their sharp disc encloses a rounded rim, so the current model cannot miss a gouge.

**What it would cost to change later**

- Internal to `camcraft-tools`, `camcraft-sim`, and `camcraft-ui`; no project file schema keys changed because `ToolDto::corner_r` already persisted `corner_r` for dovetail cutters.
