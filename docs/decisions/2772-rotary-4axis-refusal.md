## #2772 - Rotary refusal names the 4-axis case (first slice)

**Decided:** this PR ships only item 3 of the issue. When the axis fit
finds no common rotation axis, the refusal now says that the model is not
a shape of revolution, that the rotary kinds need one today, and that
4-axis work on a part clamped on A that is not turned (helical lobes, a
carving) is not supported yet (#2772). The 5-axis pointer stays, for users
who have a 5-axis machine. The words "share no rotation axis" stay, so
existing tests and callers that match on them do not change. The commit
uses `Refs #2772`, not `Fixes`: the issue stays open.

**Why:** the reporter has a 4-axis machine. The old text sent the user to
the 5-Axis kinds only, which this user cannot run. The full feature is
several PRs of new behaviour on the safety floor:

1. a stated rotary axis (from the machine's A line and the WCS, or fitted
   from the turned faces only) in the operation parameters, the project
   file, the UI and MCP, with a refusal when the stated axis and the
   turned faces disagree;
2. the same axis read by every consumer that today re-fits it from the
   mesh (`fixture::rotary_axis_of`, `first_traverse::turned_axis`,
   `rotary_law`, `rotary_picks`, the rotary stock simulation);
3. a radial drop finish: at each (X, A) station, a ball drop along the
   radial direction onto the model (a vertical ball drop-cutter in the
   frame turned by A), posed radial, with the radial gouge check;
4. radial-step roughing from the cylinder stock.

One worker run cannot deliver and verify all of that safely. A clear
refusal is the correct output until the axis is stated, by the Prime
Directive.

**Dropped:** "fit the axis from the turned faces only" as a quick first
step. The current Rotary Parallel raster plans contact for a turned
surface. On helical lobes, its radial pose can gouge, so a looser fit
alone would change a refusal into unsafe output.

**Cost to change later:** none. The text is one string in
`crates/motion/src/multiaxis/axis.rs`. Remove the "not supported yet"
clause when the stated-axis radial finish ships.
