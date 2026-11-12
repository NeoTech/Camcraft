## #1195 - Plasma cutting machine mode, explicit table, mode rows name their section

**What was decided.** (2) The mode list stays an explicit table
(`MACHINE_MODES` in `crates/ui/src/actions.rs`), one reference per kind,
and gains a fourth row: `Plasma cutting` mounting the built-in
`Tormach 1300PL`. It is not derived from `posting::MACHINES`: a test,
`machine_mode_tests::every_kind_with_a_shipped_machine_has_a_mode`, fails
when a machine kind with at least one shipped machine has no row, and
`every_mode_mounts_a_shipped_machine_of_its_own_kind` fails when a row names
a machine that is missing or of another kind. So no EDM mode appears (no
EDM machine ships) and a mode that mounts nothing cannot land silently.
(3) Each row's text is its caption plus the sections the mode adds to a mill
document, labelled from the registry: `mode_label` in `actions.rs` walks
`operation::shown_for(kind)` minus `shown_for(Mill)` and uses `Section::label`.
The plasma row reads "Plasma cutting - Plasma"; Milling reads "Milling".
The mode menu's `(current)` suffix is unchanged.

**Why.** Deriving the reference from `MACHINES` needs a design decision
(which of several machines of a kind is the reference; today the table
names it). Keeping the table and pinning it with a structural test meets the
"Done means" without that decision. The section label is read from the
registry, so the row cannot drift from the section it names.

**Considered and dropped.**
- Deriving the table from `MACHINES` automatically: needs a reference-pick
  rule, which is a product decision; deferred.
- Naming the operation ("Plasma Profile") in the row: would need a hand-picked
  or joined operation list that can drift; the section label is the registry's word.
- Listing every section a mode shows: too long for the 200 px dropdown.
- Greying the hidden sections of the other modes with a reason: a redesign
  of the ribbon, out of scope.

**Known wart.** The Turning row reads "Turning - Turning", because the
Turning section is the extra section a lathe adds. Accepted; the row is
still the registry's own label.

**Cost to change later.** Adding a kind means one row here (the test says so
when it is missing). Switching to a derived table is local to `actions.rs`.
