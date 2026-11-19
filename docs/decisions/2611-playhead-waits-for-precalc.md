## #2611 - The playhead waits for the stock removal calculation

**What was decided.** While the stock removal before playback is calculated
(`Precalc::Running`, #2181), the transport holds its playhead: `sim.t` does
not advance and the scrub slider is disabled. Play/Pause and Stop still work,
and Cancel stops the calculation (playback then cuts live, as before).

**Why.** The worker publishes no frame until the calculation ends, so the
viewport shows the uncut stock. The transport played on meanwhile, and the
tool moved over stock that showed no cut - on a large block the calculation
takes long enough that the whole program can play with no visible removal.
The transport line already said "playback starts when it is done"; the clock
did not obey it. A headless test of the worker's frames
(`crates/app/tests/stock_play_picture.rs`) showed the published frames do
draw the removal, so the defect is the clock, not the cut.

**Considered and dropped.**
- Publish progress frames from the calculation: the calculation cuts in
  program order, so its stock is at a position other than the playhead's.
  A picture at the wrong position is a silent wrong answer.
- Interrupt the calculation to cut live at the playhead: this is the lag
  that #2181 removed.

**Cost to change later.** Small: `play_precalc::advance_playhead` and the
`held` flag in `egui_app::shell`.
