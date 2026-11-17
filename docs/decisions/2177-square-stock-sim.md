## #2177 - A stock in a vise is simulated: no locate noise, quarter turns modelled

**Decided.**
- `MoveModel` (and so `LocatePart`) keeps the stock's own rotation when the
  move does not turn the part (`turns()` false, the same 1e-12 tolerance the
  square refit already uses). Before, the rigid follow wrote
  `next * prev^-1 * before`: float noise of the locate solve (about 5e-14)
  became a stored stock rotation, and the sim refused the stock.
- One law, `stock_follow::sim_frame`, gives the stock as the sim models it.
  `stock_sim::stock_grid`, `stock_sim_held::first_body` / `first_field` and
  `stock_view::stock_facets` all read it. An unturned stock is unchanged.
  A box turned by a quarter-turn multiple (within 1e-9) is its unturned twin
  with the sides swapped. An upright cylinder (axis up or down) is an
  unturned one. A from-model stock is read from its placed mesh at any turn.
  An oblique box and a laid cylinder are refused by name, in plain words.
- The UI says a simulation refusal once (`State::say_once`) and in plain
  words: "simulation shows the uncut stock: <reason>". The text "the field
  view" and "the sheet stands" is gone from the status line.

**Why.** The maintainer saw no simulated stock and the refusal twice for a
block in a vise. The cause was the noise, not a real turn. A quarter-turned
box is the same solid as its twin, so the sim can judge it exactly. A
tolerance in the refusal alone (without the source fix) would leave noise in
saved projects and in the stock that a re-fit keeps.

**Dropped.**
- Only a tolerance in the two refusals: it hides the noise but does not
  model a true quarter turn of a from-model or box stock.
- An oriented lattice for oblique stock: new geometry, effort:max, not
  needed for the reported scene.

**Effect on existing projects.** A project whose stock carries noise or a
quarter turn was "not judged" by the stock sim. It is now judged: a gouge
that the sim finds can now block the post. No posted G-code changes.

**Cost to change.** `sim_frame` is one function; a future oriented lattice
replaces its refusal arm.

## #2177 review fix - the test device for "the sim does not judge"

**Decided.** Four suites (`engrave_law`, `mark_floor`, `neighbour_law`,
`stock_source_law`) used a stock turned a quarter about Z to make the
program sim refuse, so that the operation's own gouge was the only judge of
the post. The sim now judges that stock, so their nine `by_op` checks saw
the sim's gouge clause first. The device is now a turn of 1e-6 rad about Z
(`OBLIQUE`): the sim refuses it by name (`OBLIQUE_STOCK`), and each `by_op`
check now also asserts that refusal. The corners move by less than 1e-4 mm,
so the fixture's box check takes 1e-4 mm in X and Y (Z stays at 1e-9). Each
suite has one new test: the quarter-turned stock is judged by the program
sim, its gouge blocks the post, and a legal mark posts. The MCP parity test
of `stock_source_law` moved to `stock_source_law_mcp` to keep the file
under 400 lines.

**Why.** The new judgment is correct: each red case is a real gouge (the op
convicts it too), and the post was refused in all nine. Only the device of
the test was wrong. Deleting the `by_op` checks would lose the pin on the
generator-time verdict when the sim does not judge.

**Dropped.**
- A large oblique turn (for example 30 degrees): the stock footprint no
  longer covers the part, which changes the stock-source cases.
- A rotary or posed program as the device (`post_not_judged`): it changes
  the operation under test.
