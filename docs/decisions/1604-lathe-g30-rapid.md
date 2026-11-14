## #1604 - Tormach lathe: one axis at a time after G30, out to the stock's corner before G30

**Decided.** `tormach-lathe.cps` no longer writes a two-axis rapid next to
a `G30`. The change is in that post only. The mill post `tormach.cps` does
not change.

- **After G30** (a tool change, or a "home" after-operation), the section's
  first point is approached one axis at a time:
  - The point is outside the stock's diameter: `G0 X<point>`, then
    `G0 Z<point>`. X goes first.
  - The point is inside the diameter but in front of the face: `G0 Z`, then
    `G0 X`. The Z move goes to the point's Z. When the point is ON the face
    (a hole's entry), the Z move goes to the section's `ClearancePlane`
    first, then X, then Z down to the point.
  - The point is inside the diameter and not in front of the face: refused
    by name.
- **Before G30** (a tool change, a "home" after-operation and the program
  end), the tool leaves from the section's last point. It goes `G0 X` out
  to the stock's diameter (when it is inside), then `G0 Z` out to the face
  (when it is behind the face). When that corner is not clear of the stock
  in either axis, the Z move goes to the `ClearancePlane`. These lines are
  written before the next section's notes and work offset word, because the
  words are in the old section's frame.
- **The stock** is the section's `StockXYLeft/Right/Front/Back` box and
  `StockZTop`, the parameters every stream already carries. The radius is
  the box's largest half-width from the work-offset origin, and the face is
  `StockZTop`. No new stream parameter is added.
- **Refused by name:** no stock box, a first point inside the stock with no
  `ClearancePlane` in front of the face, a section that ends inside the
  stock's envelope, a section that ends on the stock's corner with no
  `ClearancePlane`, and a feed, arc or thread half-turn as the first motion
  after G30. After a refused approach, the post writes no motion until a
  point is reached. This is the same rule as #1633.

**Why.** LinuxCNC's G30 with no axis words moves every axis in one straight
line to the stored position (parameters 5181-5189, from the public LinuxCNC
G-code reference). The operator stores that position, so the post cannot
know where the line goes. The approach is safe for every stored G30 outside
the stock when each move stays outside the stock's diameter or in front of
its face:
- X first to a point outside the diameter: the X move runs at G30's Z. That
  is in front of the face, or G30 is outside the diameter and the move stays
  outside. The Z move then runs outside the diameter.
- Z first to a point in front of the face: the Z move runs at G30's X. That
  is outside the diameter, or G30 is in front of the face and the move stays
  in front. The X move then runs in front of the face.
- At the departure, the corner point is outside the diameter and in front of
  the face. A straight line from the corner to any point outside the stock
  stays outside it. Thus the control's own G30 line cannot cross the stock.

**X first, except for a point on the axis.** The coordinator asked for X
first, then Z. That order is kept for every point outside the stock's
diameter: all turning and threading starts. For a hole on centre, X first
from a G30 beside the bar moves into the bar. X first to a clearance
diameter needs a radial clearance number, and the stream does not state
one. A margin added by the post would be a guess. Z first, to the stated
`ClearancePlane`, needs no guess, so the post uses it there.

**Turret and gang plate.** On a Tormach turret the T word indexes the
turret at G30, so the index happens at the operator's safe spot as before.
On a gang plate the T word changes only the offsets, and the idle tools
move with the slide. The post knows only the active tool's control point,
so these rules protect that point only. The idle tools on a gang plate stay
the operator's layout responsibility. The X-first rule keeps the slide's long Z move outside the
stock's diameter, which is also the side where a gang plate's tools stand.

**Goldens.** The eight Tormach lathe NC goldens and the recorded
`tool_change.nc` change only in these lines. A script (run on this PR)
checked each diff: the first two-axis `G0` after each `G30` becomes one-axis
`G0` lines that end at the same point, the added lines before a `G30` are
one-axis `+X` or `+Z` moves, and every other line is the same and in the
same order.

**Considered and dropped:**
- A fixed radial margin (for example the stock's diameter plus 2 mm) for
  the approach to a hole. It is a guess, and the stated `ClearancePlane`
  does the same work.
- Write the departure at every section end. It adds moves where no G30
  follows. The departure is written only before a G30.
- Use the circumscribed radius of the stock box (half its diagonal). That
  is correct for a square bar, but the stream does not say if the stock is
  a round bar or a box. For a round bar it puts every turning start
  "inside" the stock, so every turning program would be refused. The
  largest half-width is exact for a round bar. For a box stock on a lathe,
  the rotating corners are not covered (follow-up below).
- Add a new stream parameter for the stock's turning radius. It changes the
  stream for every post and its recorded goldens, so it is not in this
  post-only change. It is the follow-up.

**Follow-up.** The stream does not state the stock's shape. A box stock
turned on a lathe sweeps a circle of half its diagonal, and this post uses
half its width. A follow-up issue (`P3`: a box stock can still be clipped
by a corner; the workaround is a round-bar stock) asks the stream to state
the turning radius of the stock.

**Cost to change later.** The rules are in one block of
`tormach-lathe.cps` (`approach`, `departure`, `g30`). The goldens carry one
to three added lines for each G30.
