## #1562 - "Pick for WCS" routes viewport clicks itself, the pick mode stays

**Decided:** while the Setups page's WCS role draft collects
(`wcs_pick::collecting`), the viewport click and hover go to
`feature_click` whatever the toolbar's pick mode is
(`interact::feature_routed`). `wcs_pick::arm` still leaves `geo_pick`
alone; disarming, pausing or a landed Apply needs no restore step,
because nothing was switched.

**Why:** the row takes faces, edges and corners together
(`feature_click::ask` already asks for all three while it collects), so
no single pick mode fits it; forcing one would blind the other kinds.
Routing on the draft's own state is the one-line gate the circle-centre
pick (#739) already used for its hover, and it cannot leave a stale mode
behind on any exit path.

**Considered and dropped:** have `arm` switch Model to Face/Edge/Vertex
and restore the previous mode on disarm, pause and Apply - three exit
paths to keep in step, plus a page leave, and any one missed leaves the
user in a mode they did not choose.

**Cost to change later:** small - one predicate in `interact.rs`.
