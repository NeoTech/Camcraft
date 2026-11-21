## #3182 Heidenhain Klartext controller profiles

Use one Camcraft Klartext post. The machine file records the installed model
and exact NC software in `controller.identity`, and the tool-change procedure
in `tooling.tool_change`. Camcraft resolves those facts to a supported language
profile. The first profile covers the three-axis Fehlmann Picomax 55 with an
iTNC 530 running `340490-03`. Selecting a newer release needs a documented
compatibility decision and a test. A release number alone cannot establish
which options a particular machine has.

Keep the program syntax in the post. Embedding a copy of each cycle and word
in every machine file would require machine owners to maintain the dialect.
Using a separate post for every release would duplicate common geometry and
tool-change checks. A later release can share this profile if its documented
forms and machine requirements match, or supply a separately tested form
for each difference.

Limit the first profile to a single metric setup, three linear axes, manual
tool changes, straight moves, and planar XY arcs. A real stock bottom
defines the blank even for shallow cuts. Unimplemented operations produce
a diagnostic. The software checks the emitted program and the refusal paths.
Commissioning requires separate checks on the target control and machine.
The manual states that cycle 247 does not activate a preset during Test Run.
[HEIDENHAIN iTNC 530 conversational programming manual, 533 190-22](https://content.heidenhain.de/doku/tnc_guide/pdf_files/iTNC530/34049x-neu/en/bhb.pdf)

Require the named Picomax installation and an explicit positive spindle range.
Another mill with the same control software can have different tool-change
interlocks. Read spindle limits from the machine definition so a restricted
installation can state a lower maximum. The Picomax file states rapid rates.
Cutting-feed and machine-coordinate travel limits require independent checks.

Keep each tool number associated with one tool name, type, diameter, and
corner radius for the whole program. A standalone intermediate file can
give different tools the same number. Refuse the job before saving it,
including when a previous tool number returns after a tool change.

The post opts in to a hard refusal for all engine and script warnings, including the standalone CLI. Keeping only script-level refusals could release a program using the previously accepted preset when the engine rejects a later work offset. Other posts retain their existing warning handling. Informational accuracy notes remain in the program header.
