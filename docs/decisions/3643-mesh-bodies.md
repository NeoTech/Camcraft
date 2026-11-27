## #3643 - Tool component bodies from STEP, STL and OBJ files, and MCS_INSERT (mesh-body slice)

**What was decided.**
- `MCS_INSERT` is classified as `CsysRole::Mounting` in
  `camcraft_geometry::step_csys` (the app's `step_csys.rs` re-exports
  that type, so the app reads it too). Only the exact name (case and
  blanks ignored) is taken; `MCS_1` or `MCS_INSERT_2` stay `Other`. A
  file that names both `MCS` and `MCS_INSERT` has two mounting frames,
  and `ToolCsysReport::find` refuses it as ambiguous. It is not guessed
  which one wins.
- A new `camcraft_tools::mesh_body::MeshBody` reads one component body:
  - STEP through the existing kernel import
    (`camcraft_geometry::step::parse`) at a 0.01 mm chord, or coarser
    inside the facet budget, with its named coordinate systems from
    `camcraft_geometry::step_csys`;
  - STL (ASCII and binary) and OBJ through a new small reader in
    `camcraft_tools::mesh_file`, with frames that the caller stores
    (`MeshBody::with_frames`), classified by their ISO 13399 names.
- The component frame is the body's one mounting frame (MCS or
  MCS_INSERT), as `ComponentGeometry` defines it: origin on the machine
  side, +Z toward the machine. Each workpiece-side port (CSW, CSWx,
  CSWx_y) and the cutting point (CIP) is read in that frame. A port or
  cutting point that is not below the mounting frame is refused by
  name (`NotBelowMount`): this catches a file whose MCS +Z points away
  from the machine, which would otherwise place the body upside down.
- A spindle body (`MeshBody::spindle_geometry`) is the revolved
  envelope of every facet (`HolderProfile::envelope`), drawn as a
  stepped `SpunProfile` and stored as a `ComponentBody`. The drawn
  outline only grows the envelope:
  - each join of two bands takes the larger radius;
  - bands closer than 0.02 mm in radius, or shorter than 0.02 mm, are
    joined, so no drawn segment is shorter than `MIN_SEGMENT`;
  - a band with no facet (a gap along the axis) gets 0.02 mm, not 0;
  - at most 31 bands remain, so the outline fits `MAX_POINTS`;
  - a last check refuses the body (`Outline`) if any envelope band is
    not inside the drawn outline over its whole length.
- A lathe body (`MeshBody::lathe_geometry`) is the facets as a
  `LatheBody`, as #4194 builds the lathe envelope: all bodies of the
  stack go to `LatheClearance::of` as one set.
- Never smaller than the steel: a meshed STEP body grows by the
  kernel's certified deviation (`StepMesh::deviation_mm`).
  - Spindle: each band radius and both ends grow by it.
  - Lathe: `LatheBody::padded` records it, and `resolve_lathe` grows
    the clearance bands by the largest pad of the stack
    (`LatheClearance::padded`, a Minkowski grow by a box, proof in the
    module comment of `lathe_clearance_pad.rs`).
  - An STL or OBJ body has no deviation: the file is the body.
