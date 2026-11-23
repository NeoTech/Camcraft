## #1874 - STEP length units: stated in the import notes, never guessed

**What was decided.**

- A STEP model import adds one note that names the length unit that the model was read in: `length unit: in (stated by the file)`, or `length unit: in (stated at import; the file states no length unit)`.
- A STEP file whose unit contexts state no length unit is refused with the typed `ImportError::StepUnitsMissing`. It is no longer read as millimetres.
- The caller can state the unit (`mm` or `in`):
  - `ImportModel::within_in` and `from_bytes_in`;
  - the MCP `import_model` argument `units`;
  - the fourth argument of `cam.import`.
- The kernel writes the stated unit into each representation context of the file before the read (`camcraft_kernel::step_assumed_unit`). The millimetre is an SI unit; the inch is a conversion-based unit of 25.4 mm.
- A file that states its own unit keeps it. A unit that the caller also gives is not used, and the note says so.
- A unit given for a file that is not STEP is refused by name before parsing, as a facet budget is.
- The desktop app asks with a modal prompt: Millimeters (mm), Inches (in), Cancel. The answer goes through the command queue (`CommandKind::ResolveStepUnits`) and runs the same import again. The import that lands is the one undoable command. Cancel and Escape import nothing.
- Tool holders, fixtures and tool bodies from a STEP file with no unit are refused (they read through `load_model_graded`). They have no prompt.

**Why.**

- #1572 and the Prime Directive: a guessed unit makes a part 25.4 times off with no word. A refusal with a named fix is the safe output.
- The unit goes into the file's contexts, not into a scale of the finished mesh. Then the stated tolerance, the edge-gap cap and the chord are all in the same unit as the geometry. A scale after the mesh would leave those in the wrong unit.
- The file's own unit wins over the caller's, unlike the DXF import (#1862), where the caller wins. A STEP unit context is an exact statement of the CAD system. `$INSUNITS` in DXF is often absent or wrong.

**Considered and dropped.**

- A scale of the mesh after the read. This breaks the tolerance and edge-gap rules (see Why).
- A caller unit that wins over the file's. The file's unit context is more reliable than a typed answer.
- A record of the chosen unit on the model (as the facet budget is recorded), so that a linked-file re-import does not ask again. The re-import asks again now. This is a small cost, and the question is correct each time. It can be added later with a schema component like `StepBudget`.
- A prompt for fixtures and tool bodies. These are rare from unitless files; the refusal names the fix.

**Cost to change later.**

- The note text is `unit_note` in `crates/app/src/geometry/import/step.rs`.
- More choices (for example cm or m) are one row in the prompt and one arm in `mm_per`. The kernel takes any size from 0.001 mm to 1000 mm.
- Recording the chosen unit is a new schema component, a project field and one lookup in `begin_import`.
