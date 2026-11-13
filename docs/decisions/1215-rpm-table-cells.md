## #1215 - the workholding rpm table asserts all 24 cells

**What was decided.** The walk in `every_machine_dialect_and_section_cell_gives_its_verdict`
now asserts a verdict for every cell. The 9 cells the creation doors refuse
are asserted to be refused at creation (not skipped), with the refusal's
message pinned: turning work on a mill (`none`/`mill` x Work, 3 dialects),
and milling work on a plain lathe (`lathe` x Tool, 3 dialects). Every other
cell must be created. A new pin, `a_milling_section_on_a_plain_lathe_is_judged_by_the_chuck`,
reaches the `work_speed` arm `(ToolSpindle, Refused)` on a plain lathe through
`program_problems_by`: the op is made on a mill, then the machine becomes a
plain lathe. Over the chuck it is named; at the limit it is clean.

**Why.** A refused cell must say why it is refused, so a later change that makes
one creatable fails red rather than going unjudged. The plain-lathe arm had no pin.

**Observed post-time behaviour (not a rule change).** Through `post()` the same
section is refused earlier by the family clause (`posting_family.rs`: "a Face
operation cannot run on its setup's ... machine"), before the workholding
clause runs, for both `fanuc.cps` and `haas.cps`. So the `work_speed` arm is
reachable only by calling `program_problems_by` directly. That is a refusal either
way, so it is safe; the test pins both.

**Considered and dropped.** Asserting the 9 cells by a generic "refused" check
without the message (too weak: a different refusal would pass). Adding a
product-code change to make the post reach the limit first (out of scope,
test-only).

**Cost to change later.** Low: the message substrings are two constants in the test.

**Observed wording, not changed.** For a `none` (no machine mounted) setup the
creation refusal reads "the setup's machine is a mill". It is a refusal and
therefore safe; the wording is a candidate for a later polish issue.
