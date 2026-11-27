## #3643 - an app caller for the lathe resolver

**What was decided.** `crates/app/src/tool_assembly.rs` `derive` sends a
lathe-held tool's stack to the camcraft-tools lathe resolver
(`lathe_assembly::resolve_lathe`, PR #4194) in a new submodule
`tool_assembly_lathe.rs`. The document commands (`SetToolAssembly`,
`EditDocumentComponent`, `UpsertDocumentComponent`) land the Turning
record and its lathe-frame body in one undo step, as for a milling stack.

- **Turning only.** Only `ToolKind::Turning` goes to the resolver. A
  grooving, threading or form tool is refused by name: the resolver gives
  no blade width, thread form or form profile. A `Bore` tool keeps the
  spindle path. It serves both machines, and its kind does not tell which
  one holds it.
- **The station comes from the root's coupling.** A rectangular shank is
  `LatheStation::RADIAL_SHANK`, and a round or Weldon shank is
  `AXIAL_SHANK`. A VDI or other root is refused by name, because the
  document does not state its station yet. This holds the boring bar in a
  sleeve for its own slice, which is still open on #3643.
- **The orientation is always derived** from the geometry (shank, seat
  hand, insert). The tool's own `orientation` is not passed as a stated
  value: the record cannot tell a stated value from a derived one, and a
  derived value that is passed again would refuse a later holder change
  of hand. The record keeps the kind's default (2) as `None`, so a typed
  tool and the same tool from an assembly are one record.
- **What the record takes:** the nose radius, the orientation, the
  insert's inscribed circle as the `diameter` (the insert body size that
  the insert zone and the sanity law read), and the insert row's running
  data (tool material, cutting rows), as the milling path does. All other
  fields stay as the user set them. `Derived::gauge_len` is 0 for a lathe
  tool, because it has no spindle gauge line.
- **The body.** `ResolvedLathe` now also gives `placed`, the facets that
  its bands were read from (each body but the insert's, in the turning
  frame). The app builds the `ToolBody` from them with
  `tool_body_lathe::build`, with no insert zone, because the resolver
  already leaves the insert plate out. The bands are the same as the
  resolver's bands.
- **A lathe body on a row.** `DocumentComponent` gets
  `lathe_body: Option<LatheBody>`. `EditDocumentComponent::new` keeps it,
  as it keeps the cutter facts and running data. A spindle stack that names
  a row with a lathe body is refused by name, so that a body is never
  dropped in silence. The resolver already refuses a revolved body in a
  lathe stack.

**Why.** This is the first item of the "Done means" of #3644 and #3648:
a turning tool assembled from linked components. It uses the merged
resolver as it is, with one additive field, and keeps each refusal by
name.

**Considered and dropped.**
- Derive the record only, with no body: the equivalence with the
  tools-crate result would then cover only half of the result, and an
  assembled tool would lose its holder in the turning laws.
- Pass the tool's `orientation` as stated: see above (a wrong refusal
  after a change of hand).
- Persist the new field now: the component table is not in the project
  file yet. #4233 (PR for #3645) owns `crates/app/src/project/`. It must
  write and read `lathe_body` (boxes or facets) with the other row
  fields.
- MCP and UI for the lathe body: `crates/app/src/mcp/` (#4268) and
  `crates/ui/` (#4271) are in other open PRs. The commands are the same
  commands that those surfaces call.

**Cost to change later.** Small. The station and kind rules are in one
function each in `tool_assembly_lathe.rs`. A stated station or
orientation on the stack is a new field on `ToolAssembly` and one
argument to `resolve_lathe`.
