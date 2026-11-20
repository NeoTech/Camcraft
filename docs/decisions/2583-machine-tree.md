## #2583 - MachineTree: the `.mch` part tree kept whole

**What was decided**

- `camcraft_post::machine::parse_mch_tree` reads `kinematics.default.parts`
  into a `MachineTree`: a flat, depth-first `Vec<TreePart>` in which each
  part holds the index of its parent. It is a second reader beside
  `parse_mch`. `Machine`, `parse_mch` and the existing checks do not change.
- The moving side follows the rule of the issue: a linear or rotary part
  that is an ancestor of a `head` part is on the tool side; each other axis
  is on the table side. A tree with no head states no side (`None`).
- A file in the mirror form (`3axis-vmc.mch`, `kinematics.static`) states no
  part tree. It reads as an empty tree, not as a refusal. The 3-axis parent
  chain is pinned on `vmc.mch`, which is the same machine in the file form.
- The tree door is strict where the model needs the fact. It refuses, with
  the part id and the key: a part with no `id`, an `id` stated two times, a
  `type` that is not `table` / `head` / `linear` / `rotary`, a `parts` value
  that is not a list, a garbled `axis`, `direction`, `pivot`, `offset`,
  `spindle` or `tool_station`.
- The unstated-limit law is shared, not copied: `rate`, `scale` and
  `mm_factor` of `parse.rs` became `pub(super)`, and `pivot` / `offset` go
  through `trunnion::parse_axis_fields`. A garbled `travel` reads as not
  stated, as in `parse_mch`.

**Why**

- An index to the parent keeps the type plain data (`Clone`, `PartialEq`),
  with no reference between parts and no recursion in the consumer.
- The model must find a part by its `id`. Two parts with one id, or a part
  with no id, would give a wrong joint silently. All 45 shipped files
  state unique ids and only the four types.
- One set of helper functions means that the tree and the flat axis list
  cannot scale a length differently. A test compares the two readers on
  each shipped file.

**Considered and dropped**

- A `tree` field on `Machine`: dropped. The issue says that `Machine` does
  not change, and `Machine` is serialized into project files.
- A nested `children: Vec<TreePart>` type: dropped. The parent chain is the
  fact the model needs, and a flat list gives it without recursion.
- A guess of the moving side from the axis letter (Z moves the head on a
  VMC): dropped. The reader states only what the file states.
- A refusal of the mirror-form file: dropped. `parse_mch` also accepts it
  and reads nothing from it.

**Known limit (not changed here)**

Most shipped 3-axis files hang the `head` beside the axis chain
(`vmc.mch`: `base > col > ram > sld`, and `base > head`). By the rule, all
their axes read as table side. Only a file that nests the head below an
axis (`tilthead.mch`, `sinker-edm.mch`, `millturn.mch`) states a tool-side
axis. A correct machine model needs those files nested as the machine
moves. That is a change to the machine files, not to the reader.

**Cost to change later**

Small. The type has no user outside its tests yet. A new fact is a new
field on `TreePart` and one line in `walk`.
