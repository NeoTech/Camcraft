## #902 - Program units are a document setting

**Decided.**

1. **One setting, not two.** The document's existing Units choice
   (`SetUnits`, already undoable and saved as `units`) is the unit every
   post writes the program in, as the issue recommended. No second
   "program units" field was added; the machine file's
   `units.length` only says which unit the file's own numbers are in.
2. **How a post hears it.** The app adds `Units: "mm" | "inch"` to the
   `postOptions` object (`posting::program_options`) for every post path
   (whole program, per-operation files, per-mount files). The stream
   stays mm. `post.exe` takes `--units mm|inch` (default mm).
3. **The manifest law.** A script that writes inch declares
   `programUnits = "mm inch"`; the engine refuses an inch document on a
   script that does not (`post::units`), by name, before any hook runs.
   A user's own pre-#902 script therefore keeps posting mm documents and
   is refused on an inch one instead of writing mm numbers for it. An
   unknown unit word is refused, never read as mm.
4. **Which posts write inch.** Fanuc, Haas, Mazak, Doosan DNM (G20),
   Siemens 840D (G700, inch for lengths and feeds, the G710 law's twin)
   and the three Tormach posts (which already converted, now from the
   document). Inch output: lengths / 25.4 at four decimals, feeds in
   in/min at two, angles / seconds / rpm unchanged; the macro probe
   checks' "same position" band is one output step (0.001 mm or
   0.0001 in).
5. **Which posts refuse inch, by name.** Makera Carvera (firmware is mm
   only - its G20 halts the machine), **Heidenhain** and **Fanuc Lathe**.
   Heidenhain: the TNC runs inch (G70), but the feed unit its inch
   programs and cycle Q feeds take was not confirmed from the public
   manual the post is specced from. Fanuc Lathe: inch turning is more
   than dividing by 25.4 (G96 S in surface feet, G99 per-rev feeds,
   G76's Q / R in the inch increment). Both are follow-up issues (#976
   Heidenhain, #977 Fanuc Lathe); a feed
   off by a factor is not a program to post.
6. **Old projects post byte-identical.** `PROJECT_VERSION` 59. A v58 or
   older file opens on the unit its programs posted in (the units of
   the machines its setups posted on); if that differs from its saved
   display unit, the display changes and a load note says why. A file
   whose setups posted in different units keeps its display unit and a
   note names every setup whose program unit changes (one document
   setting cannot keep both).
7. **Where the user finds it.** The Post section (document and setup
   Post pages) shows a "Program units" mm / inch row - the same
   `SetUnits` as the Stock page's Units row, now shared code with a
   hover saying it is the posted unit too. The post status line says
   the unit ("posted 3 op(s), 120 lines in inch to ...").
8. **MCP.** New `set_units` tool (the `SetUnits` command, one undo
   step); `get_job` reports `units`.
9. **Angles.** Unchanged - degrees everywhere (maintainer, 2026-11-11).

**Considered and dropped.**

- A separate document `ProgramUnits` setting next to the display unit:
  it keeps every old file's display untouched, but it is the "two
  settings" the issue asked against, and it lets a user read inches on
  screen while posting mm by accident.
- Letting the engine overwrite `machine.units` with the document's unit
  so old custom scripts follow it silently: magic, and it hides the
  second derivation instead of removing it.
- Converting Heidenhain and Fanuc Lathe now from memory of their
  manuals: not specced from a confirmed public source, so refused.

**Cost to change later.** Adding inch to a refusing post is that post's
own work plus `programUnits = "mm inch"` and its goldens. Splitting the
display and program units later would add one field with this v59 rung
as its default.
