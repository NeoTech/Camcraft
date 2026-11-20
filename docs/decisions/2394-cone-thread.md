## #2394 - Example case turning/cone-thread: a cone with a threaded end

**What was decided**

1. **Two models, one axis.** The case imports the turned blank
   (`cone-blank.obj`: collar, cone, a plain 10 mm journal) and the finished
   part (`cone-thread.obj`: the same with the neck and the thread grooves).
   Face, Profile Rough and Profile Finish cut to the blank. The neck groove,
   the thread and the part-off cut to the finished part.
2. **A sixth operation for the neck.** The issue lists five operations. The
   neck is an undercut that the turning insert cannot reach, so a Groove
   Finish with the 3 mm blade cuts it (one plunge).
3. **G32 passes, not the G76 cycle.** The document keeps the default
   threading output.
4. **A custom threading insert.** `TD-1200-60`, 60 degrees, 1.2 mm wide,
   is given as a tool definition in `build.camscript`.
5. **The thread ends flush with the end face.** The groove centres are at
   Z63.9 + 1.5 k. The band of the last groove (Z76.8 to Z78.0) ends on the
   end face.
6. **A sharp shoulder.** The model has no fillet between the collar and the
   cone. The profile window starts on the collar at Z18, not on the shoulder
   plane at Z20.
7. **The pin reads the committed program.** The test
   `turning_cone_thread_case_posts_the_taper_and_the_thread` asserts the
   operation order, the taper slope of each finish block on the cone, and
   the thread passes. The two suite tests that iterate all cases give the
   regenerate, verify and byte-for-byte checks.

**Why**

1. The Threading kernel needs a modelled thread (V grooves). Profile Finish
   reads the valleys and the neck step as rest and blocks the post (#2880).
   No single model serves the two.
2. Without it the neck stays uncut and the thread has no relief.
3. The G76 start point is inside the part length. A Fanuc-type control
   moves to each pass depth at rapid at that point (#2878, filed P1). The
   G32 output plunges at feed. The issue asks for "the threading cycle";
   the program shows the threading passes instead, and the README says why.
4. The kernel requires `width < pitch - 2 * tolerance` and a reach of the
   thread depth. For M10x1.5 that is 1.01 mm to 1.4 mm. The catalog inserts
   are 0.866 mm and 1.732 mm wide (#893).
5. The verdict reads a band that goes past the end face as 5 mm of rest
   (#2879). A band that ends on the face is the nearest legal position.
6. A fillet's steep first chords send the ride out to X354 on a 30 mm bar
   (#2881). A window that starts on a shoulder plane starts on the same
   kind of degenerate station.
7. The byte-for-byte test fails on each change of the program, but it does
   not say what the program must show. The pin states the "Done means" of
   the issue.

**Considered and dropped**

- One model with a Profile Finish window that stops before the neck: the
  thread's major diameter stays at the rough allowance (10.5 mm). A wrong
  part.
- The G76 cycle as the issue's words suggest: not shipped as an example
  while #2878 is open.
- A copy of `TD-1732-60` with a changed width, made in the script from
  `list_tool_catalog`: shorter, but the reader does not see the insert.
- A catalog row for a 1.5 mm pitch insert: a tool library change, out of
  the scope of an example case (#893 covers the insert model).
- A fix of the kernel defects in this PR: they are motion and post changes
  under the safety floor, each with its own issue (#2878 to #2882).

**Cost to change later**

Low. When #2878, #2879, #2880 or #2881 is fixed, the case can go to one
model, the G76 cycle and a filleted shoulder: edit the generator and
`build.camscript`, run `cargo xtask catalog --case turning/cone-thread`,
and update the pin's thread asserts. When #2882 is fixed, the committed
`.nc` changes; rebuild the case in that PR.

**Review fix (PR #2883).** Keep both meshes embedded in the saved project.
After the imports, `build.camscript` uses `set_model_link` with `path: null`
to remove the source link. The UI checks relative source paths from the
process directory, which can differ from the case directory. An absolute
path would tie the example to one checkout. A change to the product path
resolver is outside this example fix.

The tests `every_catalog_case_opens_regenerates_verifies_and_posts_byte_for_byte`
and `every_catalog_case_rebuilds_from_camscript` pin the embedded meshes and
absent links for both models in the committed and rebuilt projects.
The UI test `catalog_cases_open_and_frame_cleanly` checks opening from
`crates/ui`.
