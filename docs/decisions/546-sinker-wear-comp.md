## #546 - Sinker EDM: electrode wear compensation

**Decided:**
- **The rate is an end wear ratio**: the electrode length lost per unit of depth burned into the work. A layer whose nominal floor lies `d` below the work top has its tip programmed `rate * d` deeper. The deepest programmed tip is therefore the stated floor plus the gap, less `rate` times the stated depth into the work. The generator checks every layer move against that bound and refuses an over-deep move by name.
- **Where a rate comes from.** A condition's optional `wear` in the `.mch` `edm.conditions[]` (#531's table, dimensionless, not unit-scaled), or a wear measured on the op between burns: `wear_lost` mm lost over `wear_over` mm burned. A stated measurement wins over the condition, because it is this electrode's own reading. With neither stated the burn is **not compensated**. The path carries the note `electrode wear: not compensated (no wear rate stated)` and the op page shows the same line. No fixed ratio is assumed and no vendor wear table ships (#290 D4).
- **The law of a rate:** a finite ratio in [0, 1). A non-finite, negative, or >= 1 rate is refused by name. A condition's rate is refused when the machine file is read (`parse_mch`, so `LoadMachine` and the project reader). An op's measured rate is refused by `CreateOperation` / `EditOperationParams` through a new by-name door, `OpParams::named_refusal`, which runs ahead of the generic "not usable". A half-stated measurement, a negative or non-finite one, and a zero or negative layer step are refused by name too.
- **Layers:** an optional `layer` step on the op. Blank burns the whole depth as one layer, which gives exactly #541's moves. The last layer is clamped to the stated depth, and more than 10 000 layers is refused by name. Layers are consecutive feed moves with no retract between them.
- **Electrode length:** with compensation the length law reads the programmed (deeper) tip. A compensated burn that would bury the holder is refused by name.
- **The stock sim judges the worn electrode.** `plan_program` maps each programmed tip below the work back to where the worn electrode burns it (`sinker_wear::worn_tip`, the exact inverse of the compensation). A compensated burn to the cavity's own floor therefore gouges nothing, and a burn whose nominal floor passes the model is still convicted.
- **A wrong rate is not caught.** The rate is per mm of cavity depth below the work top, not per mm of tip travel. A misstated rate over- or under-burns by (stated - true) x depth, and the stock sim judges the burn at the stated rate, so it cannot catch this: the rate is the user's measured data. `wear` is a sinker-only condition field; a wire machine stating it is refused by name (review of PR #1157).
- **Staleness:** the op digest adds the layer step and the measured wear as tagged bytes, only when stated. The stamp adds the condition's rate, only when stated. An existing burn's digests are unchanged.
- **File and MCP:** three additive optional keys on the Sinker Burn DTO (`layer`, `wear_lost`, `wear_over`). Older files load unchanged and write no new keys. MCP drives them through the existing params JSON door (`add_operation` / `edit_operation`), so no new command and no new tool are needed. This is pinned in `sinker_wear.rs`.

**Why:** the issue's "Done means". The compensation moves posted Z, so it has to be bounded, refused by name and verified (Prime Directive). The sim has to judge the burn it compensates, otherwise every correct compensated burn would read as a gouge.

**Considered and dropped:**
- The `d / (1 - r)` model (wear read per unit of electrode advance). The ratio users measure is length lost over the cavity depth reached, so `r * d` matches the measurement. Both refuse r >= 1.
- An op-level rate override as a bare number. A measured pair (lost over burned) is what a user reads between burns, and it keeps the provenance in the note.
- A retract between layers to let the user measure. That is a post or dialect concern (#542). The CL stays a stepped plunge.
- A sim that cuts the unworn electrode. It would convict every correct compensated burn by `r * d`.
- Assuming a default rate when none is stated. It would be a silent guess.

**Cost to change later:** low. The math is one module (`motion::sinker_wear`). The op keys are additive. The sim mapping is a single function.

### 2026-11-13 - merge with orbit and side burn

Merging main brought the sinker orbit (#544) and the Sinker Side Burn (#545) onto the same `SinkerBurnOp` this PR extends. The author decided how they combine (the maintainer's merge session stopped on it as a safety choice):

**Chosen:**
- **The orbit runs at the worn tip.** `sinker_wear::burn_compensated` expands the orbit after each layer's plunge, at that layer's compensated Z. Every orbit move sits at a tip the deepest-compensated-tip bound has just admitted, so wear never lets an orbit cut past the finished floor. The electrode-length law reads the compensated (deeper) tip, orbit or not. Main's nominal refusals (`sinker::check`, keyed on `tip = z + gap`) still run first and all survive. With no rate and one layer, the moves are exactly main's `burn_to_depth` (pinned in motion and app tests); the CL goldens are unchanged.
- **A Side Burn with wear is refused by name for now:** "electrode wear compensation is not supported on a Sinker Side Burn yet - clear the wear rate", and the same for a layer step. The refusal lives in `SinkerSideBurnOp::refusal`, so it is read at create, edit, paste and change-kind (`sinker_gate`), the project reader (`named_refusal`), and the generator (`generate_sinker_side`). A condition's stated `wear` is not applied to a side burn: its plunge stays uncompensated as on main, which can only under-burn, never over-burn. Follow-up: #1340.
- **The `sinker-wear` stamp term** stays beside main's other sinker digests (`sinker-gap`, `sinker-side-gap`) in `current_stamp`, after main's `stamp_tool` move, written only when a condition states a rate.
- **The sim reads both ends of a move as the worn electrode.** `sinker_wear_op::as_burned` now maps each move's `from` Z as well as its `to` Z through `worn_tip`. The box atom sweeps `from` -> `to`; a plunge starts above the work (unmapped either way), but an orbit's level move starts at the programmed tip, so mapping `to` alone burned the orbit `rate * d` past the floor in the sim and convicted a correct worn orbit as a gouge.
- **Keep both elsewhere:** `SinkerBurnOp` holds main's `orbit` and this PR's `layer` / `wear_lost` / `wear_over`; `is_sane` checks both the orbit and the compensation. The DTO keeps main's key order (orbit first, then the three additive wear keys). The params digest writes the orbit bytes, then the tagged wear bytes, then the side's, each only when stated, so no existing digest moves.

**Why:** the orbit sweeps the floor of the cavity; at the nominal tip it would land `rate * d` short of the floor the plunge reaches, and judging it against the nominal tip would let a worn orbit pass a bound it no longer keeps. Refusing wear on a side burn is safer than guessing how side wear changes the allowance law.

**Considered and dropped:**
- An orbit only at the last layer. Each layer's floor would then be left uncut to the orbit's radius until the end, and the sim would judge a different cavity than a real layered orbit burn.
- An orbit at the nominal tip. It under-burns the floor and keys the bound on the wrong tip.
- Applying the wear to a side burn's plunge only. The lateral move's allowance would then be judged against a burn it does not describe.
