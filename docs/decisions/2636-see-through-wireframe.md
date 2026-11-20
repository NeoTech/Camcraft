## #2636 - "Wireframe, see-through" display mode

**What was decided**

- This work is stacked on PR #2835 (#2818). That PR adds "Shaded" (faces, no
  edges), the Display menu, the draw list (`draw_list::plan`) and the saved
  display mode. This PR adds only the mode that #2835 left out:
  "Wireframe, see-through".
- The mode is a fifth entry of the Display menu, after "Wireframe". The
  labels of the other entries are those of #2818. The mode has no control in
  the Browser pane, because #2818 moved the display choice to the viewport
  menu.
- The draw list of the mode has no face step: no lit faces and no depth-only
  faces. It draws the cached model edges, the cached stock edges and the
  region fills. The depth buffer stays empty, so the model hides nothing: not
  the stock, not a fixture, not a toolpath.
- All edges draw in the same wireframe ink.
- In the mode, a click asks the feature pick for `through`, as the "Pick
  through" switch does. The rule is the existing rule of `pick_bodies`: the
  edge nearest to the cursor wins; at the same distance from the cursor, the
  edge nearest to the camera wins. The switch itself stays as the user set it.
- The wireframe switch of the toolbar shows as active in the two wireframe
  modes. One press in either gives "Shaded, with edges".
- The stored key is `wireframe-see-through`. An older build reads an unknown
  key as the default mode (#2835), so the file stays readable in both
  directions.

**Why**

- #2636 asks for the mode in the maintainer's words: "wireframe \"see thru\"
  as well".
- A depth prepass would hide the toolpath behind the model. The issue says
  that the model must not hide what is behind it.
- The pick rule already exists and has tests. A second rule for one display
  mode would give two answers for the same click.

**Considered and dropped**

- Hidden edges in a lighter tone. The issue says "may". It needs a depth
  prepass and a second edge pass, and the prepass hides the toolpath lines
  behind the model. "Shaded, with hidden edges" (#2835) already shows the two
  tones on a shaded model.
- Box selection that takes hidden edges in the mode. The issue names only the
  click. The box follows the "Pick through" switch, as before.
- Four radios in the Browser pane, as #2636 first described. #2818 is the
  later instruction and moved the choice.
- A new pick order (camera depth first). The issue's sentence agrees with the
  existing order, and the test states it.

**Cost to change later**

- The lighter tone for hidden edges is one more `Pipe` step and one pipeline.
  The draw list test of the mode then changes.
- A rename of the stored key needs a reader for the old key.
