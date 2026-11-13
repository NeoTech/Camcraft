## #545 - Sinker EDM: side burning as its own op kind

**Decided:**
- **A separate kind, Sinker Side Burn** (`OpType::SinkerSideBurn`, stored id 42, appended; file key `sinker_side_burn`). Its params are `SinkerSideBurnOp { burn: SinkerBurnOp, direction, amount, undersize }`. The burn's plunge (approach, heights, datum, depth, burn point, feed and the op's **own** condition) is #541's struct embedded whole, so every height, digest, sim and DTO site reads it through one or-pattern and none is duplicated. The embedded `orbit` must stay `None`: a side burn moves along one axis. An orbit is refused by name (orbit with a Sinker Burn).
- **The move:** the burn's approach and plunge (with every #541 refusal, through `sinker::check`), then one Feed of `amount` along +X, -X, +Y or -Y at the floor, the same Feed back to the burn point, and the rapid retract up Z. Every move is explicit CL (#290 D3), and the box atom sweeps it (#290 D7). No new sim law.
- **The allowance law.** The electrode is `undersize` smaller per side than the finished cavity. Sunk under its condition, it leaves each wall one spark gap out, `undersize - gap` short of size. So `allowance = undersize - gap`. A lateral `amount` moves the faced wall out by exactly that amount, and an amount past the allowance is a move into the part. It is refused by name ("... burns into the part - the side's allowance is ..."). A gap larger than the undersize is refused too: the plunge alone would burn past the wall.
- **The undersize is the op's own** (the issue: "its own undersize and condition"), not the condition's `undersize` field (#543 reads that one for rough/finish). The op states what its electrode was made to, and the condition supplies the gap.
- **Validated at command creation** (`commands::operation::gates::sinker_gate`, on create, paste, edit and retype, ahead of `is_sane` so the refusal is by name):
  - always: a non-finite, zero or negative amount; a non-finite or negative undersize; an amount past the undersize (past any allowance, since no gap is negative); an orbit.
  - once a condition is named and the setup is a sinker EDM: an unknown condition (or one with no gap), a gap the undersize can't honour, an amount past the allowance.
  
  An unnamed condition passes creation as a draft (the ribbon's seed names none) and is refused at generation. Off a sinker EDM, the family door refuses with its own words. Generation re-runs the whole law (`sinker_side_op::side_gap`, read by the job, the stamp and the sim plan), so a machine that later re-states a wider gap refuses the stored op by name and marks it stale.
- **The seed** is the burn's draft, +X, amount 0.1 and undersize 0.2. These are editable draft numbers, not any machine's values. The allowance law judges them against the named condition.
- **MCP** goes through the params JSON door (`add_operation` / `edit_operation` with `direction` / `amount` / `undersize`). `list_operation_types` lists the kind from the registry. No new command and no new MCP tool.
- **UI:** only the minimal rows on the existing sinker page's Passes tab (wall, amount, undersize, in place of the orbit rows). The full pages are #548's.

**Why:** #290 D3 and D7 and the Prime Directive. What posts is what the sim saw. A lateral move into the part is a gouge, so it is refused before it exists, and the sim convicts an undersize the part doesn't actually have.

**Considered and dropped:**
- A side mode inside the Sinker Burn. The issue asks for a separate op with its own undersize and condition, and a burn's orbit and a side move don't compose cleanly.
- An arbitrary XY angle. Along a diagonal, a rectangular electrode's two walls each move by `amount * cos`, so "moves the wall by the stated amount" no longer holds for one wall. Several directions are the orbit's vector pattern (#544).
- Reading the condition's own `undersize`. That ties the side burn to #543's rough/finish semantics, and the op's electrode is a user statement.
- Checking against the model's walls in the generator. The stock sim's program verdict already judges the moved wall against the part (a test pins it), so a second geometric law would duplicate it.

**Cost to change later:** low. One motion module (`motion::sinker_side`), one app module (`operation::sinker_side_op`), one appended kind and DTO variant. Adding an angle means replacing the direction enum, plus a file word.
