## #530 - EDM machine kinds and the `.mch` edm section

**Decided:**
- `Kind::WireEdm` / `Kind::SinkerEdm`, declared only (`wire_edm`, `sinker_edm` and a few synonyms). No axis inventory infers an EDM.
- The required axes are X+Y for wire and Z for sinker. A blank EDM machine gets no spindle and no coolant.
- `Machine.edm` is an optional section with the wire guides, max taper, wire diameters and AWT, and the sinker orbit list (CamCraft's geometry name plus the control's word), max orbit radius and pickup. Every key is optional; absent means not stated; a garbled value refuses the file. The same law is restated in `is_sane` for project files.
- No op family admits an EDM, so every current op is refused at the door on an EDM setup.
- **Post-time family gate:** `op_streams` now re-checks every op's family against its own setup's machine and refuses by name. Before this, an op created on a mill posted in a milling dialect after its setup was remounted onto a lathe or an EDM. The door law only guarded creation.
- `wire-edm.mch` and `sinker-edm.mch` ship as empty templates (no vendor values). They are not in the built-in picker until an EDM op exists (#532 / #541), so nobody is offered a machine that can run nothing.

**Why:** the #290 design (D1, D4, D5). The post-time gate is the Prime Directive: a program must never post in a dialect its steel does not speak.

**Considered and dropped:**
- Inferring an EDM from U/V letters: a mill can name a linear axis U.
- Listing the templates in the picker now: a dead end for a user.
- The condition table and codes in this issue: they are #531 and #533.

**Cost to change later:** low. Adding keys is additive, and the gate is one loop.