- Facet budget: one component body has at most `MAX_BODY_FACETS` =
  1 000 000 facets, and a file at most 512 MB. Past the budget the body
  is refused by name, with #1041 named in the text. A STEP model is
  first meshed coarser by the existing importer (up to its six tries);
  only when no chord fits is it refused (`StepOverBudget`). Automatic
  coarsening of an STL or OBJ (#1041) is not done here.
- Every refusal has a name (`MeshBodyError`, `MeshFileError`). No path
  panics: the readers use checked lengths, `get`, and `try_from`. A
  fuzz-style test cuts short and flips bits in STL, OBJ and STEP files.

**Why.**
- The brief asks for the body in the crate that owns component
  geometry (`camcraft-tools`). That crate already depends on
  `camcraft-geometry`, which holds the STEP mesher and the coordinate
  system reader, so no new crate edge is needed.
- The app's STL and OBJ importers are in `crates/app/src/geometry/import`
  and return the app's `ImportError`, so `camcraft-tools` cannot call
  them. A small reader in the tools crate was chosen over a move of the
  app importers, because the move touches the model import path and its
  tests, which this slice must not change.
- A revolved outline is what the spindle laws read today
  (`ComponentBody`, `HolderProfile`), so the mesh enters the same
  judged path as a parametric holder, and the existing union of bodies
  in `tool_assembly_body::judged` works with no change.
- The deviation is a measured bound, so the grow is the smallest grow
  that keeps "never smaller than the steel" true for a curved surface,
  where the facets lie inside the surface.

**Considered and dropped.**
- A mesh variant of `ComponentBody` that the spindle laws read
  directly: dropped. It changes `judged` and the collision laws, and
  the revolved envelope is already how an imported holder mesh is
  judged today.
- Moving the app's STL and OBJ readers to `camcraft-geometry`: kept for
  later. A follow-up can make the app import call
  `camcraft_tools::mesh_file` or a shared reader.
- Refusing a pad larger than half a lathe band: dropped. The grow
  takes the union of as many neighbour bands as the pad needs.
- Taking `MCS` over `MCS_INSERT` when a file names both: dropped. The
  two are the same role, and a guess can place an insert wrongly.

**What it costs to change later.**
- The budget and the chord are constants in `mesh_file.rs` and
  `mesh_body.rs`. #1041 can replace the refusal with a coarser mesh.
- The app wiring is not in this slice: a document field for the mesh
  file and its stored frames, the commands and MCP tools that set it,
  and the call to `MeshBody::spindle_geometry` / `lathe_geometry` in
  `crates/app/src/tool_assembly*.rs` (open PRs #4249 and #4278 change
  those files now).
- The near-tip check of `resolve_lathe` (`BodyPastEdge`) reads the
  facets without the pad. It finds a wrong hand or seat frame, and it
  is not a clearance law.
- The boring-bar overhang in a sleeve is not in this slice.

**Merge with #4278 (`ResolvedLathe.placed`).** Main added
`ResolvedLathe.placed`: the app builds the lathe body and its bands
again from these facets (`crates/app/src/tool_assembly_lathe.rs`).
- Chosen: `placed` stays the bare facets, as on main. A facet set
  cannot grow by a distance, and the equivalence pin of #4278 compares
  the facets one to one. A new field `ResolvedLathe.pad` carries the
  largest pad of the stack. The app grows the bands that it reads from
  `placed` by `pad`, so the collision check reads the same grown bands
  as `ResolvedLathe.clearance`, never the smaller bands of the bare
  facets.
- Pinned by `a_padded_holder_body_keeps_the_grown_bands_in_the_app`
  (`camcraft-app`, `tool_library_holders`).
- Dropped: `placed` as grown facets (no exact facet grow exists); the
  app reading `r.clearance` in place of its own bands (it changes
  `tool_body_lathe::build`, which other callers use).

**Merge with #3645 (lathe bodies in the `.camproj`).** Main writes a
row's lathe body to the project file as its facets only. A padded body
lost its pad on a save and an open, and its bands became the smaller
bands of the bare facets.
- Chosen: `LatheBodyDto` gets an optional `pad` key. It is written only
  when the pad is above 0, so each file without a meshed body stays
  byte for byte the same. On open, the pad goes through the checked
  `LatheBody::padded`: a negative or non-finite pad refuses the file by
  name. An older build refuses a file with the key (unknown key), which
  is a refusal and not a smaller body.
- Pinned by `a_padded_lathe_body_keeps_its_pad_through_save_and_open`
  and the negative-pad case of
  `a_corrupt_lathe_body_refuses_the_file_and_a_library_save_refuses_the_row`.
- Dropped: a refusal to save a padded body (no app path makes one yet,
  but a refusal would block the mesh wiring later).

**Review of PR #4323 (three SAFETY blockers).**

1. A spindle body grows by its pad across an internal shoulder.
   - Found: the outline grew each band's radius by the pad, but only the
     first and the last band grew along the axis. Below an internal
     shoulder, a point within the pad of the steel stood outside the
     drawn body. The safety check read the bands at their old heights,
     so it did not see the gap.
   - Chosen: each envelope band grows by the pad at both axial ends and
     in radius (`grown_bands`). The outline is the union of the grown
     bands: each step takes the largest radius that reaches over it
     (`outline_bands`). The safety check reads the grown bands, not the
     old ones. A sweep with a running top keeps the union near linear
     in the band count.
   - Pinned by `a_spindle_body_grows_by_the_pad_across_an_internal_shoulder`
     (`camcraft-tools` lib, `mesh_body::safety_tests`).
   - Dropped: a grow of the first and last band only (the defect); a
     grow of each band downward only (a shoulder that faces up needs
     the grow upward too).

2. An assembled lathe tool keeps its pad through save and open.
   - Found: the app grew the effective body's bands in memory, but the
     tool body was saved as bare facets. On open the bands came back
     from the bare facets, smaller by the pad.
   - Chosen: the pad is part of the lathe body (`BodyFrame::Lathe.pad`).
     `tool_body_lathe::build_padded` grows the bands and refuses a
     negative or non-finite pad by name. Each rebuild keeps the pad:
     the assembly derive, the modular-piece rebuild and the insert-zone
     command. The tool body on file gets an optional `pad` key, written
     only when the pad is above 0, so each other file stays byte for
     byte the same. The reader is strict: a `null` or a text pad is
     refused (never read as no pad), a negative pad is refused by name,
     and a pad on a spindle body is refused by name.
   - An older build cannot open such a file: the padded component row
     of the same file has a `pad` key, and its strict row reader refuses
     it. So no older build reads the smaller bands.
   - Pinned by `an_assembled_padded_lathe_tool_keeps_its_bands_through_save_and_open`
     (`camcraft-app`, `tool_library_holders`): a padded PCLNR with a
     CNMG has the same bands and pad before save and after open, and a
     second save is byte-stable.
   - Dropped: a re-derive of the pad from the stack on open. The file
     states the derived body (the module law of `assembly_wire`), and
     a key on the body keeps that law. A hand edit that removes the key
     is the same case as a hand edit of the facets.

3. A stated but malformed frame is refused, never read as absent.
   - Found: the STEP reader kept a refused CIP as a reason, but
     `parts()` read it only when no mounting frame existed. A file with
     a broken CIP then used the default cutting point (another gauge
     length).
   - Chosen: one rule for the whole class. When the body has a mounting
     frame, each refused frame with a role that `parts()` uses (a
     mounting frame, a workpiece-side port or the cutting point) refuses
     the body by name (`MeshBodyError::RefusedFrame`). Without a
     mounting frame, `NoMount` lists the refused frames, as before.
   - Pinned by `a_stated_but_malformed_cip_is_refused_by_name`
     (`camcraft-tools` lib, `mesh_body::safety_tests`), with a broken
     CIP and a broken CSW in our STEP box.
   - Dropped: a refusal of the CIP only (a broken port is the same
     class); a refusal of a broken `PCS` or other frame that `parts()`
     does not use.
