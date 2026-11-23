## #2604 - Machine tool reservations

**Decided:** A machine can state `tooling.reserved` entries with a `number`,
a `name`, or both, and `use: "probe"` or `use: "tool_setter"`. Numbers must
be integers in 1–99999. Zero is not a tool. Malformed entries and duplicate
selectors refuse by name. Name comparisons ignore ASCII case.

New tools use the active machine's reservations. Edits check the machines
that use the tool. Existing tools keep their numbers. A probe takes its
declared pocket; an occupied probe pocket refuses. There is no tool-setter
tool kind, so neither cutters nor probes can take a setter reservation.
No shipped machine file reserves a pocket yet. A machine reserves only
what its file states. A shipped file states a reservation only where a
public manual of the maker names the pocket, and the file names that
document.

Existing commands and their MCP tools share the guard. Verification and
posting refuse unsafe assignments, including old documents. The standalone
post CLI checks the stream before it writes output. It keeps the physical
tool selection across sections and requires a fresh kind for each new tool.
A missing selector or required kind refuses. The kind can precede or follow
the tool selection. Allocation uses a hash set. CLI reservation indexes are
built once, so these checks do not repeat full scans.

**Why:** A tool call must not select a probe or setter for cutting. Machine
data states those identities; a guessed convention cannot establish them.

**Dropped:** Silent renumbering could change an existing machine assignment.
Inferring T1 for other manufacturers could reserve a real cutting tool.
T99 in the 18 Tormach mill and router files was dropped at review: no
public Tormach document that names T99 as the probe pocket was cited,
and the earlier #362 research calls it a convention of the probing
screens only. A follow-up adds it with the cited document.
A new command would duplicate the existing edit path. A reserved name does
not select a controller dialect; this change does not alter emitted syntax
or infer name-based tool calls.

**Cost to change later:** More reservation uses need explicit tool-kind
rules and tests. Controller name-based calls need a separate post change.
Project version 78 marks the stored `reserved_tools` field. An older
reader refuses the file. It does not drop the reservation and post a
cutting tool at a reserved pocket. A v77 file steps up unchanged.
PR #3485 merged first and took version 77 for the setup `order` (#3447),
so this PR takes 78. The step is a breaking change of the project file:
the commit has a `BREAKING CHANGE:` footer. Changing the stored shape needs a new version step. Existing
machine assignments must remain explicit during either change.
