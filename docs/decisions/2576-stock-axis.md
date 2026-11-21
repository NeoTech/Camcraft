## #2576 - Stock axis on the Stock page: Along Z / X / Y / Custom

**Decided.** The Stock page gets a "Stock axis" choice: Along Z (the
default), Along X, Along Y and Custom. Custom shows three angles in
degrees (turn about Z, tilt about Y, tilt about X). The choice applies to
a box and to a cylinder. The turn is stored in the stock's existing
`Location.rot`, so the project file does not change and needs no new
version. The page reads a stored turn back: a preset turn shows its
preset, any other turn shows Custom with its angles.

- Apply sends the whole turn through the undoable `SetStock`. The UI has
  a new queue entry `CommandKind::SetStockPose { shape, pos, rot }`. The
  old `CommandKind::SetStock { shape, pos, yaw_deg }` stays as it is,
  because about 96 test sites use it. The two run the same dispatch.
- The MCP `set_stock` tool takes an optional `axis` ("z", "x", "y") or
  `angles_deg` ({yaw, pitch, roll}), not both. It calls the same
  `SetStock`.
- The simulation laws do not change here. `OBLIQUE_STOCK` still refuses
  a stock that the simulation cannot cut. The page shows a plain note
  before any simulation, from the same law (`stock_follow::sim_frame`),
  so the note and the refusal cannot differ. A box or a cylinder laid
  along X or Y is cut (the cylinder since #2758, merged from main), and
  no note shows.
- A DXF profile lies on the stock's top face. That face was read in the
  stock's own frame and ignored a tilt. Now an upright stock (any turn
  about Z) keeps its own top face, a stock laid square is read as the
  same solid unturned (the simulation's law), and a laid cylinder or a
  tilted stock is refused by name (`sketch::NO_TOP_FACE`).

**Why.** The maintainer asked for a cylinder that is not along Z. The
turn already exists in the document (`Location.rot`), and the viewport
already draws it. Only the page could not set it.

**Dropped.**
- An axis field in `StockShape::Cylinder`: it duplicates the turn that
  `Location.rot` holds, and it needs a project-file version step.
- Silent acceptance in the simulation: a laid cylinder would simulate as
  an upright one, which is a wrong answer. The refusal stays until the
  follow-up issue makes the simulation cut it.
- A rename of the old `CommandKind::SetStock` fields: it touches about 96
  test sites for no change in behaviour.

**Cost to change later.** Low. The axis choice is a view of
`Location.rot`. When the simulation cuts a laid stock, the note goes
away by itself, because it asks the simulation's law.

### Fit to Models on a turned stock (review of PR #2773)

**Chosen.** "Fit to models" (the Stock page button and MCP `fit_stock`)
measures the parts in the stock's own frame: each part vertex is turned
back by the stock's turn, the fit is made there, and its base point is
turned onto the setup's frame (`stock_fit::fitted_turned`). A stock laid
along X or Y wraps the parts along its own length. An upright stock uses
the old path unchanged.

**Why.** The review found that the fit put a world-frame size into a
laid stock, and the result did not hold the parts, with no message.
A fit in the stock's frame is exact for each turn, so the button stays
useful for a laid bar.

**Dropped.** A refusal of the fit for each turn that is not about Z: it
is safe, but it takes the button away from the case that the axis choice
was made for.

**Note.** A stock turned about Z only now fits on its own axes too. Before,
it took the world-frame size and could miss the parts at the corners.
