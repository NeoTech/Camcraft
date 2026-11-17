## #1473 - A picked start point with tabs: the tab windows are measured from the pick

**Decided.** A closed 2D Contour now takes a user start point when it has
tabs (item 3 of #1473). The picked ring starts at the pick on every level,
the tabbed levels too. On the picked ring, the tabs keep their count, width
and equal spacing, but they are laid from the pick: tab `i` spans
`[(i+1)s - width, (i+1)s]` along the walk, where `s` is the spacing. So the
first window begins at the pick, and the last tab ends on the seam, just
before the pick. Every level therefore enters the ring at the pick. The
picked start's lead-in law (refuse by name when no lead-in fits) is checked
on the first window only. The later windows begin after a tab, where the
tab law puts them, as before.

**Why.** "A pick is a wish": the user picks where the cut enters. With the
automatic tab layout (tabs centred at `(i + 0.5)s`), the window that holds
the pick crosses the seam, so a tabbed level would enter after the last tab
and not at the pick. A layout that ends a tab on the seam keeps the entry
at the pick on every level, and it does not change the count, width or
spacing of the tabs.

**Considered and dropped.**
- Keep the centred layout and only re-sequence the ring. The tabbed levels
  would then enter half a spacing away from the pick, which the user did not
  ask for.
- Put a tab centre at the pick. Then no level could enter at the pick on
  the tabbed levels.

**Not changed.** A ring without a pick keeps the automatic layout, move for
move (pinned). The exit point with tabs stays refused by name. Control and
wear compensation with a pick stay refused by name (item 4).

**Cost to change later.** One function (`Ring::tab_keeps_from_start`) and
one branch in `contour`; no file format change.

**Review fix (#1822).** A picked start with tabs accepts only no lead or
perpendicular leads. It refuses linear, arc and tangential leads by name,
including an independent lead-out. It also refuses either extension,
even when the lead shape is None. These shapes and extensions can cross
a standing tab that the part-boundary room check does not include.

The fix uses a conservative refusal instead of new tab collision geometry.
The refusal applies whenever tabs are configured, even above the tab top.
The existing perpendicular entry and tab layout stay available. Separate
lead-in and lead-out tests pin the reported cases. Tests also pin the other
lead shapes, both extensions, and the no-lead case.
