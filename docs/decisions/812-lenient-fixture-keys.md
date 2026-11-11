## #812 - Lenient fixture keys, the strap clamp's stud, the parallels-fit hints

**What was decided.**

- **The `vise` and `accessory` keys are raw JSON on the wire.**
  `FixtureDto` holds both as `serde_json::Value`, and a new
  `project::fixture_keys` reads each into its typed form. A key that
  doesn't parse (an unknown accessory `kind`, a shape this build can't
  read, a mistyped figure) or that breaks its law is dropped on its own.
  The fixture and its saved facets stay, as plain steel, and the rest of
  the file loads. This is the shape #819 (PR #823) took for `plate` and
  `on_plate`. Dropped: a `#[serde(other)]` catch-all variant on
  `Accessory`. It covers an unknown kind but not a bad shape, and it
  would put an "unknown" arm into every match over the enum.
- **A dropped key is a load note, not only a log line.** `fixture_def`
  pushes a note naming the fixture, the key and the serde or law reason
  onto `check_and_apply`'s notes. `open_noted` logs each note, and the
  GUI's open puts the notes on the status line. Before this change a
  law-breaking key was only logged. The old log text ("vise parameters
  dropped on load") is kept inside the note, so log searches still work.
  `ViseDto::to_params` now returns the parameters unchecked, and the
  check moves to the loader. `accessory::checked_on_load` is removed for
  the same reason.
- **One shared helper, but #823's plate keys are not moved onto it
  here.** `fixture_keys::read` and `lawful` are generic. #823 isn't in
  this branch's stack, so its `plate_dto::link_from` still only logs.
  When both have merged, a small follow-up can route the plate keys
  through `fixture_keys` so that they also reach the notes.
- **A stud as wide as the strap, or wider, is refused by name.** The
  check compares the stud polygon's enclosing diameter with the strap
  width: "Strap clamp stud diameter ... must be less than the strap
  width ...". `InsertAccessory` and `SetAccessory` both build through
  `mesh()` and therefore `check()`, so the refusal happens when the
  command is created. Dropped: a ring shell around a wider stud (new
  geometry for a case a real strap can't have, because the stud passes
  through the strap's slot).
- **Parallels-fit hints.** In fit mode the height row reads "Height (set
  by the fit)" and is disabled. A hint says that the fit keeps the stock
  where it is and drops the vise, so a stock resting on the vise floor
  takes the vise below the table. The hint is static text. Dropped:
  computing the sink depth live in the dialog. That would need a dry run
  of the fit on every frame.

**Left for their own PR** (the #811 review items on #812, not in #809's
code): the dovetail lip hint and its "never less steel" wording, a
"Centre on stock" action for a placed self-centering vise, and
`CENTRE_TOL`'s absolute tolerance. #811's `self_centering` key sits
inside `ViseDto`, so it gets the lenient read when #811 merges main. It
only needs to change its `to_params` to the unchecked form above.

**Cost to change later.** Low. The wire format is unchanged, and only how
it is read differs.
