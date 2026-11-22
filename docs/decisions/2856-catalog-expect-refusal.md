## #2856 - A catalog case can expect a refusal

**Decided.** The `expect` field of `case.toml` takes one of two values:
`"post"` or `"refusal"`. #2855 (PR #3139) read `expect` as a free-text
description. This change narrows it to the two values that #2856 specifies,
and the 12 existing cases change to `expect = "post"`. The field stays
required, as #3139 made it: each case has a `case.toml`, so a default value
never applies.

A refusal case has no `<case>.nc`. It has `refusal.txt`. Its script saves the
project, then runs the refused step without a catch, so the run ends with an
error. `refusal.txt` holds the `errors` lines that the script host records,
one line each. The CLI prints each line after `camcraft-cli: `; the file holds
the lines without that prefix. The readers skip the other stderr lines: they
are the log, and their times change on each run.

- The app test `regenerates` opens the saved project in the script host
  (`script::run`: `cam.load`, `cam.generate`, `cam.post`) and compares
  `ScriptOutcome::errors` with `refusal.txt`.
- The app test `rebuilds` runs `build.camscript` with `camcraft-cli`. It
  requires a failed exit, a saved project, no program, and the same lines.
- `cargo xtask catalog --check` does the same check as `rebuilds`.
  `cargo xtask catalog` (rebuild) writes `refusal.txt` from the run.
- A refusal case with a `.nc` file fails the layout check.

The first refusal case is `2d/drill-overfeed`: a Drill fed at 400 mm/min,
above the feed of its tool. The post refuses it by the #2598 clause.

**Why.** The issue names `expect = "post" | "refusal"`. A closed set lets the
tools act on the field. The free text that #3139 held duplicated the README.
The script host's `errors` list is the record that the host keeps for each
run, also for a caught refusal. The CLI prints the same list. So the two
tests and `--check` compare one record, and the host law does not change.
The overfeed refusal is deterministic, fast, and comes from one named law.

**Considered and dropped.**

- A second key (for example `result`) next to the free-text `expect`: the
  issue names `expect`, and two fields would say the same thing.
- An optional `expect` with the default `"post"`: #3139 requires all three
  fields, and a required field keeps one rule for each key.
- In-process `posting::post_setup_unwritten` in `regenerates`: its error text
  is not the line that the script host records (no `post: ` prefix, and a
  different entry point), so the two tests could disagree.
- A gouge or a machine-limit refusal for the first case: a gouge needs a
  larger model, and a limit refusal depends on the machine file.

**Cost to change later.** A third `expect` value needs a branch in
`xtask/src/catalog.rs`, `xtask/src/catalog_refusal.rs` and
`crates/app/tests/example_catalog/refusal.rs`. A change of the CLI's
`camcraft-cli: ` prefix changes one constant in each of the two readers.

**Review fixes (#3162).** Merge current main to retain the generated
`Slot Undercut` catalog block. Extend the guide link pin to require that block.
Move the adaptive-pocket regeneration and rebuild checks to the release heavy
lane because they exceed the 60 s debug budget. Keep both checks and their
assertions. A default-suite contract requires both names in the ignored test
list. The existing `example_catalog_adaptive` suite still checks the committed
program in quick. Do not reduce the model or increase the quick budget: those
choices either change the example or slow each quick run.
