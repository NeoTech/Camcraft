## #3996 - Agie-only controls show only for an Agie job

**Decided.**
- The Post page section "Agie nominal inspection" (with its "Import DXF
  drawing" button) draws only when the setup context mounts the AGIECUT
  CLASSIC 2S wire machine. The hint "Mount the AGIECUT CLASSIC 2S machine
  ..." is removed: a user of another machine sees no Agie text.
- The operation page shows "Export Agie inspection setup" and the hint
  "Nominal inspection only ..." only when `has_agie_inspection_source` is
  true AND the operation's setup mounts that machine. The machine test is
  the same one that `prepare_inspection` uses (model and wire EDM kind).
- The Wire page open-cut rows "Both pieces clamped through all passes",
  "Stock centered in installed XY travel" and their Agie hint show only
  when the setup mounts the Agie machine or posts through the native Agie
  post (`posting::agie::is_native`).

**Why.** `has_agie_inspection_source` accepts every active die-side Wire
Contour on one DXF sketch chain. That is the normal DXF wire job, so the
export showed on every such job for every machine. Only the Agie post reads
the two open-cut flags (`agie_profile.rs`).

**Dropped.**
- Limit the export to an operation with no machining condition (the
  operation that "Create inspection opening" makes). Dropped: on an Agie
  setup the export of a conditioned die opening works today, and the issue
  says that Agie users keep the present behaviour.
- Keep "Both pieces clamped" as a generic row (naming audit, option 1).
  Dropped: no other post reads it, and the brief hides both rows.
- Renames (`ImportAgieDrawing`, the post name, the alias module). They wait
  for the open naming decision.

**Cost to change.** Small: the two predicates are in
`crates/ui/src/egui_app/agie_inspection.rs` (`agie_mounted`, `agie_job`).
