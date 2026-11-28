## #2586 - camcraft-sim check: what each check judges, and what it names as not judged

Decisions taken unattended while the first slice of #2574 was written.

**The limits are judged in program coordinates.** A `.mch` stroke is in
PROGRAM coordinates (`assets/machines/AGENTS.md`, `travel`; #2574 answer 3).
The NC moves are program coordinates, so the #2585 check runs with a zero
origin and needs no `--wcs`. A stroke that the file does not state is named
"not judged" (the check itself skips it in silence). Dropped: judging
`work + --wcs` against the stroke (that frame is not the file's).

**`--wcs` is for the machine model and the frames.** The machine-part
collision poses the model at `work + origin`; an offset with no `--wcs`
names its lines as not judged. The stock and the fixtures are in the frame
of the first section's offset; a section in another offset is moved by the
difference of the two origins, or named as not judged.

**G28 and G30 go to machine zero.** That is the reference point of a control
that states no other (the Fanuc default). In an offset with no `--wcs`, a
G28, G30 or G53 move has no known work position: the parser gets a
stand-in origin so that it completes, and each such move (and the move
after it) is skipped by each check and named ("reference moves", exit 3).
Dropped: a refusal of each program that uses G28 with no `--wcs` (most
posted programs end with G28); a guessed home.

**The first move: only its end is judged.** The program does not say where
the machine stands at its start. The report says so in a note. Dropped:
the parser's guess (the work origin) as the start, which can read a
diagonal rapid through the stock; a "not judged" for each program.

**The rapid law reads the simulated stock.** A rapid fails when its tip disc
(the centre, and eight points at the radius less one cell) is more than one
cell deep in the material that is left before the rapid. Dropped: the
geometric law of `cl` (each rapid under the stock top inside the
footprint), which flags a rapid into a pocket that an earlier move cut.

**No "cut under the stock bottom" in `check`.** The catalog program drills
through the stock bottom, which is legal. What stands under the stock is
judged by the fixture law and the machine model. The `cl` subcommand keeps
its old verdict unchanged.

**The holder law reads the simulated stock move by move.** The shank band
(when the T map states a shank) and the holder band must not be more than
one cell deep in the material that is left before the move. `verify`'s
region-based shank law is per operation: on a whole program it reads stock
that earlier operations cut, and it over-reports. To keep the cost low, the
moves are cut in batches, and each move is probed on the stock before its
batch. Material only goes, so a clean probe on that stock is clean on the
exact stock; a suspected hit cuts the batch and probes again. The verdict is
the move-by-move verdict. A tool with no `flute=` and `holder=` is named as
not judged.

**The fixture law is `verify`'s region-free law.** `bands_vs_fixtures` and
`rapids_vs_fixtures` over `--fixture` boxes. With no `--fixture`, the line
says "no fixture is stated" and passes: the program cannot know of a
fixture that the user does not state.

**The machine model.** The tree maps to `MachineSpec`: a linear part drives
the letter of its `name` (else its `id`); a table-side part runs against
the program axis; a rotary is posed at zero. The spindle side is a head and
each axis whose moving side is the tool; the work side is each other
`Block` body. The tool envelope is not steel and is not judged. Bodies are
tested with the separating axis theorem (each body is convex), at poses
about 2 mm apart along each move, with a 0.01 mm touch tolerance. In the
generated model, joint zero puts the spindle nose on the table top
(#2584).

**Inputs.** The dialect comes from `--dialect`, else from the post's file
name when it is a built-in profile name (`fanuc.cps` gives `fanuc`); else
the run refuses. A post with no `serves` serves milling. A machine that is
not a mill refuses (this slice is 3-axis milling). A T number that the
program uses and `--tools` does not state refuses (exit 1, the rule of
#2213). A `.camtools` file refuses by name: its reader is in `camcraft-app`
and it states no T numbers (#4465). The stock is the field engine with the
`auto_cell` of the smallest tool, or `--cell`.

**Cost to change later.** Each law is one module of `crates/simview/src/check/`
with its own pins in `crates/simview/tests/check/`.
