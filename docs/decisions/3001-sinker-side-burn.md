## #3001 - Post explicit Side Burn moves through the machine dialect

**Decided:** Add the optional `edm.codes.side` template to the shipped
sinker post and the machine editor's code list. Require `{x}`, `{y}`,
`{z}`, and `{f}` in the template. Fill them with the absolute endpoint
in the work offset and the feed, in the document's units. Keep all EDM
codes in the machine file. Add no production code for a real control.

**Why:** The generated Side Burn already states its downward plunge,
lateral feed, return feed, and rapid retract as explicit CL moves.
Post the plunge with `depth`. Post each lateral feed with `side`.
Preserve the direction and amount that the generator states. Accept only
feeds along one X or Y axis at constant Z. Refuse missing dialect data,
incomplete coordinates, invalid feed, and unsupported moves by name.
The existing straight burn keeps its current depth code and needs no
`side` entry. The tests use an invented dialect. No real EDM control was
exercised.

**Dropped:** Do not choose an implicit lateral code or reuse `depth` for
a lateral feed. Either choice can omit the required movement. Do not
replace explicit moves with a control orbit. Do not add general XYZ or
diagonal servo feeds; the generated Side Burn does not need them.

**Cost to change later:** A real control needs verified machine data from
its public manual and inspection of its NC output. Additional motion
types need a separate safety rule and pinning tests. Existing machine
files remain valid and refuse Side Burn until they state `side`.

### Bore fixture and editor coverage

**Decided:** Refresh the Linux bore path pin for the current imported
6,228-facet fixture. Pin the mesh size and exact cutting levels. Keep
the verification, fallback, and time-budget checks. Pin `Code side` by
its literal label in the existing machine editor test.

**Why:** PR #2904 replaced the bore fixture. Its old Linux path counts
describe the previous mesh. The explicit UI label assertion does not
depend on the production code-role list.

**Dropped:** Do not change the production planner to recover old fixture
counts. Do not remove the exact path pin or weaken the safety checks.

### Aligned-setup sinker test: the refusal names the missing code

**Decided:** Change the expected refusal text of
`sinker_electrode_burn_matches_baked_and_side_burn_stays_refused`
(`crates/app/tests/setup_align_non_mill.rs`). The fixture dialect
`SINKER EDM SYNTHETIC DIALECT (FIXTURE)` states no `edm.codes.side`.
The post still refuses its Side Burn and writes no file. The refusal now
names the missing code: "states no edm.codes.side". The test also
asserts that the refusal does not name `edm.codes.depth`, so the plain
burn SB1 in the same program still posts.

**Why:** The old text "a Sinker Side Burn moves the electrode sideways"
described a post that wrote no lateral move at all. This PR replaces
that refusal with one that names the machine data to add. The test's
meaning stays: a machine without a lateral servo template refuses the
Side Burn by name. The straight burn half of the test (the aligned and
baked programs are equal) is unchanged and passes.

**Dropped:** Do not add `edm.codes.side` to the fixture dialect. That
would make the test post a Side Burn and drop the pin that a dialect
without the template refuses it. Do not weaken the assert to "any
refusal".
