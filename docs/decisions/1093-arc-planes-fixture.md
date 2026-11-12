## #1093 - arc_planes goldens carry a synthetic marker; the UNJUDGED list is gone

**Decided.** The five `arc_planes.nc` goldens lead with a comment line,
`synthetic program, posted for syntax conformance` (parentheses for the
Fanuc family, `;` for Siemens and Heidenhain), written only by the suite's
`WRITE_GOLDEN=1` path. `solid_law_goldens` accepts the marker only for a
stem no gate-judged op program makes, and fails a marked golden that is
judged, or an unmarked unjudged one. `UNJUDGED` is removed.

**Why.** A list of names let a new unjudged golden join it by editing one
line; a marker the guard cross-checks against the judged set cannot be
claimed by a real op program.

**Dropped.** A sidecar file (not "first line", and drifts); one shared
comment syntax (a `;` line is not a Fanuc comment). The Heidenhain frame
test skips the marker line. Cost to change: regenerate five goldens.
