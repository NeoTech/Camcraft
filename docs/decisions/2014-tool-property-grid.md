## #2014 - Tool settings are a property grid with the labels to the left

**What was decided.** The tool page in Properties draws every setting as
one property row: the label in the left half of the pane, the input in the
right half. The rows stand in four collapsible groups, in this order:
Geometry, Holder, Cutting data, Catalog. `crates/ui/src/egui_app/tool_settings_grid.rs`
holds the grid. `tool_settings.rs` keeps the page, the three columns, the
Apply / Add commands and the kind seeding.

- Geometry holds the type, the cutting shape (diameter, or the kerf, wire
  or electrode sizes), the kind-read rows, the corner radius, the flute and
  the shank.
- Holder holds the holder diameter and length.
- Cutting data holds spindle rpm, feed, the per-material table, the
  turning rows and centre cutting.
- Catalog holds the material group, the catalog facts and the tool number.

The inline help paragraphs became hover tooltips on the row labels
(`prop_row_hint`), per the form law in `layout/form.rs`. The waterjet
recipe and the catalog rows lost their own nested headers: their group is
the one header now. The per-material table keeps its six columns and
scrolls across inside the Cutting data group, so it never widens the page
or the tool editor window. A group's title folds the group as its arrow
does (`group_open`); before, only the arrow was clickable.

**Why.** The maintainer reported the crammed page twice: each label stood
above its input and the right half of the pane was empty. The form helpers
already state the label-left property grid and the collapsible group
(#1122). The page follows them now instead of its own stacked form.

**What was dropped.** The modal `tool_editor_window` in `pages.rs` stays.
It shares `tool_settings::contents` and is retired separately (#2003);
this change is safe for it but removal is out of scope. A `Catalog` group
title now exists on the tool page; the catalog dock's own "Catalog"
heading is a different widget in a different pane. The #2002 tests assert
the dock's controls by their own names, so they still guard that
separation. The waterjet recipe caption "Cutting data by material and
thickness" is gone; the group header "Cutting data" carries it. The plasma
cut-data recipe PR (#2596) brought that caption back on `main` as a nested
`CollapsingHeader`; the merge of `main` into this branch applied the same
rule again and kept the grid's single group, with the plasma PR's rules
intact (a recipe does not fill a material slot).

**What a change would cost later.** A new tool field goes in one group of
`tool_settings_grid.rs` as a `prop_row`; a new group is one `group_open`
call. A test that moves a row expects the label and the input to share a
row (vertical centres within 4 px) and the label to sit left of the input.
