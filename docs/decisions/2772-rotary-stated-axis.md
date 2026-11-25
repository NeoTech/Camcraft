## #2772 parts 1-2 - the stated rotation axis and the disagreed-face refusal

**Decided:** this PR ships the first two parts of the issue's plan.
The rotary kinds (Contour, Parallel, Offset) read their axis from a
STATEMENT first: the op's own `stated_axis` direction, else the
mounted machine's stated rotary lines - a stated `axis` vector, else
the RS-274 letter's line (the index-swing clause's own `rotary_line`
reading) - each anchored through the setup's work offset origin and
JUDGED against the model's faces by the fit's own gates
(`multiaxis::axis_stated`: the concurrency share, the coarseness
skip, the bearing law, about the stated line). A revolve on the line
generates the very path the free fit reads (the canonical line is
the fit's own: signed and pinned by the same law). A line the faces
disagree with refuses by name - the part's own fitted axis and the
two acts that fix it when the model is a revolve placed off the
line; the freeform's 4-axis wording when no axis fits at all. Every
consumer reads the stated line first with today's fit as fallback:
`fixture::rotary_axis_of` (the sim's chord pass, the first traverse,
the post gate's struck-steel law), `rotary_law` (the radial field
verdict, which now takes the job's resolved axis), `rotary_picks`,
the centre-cutting audit's revolved bar, and the letter door (which
answers a stated line without a fit). Nothing stated keeps today's
behaviour bit for bit. The op param saves in the project file as
`stated_axis` (PROJECT_VERSION steps to v81 - revised at rework, see
below), edits in the property
page through the existing EditOperationParams commit, and patches
through `edit_operation`'s merge patch.

**Why:** the axis is a fact about the steel (a 4th-axis table states
its line; the WCS sits on the centreline by the indexed post's
setup law), not only a reading of the mesh - and a helical rotor
shares no normal-line axis at all. But a stated line is JUDGED, never
trusted: the radial kinds plan contact for a turned surface, and the
merged decision `2772-rotary-4axis-refusal.md` records why a looser
fit alone was dropped - on helical lobes the raster's radial pose
can gouge. So the disagreement refusal keeps the kinds locked on the
reporter's part, honestly named, until the radial drop kernel (the
NEXT slice, under a live claim) can consume the stated axis safely.

**Scope held:** the radial drop finish kernel and the radial-step
roughing are NOT here - `Refs #2772`, and the branch
`mkinney:issue/2772-radial-drop` holds that work. The stamp digest
writes the machine's stated lines and the anchor only when a line is
stated, so no cached path of a document without one goes stale.

**Dropped:** judging inside every motion kernel entry. The arm
resolves the axis once (`judged_any`) and hands the judged line to
the kernels' new `*_stated` entries; the plain entries keep the fit
inside, so turning, drilling and every existing golden path are
untouched. Also dropped: reading a letter-default line (an A with no
`axis` vector states) for a rotary named neither A/B/C - the post's
own door names that machine, and the resolver does not double it.

**Cost to change later:** the resolver is `crate::rotary_stated`
(world-free core `lines_of` + the world wrapper) over
`multiaxis::axis_stated`; the radial drop slice consumes the same
lines without touching the resolver. Remove the "not supported yet"
clause of the disagreement refusal when the radial drop lands.

**Revised at rework (PR #3674, mkinney's SAFETY review):**

- **The judge's radius is read ABOUT THE STATED LINE, never off the
  world origin.** `axis_stated::holds_data` computed the facet radius
  from `d.p` relative to the world origin; a common model or WCS
  translation (e.g. +10000 mm in Y) inflated the sagitta budget with
  the translation and let a 1 mm wrong line generate silently. Each
  radius is now the facet's distance from the stated line
  (`d.p - ax.origin`). Pinned by motion tests (a moved part holds its
  moved line on every coordinate axis; a wrong line on each of the
  three coordinate planes refuses at home and after any common
  translation) and by the app-level mutation the review probed
  (bar at Y+10000, stated X, clearance 100: WCS Y=10000 generates,
  Y=10001 and Y=10002 refuse by name).
- **The project version is v81, not v78.** Upstream main owns v78
  (`reserved_tools`, #2604), the conditional v79 (tip form, #3368)
  and, since the second refresh, v80 (`minimum_cone_radius`, #3153).
  v78 is refused (main's reader accepts it and silently drops
  `stated_axis` on re-save - the review's finding), v79 is refused
  (it is upstream's live tip-form version; a v79 stated-axis file
  would read as v78 and drop the same way), and v80 is main's own
  step. The next correctly-ordered free version is v81, marked
  breaking: older readers refuse a v81 file rather than turn the
  part about an axis the user never named. The v79 tip-form
  conditional stays: `tip_form_read_version` is now inert (the
  ladder reads a v79 file natively at 79), and the tip-form test
  carries #3153's asserts plus the v79-reads-natively pin. The #2826
  held branch numbered its Rotary Flow step v80 before main took
  v80; it must renumber past main's ladder when it refreshes,
  whatever this branch had taken.
- **The page's help follows the resolver.** The stated-axis group's
  hint said an absent direction uses the fitted axis before the
  machine line; `rotary_stated::lines_of` reads the machine's stated
  line first and fits only when nothing is stated. The hint and its
  comment now say the resolver's order, pinned by a text assertion.
- **UI tests** for the group live in
  `crates/ui/src/egui_app/tests/rotary_stated_axis.rs`: labels,
  roles, enabled state, the Apply commit and its undo, across the
  three rotary kinds; release PNGs unchanged.
