## #869 - Air coolant mode, the machine's default coolant, and no codeless coolant in the editor

**Decided.**

1. **`CoolantMode::Air`** (label "Air", stream word `PARAM Coolant air`,
   project word `"air"`). The posts resolve it from the machine entry
   named `Air` or `Air blast` (case-blind; spaces and underscores read
   as hyphens, the #150 spelling law), like Mist. `COOLANT_KINDS`, the
   mounted-machine editor's pick list, gains `Air` between Mist and
   Through-spindle.
2. **Every post that reads the ask handles Air.** The eight generic
   `coolantModes` posts (Fanuc, Fanuc Lathe, Haas, Siemens 840D,
   Heidenhain, Mazak, Doosan DNM, Tormach Lathe) resolve `air` from the
   machine list, or refuse it by name ("the machine states no air
   coolant code"). Tormach Plasma already refuses every ask but Off.
   The Tormach mill post does not read the ask, so the engine's
   existing `coolantModes` refusal covers an Air op there.
3. **`carvera.cps` now declares `coolantModes` and posts the op's ask.**
   It used to take "the first Flood entry, else the first coded entry",
   which posted M8 for a profile that kept the seeded Flood M8 next to
   an Air M7. Now a flood ask takes the Flood entry, failing one a
   shop's own coded name (the generic posts' law). Mist, Air and
   Through-tool take their own entry. Anything else is refused by name.
   **A flood ask on an Air-only Carvera is refused**, not posted as M7.
4. **The machine's default coolant is derived, not stated.**
   `CoolantMode::machine_default(coolants)` is Flood when the machine
   can post a flood ask (a coded Flood entry, a shop's own coded name,
   or no coded entry at all). Otherwise it is the first coded entry's
   mode. `CreateOperation` starts a new op on its setup's machine's
   default, so on the shipped Carvera (only `Air` M7) a new op is Air
   and posts M7 / M9.
5. **The machine file editor never writes a codeless coolant.**
   "Add coolant" takes a name and an M code together and refuses an
   empty code by name. Every coolant row has a "Remove" button, the
   seeded Flood of a blank machine included. A coolant with no code is
   a draft refusal (`coolant <name>: has no M code - enter its code ...
   or remove the coolant`) that blocks Save and Save As. `save()` now
   checks the refusals itself, as `save_as()` already did.
6. **`tormach.cps` flood law aligned.** It also took "the first coded
   entry" for flood, so it would have posted the air code M7 for a
   flood op on an Air-only machine. It now uses the generic flood law
   and refuses by name when the list has only other known kinds.

**Why.** The Prime Directive says never post a wrong M-code. The user's
report says M8 posted on a machine whose coolant is air, and their added
Air M7 was "ignored on save". Air is its own family on Makera machines:
M7 starts the airflow there, but it is not "mist". Naming it lets an op
ask for it and lets a post refuse it honestly. Deriving the default from
the list needs no new machine-file key, so every existing `.mch` and
project file loads unchanged. A machine with both Flood and Air keeps
Flood as its default, and the op picks Air.

**Considered and dropped.**
- *An explicit `tool_station.defaultCoolant` key.* It is more flexible
  ("keep both, default Air"), but it is a new file-format key that needs
  validation, an editor field and parsing. Removing the seeded Flood, or
  picking Air per op, already covers that case. It can be added later
  without breaking the derived rule (a stated key would override it).
- *Mapping Air onto Mist.* The Carvera's M7 is an air blast, and a
  machine can state Mist M7 and Air M10 side by side. Folding them
  together would post the wrong code there.
- *Letting a flood ask fall back to the Carvera's air code.* That is
  what posted M7 for "Flood" before, and with Flood M8 present it
  posted M8 for an op that wanted air. The issue asks for a named
  refusal when the machine has no matching entry.
- *Refusing codeless coolants in the general machine validator.* A
  hand-written `.mch` may name a coolant without a code. The setup sheet
  shows it and the posts stay silent; this is the #150 "operator's own
  valve". Refusing it at load would refuse files that read today. Only
  the editor, which writes files, refuses it.

**Compatibility.** Old project files load unchanged: a missing `coolant`
key is still Flood, and `air` is a new word older builds do not write.
Old `.mch` files load unchanged. **Behaviour change:** a project made
before this change whose ops sit on an Air-only Carvera has those ops at
Flood (the old default). They now get a named refusal ("operation X:
the machine states no flood coolant code - pick the operation's coolant
from the machine's list (Air for the Carvera's air blast ...") instead
of posting M7. The fix is one coolant pick per op. A generic post on a
machine whose only coolant is named "Air" refuses a Flood op the same
way, where before it treated "Air" as the shop's own flood.

**Cost to change later.** Adding an explicit default key is additive.
Re-allowing the Carvera flood-to-air fallback is a five-line change in
`coolantFor`.
