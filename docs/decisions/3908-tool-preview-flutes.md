## #3908 - The 3D tool preview shows helical flutes (display only)

**Decided.** The tool library window's 3D preview cuts helical grooves
into the fluted length of the model that is revolved from the steel
meridian. A "Flutes" toggle stands after "Dimensions" and "Holder"
(view state, starts on). The rule, in
`crates/ui/src/egui_app/tool_library_dialog/flutes.rs`:

- Kinds with flutes: end mill, ball nose, bull nose, twist drill and
  chamfer mill (`Taper`). All other kinds draw the plain model: a tap,
  thread mills and form mills have a ground form, not plain flutes; turning
  inserts, blades, a probe, a torch, a waterjet, wire and electrodes have
  no flutes; T-slot, keyseat, dovetail and lollipop heads are short teeth
  that a groove rule does not show correctly.
- The count is the stated `flutes`, 1..=16. No stated count draws no
  flutes: the preview shows facts, it does not guess them.
- The helix is `catalog.helix_deg`, 0..=80 degrees; not stated, or out of
  that range, gives straight flutes. Right-hand helix: the twist is
  `z * tan(helix) / R` radians (lead `pi * D / tan(helix)`).
- Groove section: a sine over 45 % of each flute pitch; the floor is 60 %
  of the LOCAL radius. So a groove fades to nothing at a ball tip or a
  drill point, and the bottom face of an end mill shows the groove ends.
- The grooves run out over the top 8 % of the fluted length, so the
  fluted steel meets the plain shank without a crack.
- Mesh: 96 facets around, the fluted length in 16..64 rings (one ring for
  each 0.12 rad of twist). A meridian of more than 120 points draws no
  flutes. Bound: 2 * 96 * 119 = 22,848 facets of fluted steel, inside the
  model view's painted budget (`MAX_PAINTED` = 30,000), with room for the
  holder. A test pins the bound.

**Safety separation.** The fluted mesh is built in a private module of
`crates/ui` (`mod flutes`, not `pub`). No crate below `ui` (app, sim,
motion, post, tools) can name it. The model keeps its plain meshes, its
steel outline and its holder bands, which do not change with the flutes.
Pinned by `the_flutes_do_not_change_what_the_checks_read` (steel, bands,
plain mesh, `Tool::geom`, `Tool::profile` are equal with and without
flutes) and `the_fluted_steel_stays_inside_the_revolved_envelope` (each
fluted point is on or inside the revolved envelope).

**Frame cost.** The fluted meshes are built with the rest of the model,
on the same key (the draft tool value includes `flutes` and the helix).
The toggle only picks a mesh that is already built; it builds nothing,
and the projection cache projects the new pick once. The box of the plain
mesh fits the fluted mesh, so the view does not jump on a toggle.

**Why.** The user asked to see the flutes. The maintainer wants to keep
and grow the 3D preview ("running weirdly shape tools requires models").
A display-only groove rule gives that without risk to the checks.

**Dropped.** A fixed core of 60 % of the tool radius (a ball tip would
need a special case); a separate cache keyed on the toggle (a second memo
for a mesh that costs about a millisecond); guessed flute counts by kind;
grooves on the side view (the side view stays a to-scale outline).

**Cost to change.** The constants and the kind list are in one file. A
real fluted body (an imported model) already wins over the revolved one.
