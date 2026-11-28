## #2872 - Example case edm/sinker: Sinker Burn and Sinker Side Burn

**Decided.** The case `examples/catalog/edm/sinker` burns one blind
rectangular cavity (20 x 12 x 8 mm, in a 60 x 40 x 25 mm block) with one
electrode. A rough Sinker Burn (`R1`) burns the cavity. A Sinker Side Burn
(`F1`) then burns to the floor and moves the electrode 0.22 mm toward +X
and back. The +X wall and the floor go to size; the other walls keep the
rough allowance.

- The machine is the shipped template `Sinker EDM 3-axis (template)`.
  The build script adds two conditions and all of the sinker code words
  with `edit_machine`. The values and the words are invented. The README
  and the script say so.
- The electrode comes from `electrode_from_faces` over the cavity faces,
  so the case also shows how an electrode is made. Its `burn` reply seeds
  both operations.
- The post is chosen with `add_setup` and `set_setup_post`, as the
  turning cases do. The regenerate check posts with the document's post.
- The side amount (0.22 mm) equals the allowance (0.25 undersize - 0.03
  finish gap). The wall thus reaches its drawn size.

**Why.** The shipped template states no codes, and the post refuses a
program without them (by design, #3001). An example cannot ship real
vendor codes (Clean-Room Protocol). Invented, labelled codes show the
whole flow and keep the program readable.

**Dropped.**
- A side burn on all four walls: four operations make the program longer
  and show nothing new about the kind.
- A new machine file with the codes: a second sinker machine in
  `assets/machines` would look like a real machine. The patch stays inside
  the example.
- A stated electrode with `add_tool`: `electrode_from_faces` is the
  documented route, and it keeps the electrode size tied to the condition.

**Cost to change.** Low. Edit `build.camscript` and run
`cargo xtask catalog --case edm/sinker`. The pin
`sinker_case_posts_the_burn_and_the_side_burn` states the depths and the
lateral X values, so it changes with them.
