## #3451 - Measure the spot cone at the part top

**What was decided.** Add an optional `VerifyContext::rim_top_z` upper bound
for the source material. The axial cone check uses this bound for declared
hole rims. A missing bound retains the stock-top reading. Nonfinite bounds
retain the nominal cutter radius. The rule still requires a trusted cone,
an axial path, a boundary context, and declared rims.

The plain operation context supplies `Some(material.top)` in world
coordinates only for `OpParams::Drill`. Other operation kinds supply `None`.
The framed context also supplies `None` and retains the conservative stock
cap. Drill operations cannot tilt, so they do not need a local cap.
The stock cap remains the input to the rapid and collision checks. The
part-solid containment check continues to refuse real wall cuts.

**Why.** Stock can extend above the finished part. Reading the cone at the
stock top can convict a spot that fits inside the hole at the part top.
The maximum source height bounds every hole wall conservatively.

**What was dropped.** Do not use the declared `entry_z` as geometric proof.
A user can set that value below a real wall. Do not change `stock_top_z`:
that would lower the material cap used by other safety checks. A local
height for each hole needs more geometric evidence than this fix requires.
Do not add a local-mesh cap for framed operations. It adds logic that the
affected Drill operation cannot use.

The quick gate found one stale expectation inherited from PR #3472.
`example_catalog::turning_cone_thread_case_posts_the_taper_and_the_thread`
still expected `G01 X0. Z13.5`, while the parent's golden posts `Z12.`.
Update that assertion to `Z12.` under the expected-change rule. Do not skip
the test or revert the parent's kerf correction. No golden changes are
needed. The failed step passes after this correction.

The synthetic regression uses the stock top at Z13 as its R plane. This
keeps generation legal against raw stock. The part top is Z12 and the tip
reaches Z10. The test does not replay the original QA project or its prior
Face operation.

**Overlap decision.** Stack on PR #3472 because both changes touch
`generate::run_job_arms`. The other open changes to this large function
touch different lines. They require a refresh if their merge order demands
one.

**Cost to change later.** Existing callers can retain their default context
and its stock-top behavior. A future local hole cap must prove its bound
against the model in the operation frame. It must retain the real-gouge
and invalid-bound tests.
