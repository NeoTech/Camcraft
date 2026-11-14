## #1666 - The lathe stream states the stock's turning radius

**What was decided.**

- `post::dump` writes a new PARAM, `StockTurnRadius`: the largest distance of the stock mesh's vertices from the world Z line through the work offset's origin (`OriginX`, `OriginY`). The app computes it in `app::stock_turn::turn_radius_for`.
- The PARAM is written only when the operation's setup is on a lathe or a mill-turn. A mill stream does not change.
- `tormach-lathe.cps` uses the larger of `StockTurnRadius` and the box's largest half-width as the stock's radius around G30. A stream without the PARAM (a hand-built or older stream) keeps the box's half-width.
- `fanuc-lathe.cps` does not change. It does not read the stock, so the PARAM has no use there.

**Why.**

- The largest distance of a polyhedron from a line is at a vertex, so the vertex maximum is the exact swept radius of the mesh. It is correct for a round bar, a square bar, a bar off the axis, and a stock from a model. The stream does not have to say which shape the stock is.
- The faceted cylinder has its vertices on the true circle, so a round bar gets its own radius. The posted G-code of a round bar does not change.
- The maximum with the box half-width makes the post safe when a stream states a turning radius that is too small.

**Considered and dropped.**

- A stock-shape PARAM (round or box) and the half-diagonal in the post: it fails for a bar off the axis and for a stock from a model.
- The PARAM on every stream: no mill post reads it, and it would change the mill streams for no gain.

**Cost to change later.** Low. The PARAM is optional. A post that does not read it is not affected.
