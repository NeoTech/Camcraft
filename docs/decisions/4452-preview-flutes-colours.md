## #4452 - 3D tool preview: flutes for each kind, a finer mesh, three part inks

**Decided.**

- **Kinds.** The 3D preview cuts flutes into every milling cutter with a
  cutting length, the twist drill and the tap (`flutes::fluted_kind`):
  end, ball and bull nose, chamfer (`Taper`), tap, both thread mills,
  T-slot, keyseat, dovetail, lollipop and form mill. The lathe tools, the
  probe, the electrode and the beam and wire tools stay plain. The side
  view's flute lines follow the same rule.
- **Counts.** The rule of #3908 stays: no stated count, no flutes. Two
  display defaults are added: a twist drill that states no count draws 2
  flutes at a 30 degree helix (DIN 338 type N), a tap draws 3 straight
  flutes. The tool keeps no count; nothing is written. The shipped drills
  and taps state no count, so without the defaults the drill on the
  issue's list still drew none.
- **Thread mills.** The flutes are cut into the toothed steel of #4451.
  The Properties side preview (`tool_preview::paint` without a steel
  override) also draws the teeth now (`thread_form::toothed`, now
  `pub(crate)` inside `crates/ui`).
- **Mesh.** The preview's revolve (`tool_library_dialog::model::RING`)
  takes `flutes::SEGS` = 96 facets around, up from 32. The collision and
  body meshes (`tool_body_view`, `Tool::geom`, the holder bands) do not
  change. The model is still built only when its input changes, and the
  screen mesh is projected only when the mesh, its parts, the rect, the
  orbit or the part inks change: a still frame builds nothing.
- **Inks.** Three new theme tokens in `crates/ui-kit`:
  `tool_model_flute` `#DEB25C` (a warm gold for the cutting part),
  `tool_model_shank` `#B0B6C0` (steel grey) and `tool_model_holder`
  `#6888B2` (slate blue). Both columns hold the same values, as the other
  tool inks do, because the previews keep their own dark ground in both
  themes. Each facet is the ink of its part, shaded from 45 % (edge-on) to
  100 % (facing the eye). A steel facet below the flute length (by its
  centre) is the cutting part, the other steel is the shank, and the
  holder facets (revolved holder or landed body) are the holder.

**Why.** The issue asks for flutes on each kind with a flute length, round
edges that look smooth, and a clear border between holder, shank and
cutting part, in theme tokens and with no colour literal.

**Dropped.**

- Reuse of `tool_flute`, `tool_shank` and `tool_body` (the side view's
  fills): they are three greys, and under the light shading they read as
  one grey again - the fault the issue reports.
- A default count for every kind: a keyseat or dovetail cutter without a
  count is rare in the catalog (all shipped ones state one), and a guessed
  count there is less sure than for a drill or a tap.
- More than 96 facets: `MAX_TRIS` of the fluted steel stays inside the
  view's painted budget (`MAX_PAINTED`) only up to 96.

**Cost to change.** Display only. The values are theme rows; a theme file
can set each ink by name. No toolpath, collision, simulation or posted
output reads any of it.
