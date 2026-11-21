## #2608 - simulation steps to the next and the previous operation

**Decided:**

- The lookup is `Script::op_starts`, `next_op_start`, `prev_op_start` and
  `op_starting_at`, in `crates/sim/src/sim_steps.rs`. `Script` is in
  `camcraft-sim`, so the transport bar and the stand-alone simulator
  (#2587) use the same steps.
- An operation starts at its first segment on the timeline. That is the
  connector rapid when there is one, because the script gives that rapid
  to the operation it arrives for. At that position every earlier move is
  complete and the operation has cut nothing. The stock playback reads it
  as "all moves of the earlier operations".
- "Previous" goes to the start of the current operation. When the
  playhead is already at that start, it goes to the start of the one
  before it.
- A step pauses the playback, so the user can check the operation, then
  press Play.
- The keys are Page Up and Page Down. They work only while a simulation
  runs and no text field has the focus. The tips name them; the app has
  no other shortcut list.
- The steps are disabled while the stock removal is calculated, like the
  scrub slider (#2611).
- The current operation is the first word of the existing readout
  (`sim_readout`), so no second label was added.
- The starts are calculated one time for each timeline, not in each
  frame (review of PR #2763, #2238 frame-cost rule). `OpStarts::of`
  walks the segments one time; `next`, `prev` and `op_at` are binary
  searches over one entry for each operation. The transport reads them
  through `sim_steps::cached_op_starts`, kept in `State` beside the
  cached script and keyed on the script's `Arc` (a `Weak`, so an old
  script is freed). The `Script` methods stay for a one-time question.

**Why:** a step is a scrub to a computed time. The stock follows the
playhead as it does for a scrub, so a step and plain playback show the
same stock. `sim_op_step_stock` compares the two pictures.

**Dropped:**

- The start of the operation's first move after the connector. The
  script does not mark a connector, and a first rapid of the path looks
  the same. The two positions show the same stock.
- Selecting the operation in the browser on a step (`SelectEntity`).
  That changes the selection and the property page, which the user did
  not ask for. A later issue can add it.
- Arrow keys. The slider uses them when it has the focus.
- A starts field in `Script`, filled by `sim::script`. `Script` has
  public fields and is built field by field in tests, so such a field
  can disagree with the segments. A separate value cannot.
- A cache in egui memory, held by the buttons. The UI rule says that the
  writer invalidates and that no widget holds a cache.

**Cost to change later:** low. The rules are in one file of each crate.
