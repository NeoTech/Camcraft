## #2064 - Theme tokens for status strip, ribbon tabs, window margin, and drop hint

**What was decided:**

- Add five `f32` tokens to `theme::Tokens`: `window_margin` (`6.0`, clamped `0.0..=24.0`), `status_h` (`22.0`, clamped `16.0..=64.0`), `ribbon_tab_pad_x` (`12.0` per side, clamped `2.0..=32.0`), `ribbon_tab_gap` (`2.0`, clamped `0.0..=24.0`), and `ribbon_tab_stroke` (`2.0`, clamped `1.0..=8.0`). Both `Tokens::DARK` and `Tokens::LIGHT` keep the prior defaults so default pixel output does not change.
- Replace literal `24` / `24.0` upper bounds for window and menu corner radius with `theme::clamp::RADIUS_WINDOW` (`24`).
- Add `pub const fn theme::status_h(t: &Tokens) -> f32`, returning the maximum of `t.status_h`, `t.control_height + 4.0`, and `t.font_row * 1.3 + 4.0`, and wire `shell::shell_layout` to it instead of a hardcoded `STATUS_H = 22.0` constant.
- Wire `Tokens::style()` to set `spacing.window_margin` from `clamp::range(self.window_margin, clamp::WINDOW_MARGIN).round() as i8` instead of `self.item_spacing_y.max(6.0)`.
- Wire `panes::ribbon_tabs` left margin to `pal.item_spacing_x`, tab box width to `galley.size().x + pal.ribbon_tab_pad_x * 2.0`, underline inset to `(pal.ribbon_tab_pad_x * 0.5).min(w * 0.25)`, underline thickness to `pal.ribbon_tab_stroke`, and inter-tab gap to `pal.ribbon_tab_gap`.
- Pass `&theme::Tokens` into `file_drop::paint_hint` so inset reads `pal.window_margin` (clamped to non-negative and `<= 0.4 * min(w, h)`), corner radius reads `f32::from(pal.radius_window)`, stroke width reads `(pal.font_label / 5.5).clamp(1.0, 6.0)`, and hint font size reads `(pal.font_label * 2.0).max(12.0)`. Overlay scrim and white ink stay untouched (tracked separately under `#1120`).
- Split `mod names`, `COLOR_NAMES`, and `GEOM_NAMES` out of `theme_files.rs` into `crates/ui/src/egui_app/theme_files/names.rs` to keep every file under 400 lines.
- Leave `assets/themes/theme.json.example` untouched on this branch because PR `#2038` introduces that file and is itself stacked on `#2027` (single-level stacking limit).

**Why:**

- Separating `window_margin` from `item_spacing_y` keeps dense list spacing independent of window outer padding.
- Computing `status_h` from `status_h`, `control_height`, and `font_row` gives theme files direct height control while preventing clipped widgets or text when typography scales up.
- Derived defaults keep existing themes (`horizon-day.json`, `teal-night.json`) at exact `22.0` px status bar height and `6.0` px window margin.

**Considered and dropped:**

- Folding `ribbon_tab_pad_x` and `ribbon_tab_gap` into one stride token: dropped because tab fill/underline width must expand cleanly around text separately from inter-tab spacing.
- Reading `t.status_h` without `control_height` and `font_row` floors: dropped because scaling fonts in a theme without editing `status_h` would clip the bottom strip.
- Stacking onto `#2038` (two levels above `main`) just to update `assets/themes/theme.json.example`: dropped per one-level stacking rules. A small follow-up can list the five keys in the example file once `#2038` and `#2064` merge.

**Cost to change later:**

- Low. The tokens and clamps sit behind `Tokens`, `theme/tables.rs`, `theme_files/names.rs`, `theme_files/overlay.rs`, and `theme_shell_metrics.rs`. External JSON themes only need backwards compatibility if shipped presets start overriding these keys.
