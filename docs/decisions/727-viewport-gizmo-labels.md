# Decisions for #727

## #727 - viewport origin gizmos wear their names

**What was decided:** every origin gizmo in the viewport labels itself. Each
work-offset triad draws its entity name - exactly what the Setup pane
shows, `G54` for the first setup, `Setup2/WCS` for the second - just under
its origin, in a bright axis color at normal label size. The dimmed world
axes draw `World` at theirs, neutral gray dimmed by `WORLD_DIM` like the
axes it stands for. Labels ride `gizmos()`: always visible, and an offset
hidden from the pane loses its label with its lines. Only the `X`/`Y`/`Z`
letters keep the big `is_axis_label` font; names draw at the ordinary
overlay-label size.

**Why:** with several setups on screen, the triads are otherwise
indistinguishable - the issue's ask was to tell whose origin is whose
without hovering, and a hover label cannot be read at a glance or
screenshotted into a setup sheet. The name at the origin is the same
identity the rest of the UI already uses, so the viewport needs no new
vocabulary.

**Considered and dropped:** hover-only labels (the issue asked for always
visible; hover answers "what is this" only for one gizmo at a time);
offset letters (`G54 triad X` per tip - six-plus glyphs per gizmo, clutter
that buries the axes the triad is for); coloring each offset's whole
triad by setup (the axes' RGB channels carry axis meaning already,
mirrored by the Setup pane's colorless rows).

**Cost to change later:** low. The label text is produced in `gizmos()`
and placed by `overlay_labels`; swapping always-visible for hover would
mean gating on `state.hover` in the shell's draw loop and leaving the
producer untouched. `setup_chip_tests` filters plane chips by `" · "`,
which never collides with an entity name - re-scope that only if names
start carrying separators.
