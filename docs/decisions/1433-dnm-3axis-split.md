## #1433 - the built-in DNM 5700 is split into a plain 3-axis machine and a rotary-table variant

**Decided:** option 2 of the issue. `assets/machines/doosan-dnm-3axis.mch` is the built-in "Doosan DNM 5700": the same catalog figures (#1410's strokes) with no A axis, so the index-swing law has nothing to judge and a 3-axis job posts. `assets/machines/doosan-dnm.mch` keeps the catalog's optional 4th-axis table as the built-in "Doosan DNM 5700 + rotary table" (its `general.model` renamed to match), still without `nose_to_axis`, so every op on it is refused by name until the setup states its own (`SetSetupNoseToAxis` / MCP `set_setup_nose_to_axis`).

**Why:** no published figure for the catalog's rotary table's nose-to-axis distance was found, and a safety figure is never invented (option 1 dropped for now). Option 3 changes the index-swing safety law and is the maintainer's call. The split matches the shipped precedent `Tormach 770MX` / `Tormach 770MX + microARC 4`.

**Old projects:** a project file embeds its mounted machine (`MachineDto`), so a project saved on the old "Doosan DNM 5700" keeps its A table and keeps refusing exactly as before; remounting the built-in "Doosan DNM 5700" gives the 3-axis machine. The A-table file keeps its path, so every test and doc that needs the DNM's A (post tests, conformance, `doosan_dnm_a_state`, `program_units`) reads it unchanged; only the library name moved (`fourth_axis_pose`, `machine.rs`'s catalog pin).

**Considered and dropped:**
- Make `doosan-dnm.mch` the 3-axis file and move the rotary to a new path. Dropped: about a dozen post tests read the A from that path; moving them is churn for no behaviour.
- Keep the rotary variant's model string "Doosan DNM 5700". Dropped: two picker entries would post the same header and read as the same machine.

**Cost to change later:** when a published nose-to-axis figure for the table turns up, state it in `doosan-dnm.mch` (option 1) and flip `vmc_stroke.rs`'s rotary pin.
