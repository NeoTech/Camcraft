## #2585 - Share the machine limit judgment without the ECS world

**What was decided:** Put the pure `MachineCheck` accumulator in
`camcraft-post::machine`. It checks tool count, rpm, tool size, and paths.
Each finding can use an optional `SourceLocation`, with an operation name
or an NC line number. Pass tool length, diameter, and rpm as scalar values.
Add no crate dependency.

**Why:** A stand-alone simulator needs the same machine limit checks as
the app. The post crate already owns the machine definition and move
types. The app keeps the ECS walk, holder resolution, spindle power,
part transfer, and the adaptation of the posted lathe path. It calls the
pure checks in the existing order to preserve its diagnostic output.

**Considered and dropped:** Adding a tools dependency would couple the
checks to a full tool definition when three scalar values are sufficient.
Moving world-specific code would bring document concerns into the post
crate. Changing diagnostic text would make this extraction change app
behavior and weaken comparison with the existing tests.

**Cost to change later:** Additional source kinds require an API change
and tests for their displayed locations. Additional tool properties can
extend the pure input API without moving the ECS walk. Moving the retained
app checks later requires plain inputs and tests that preserve diagnostic
order and text.
