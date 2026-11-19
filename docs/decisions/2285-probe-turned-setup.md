## #2285 - Read the Probe op's pick in the setup frame

**Chosen:** Read the picked feature's placement in `operation::probe_pick`
through `setup_align::pose_in(world, entity, setup)` instead of
`model::transform_of`, and carry the setup name through the three seed
helpers (`probe_seed`, `probe_from_picks`, `probe_account`) as
`setup: Option<&str>`, the shape #2276 gave the Slot helpers. Add Probe to
the verified aligned-setup mill kinds and to the supported kinds named in
the refusal message. Require a proof: a pick on a box turned about two
axes must seed the touch against the part as the setup holds it, cut the
same path and post the same byte-identical program as a part with that
turn baked into its mesh, while the part as modelled must seed another
touch and cut another path. Add a second test for the safety law: a probe
target written in the model frame in a turned setup drives the stylus
through the part, and the protected-move law convicts it with a
post-blocking warning. Four sentinels that pinned Probe as refused now
name kinds that are still unverified.

**Validation:** The proof test fails with the old seed read: the turned
case seeds the touch on the model top (`the turned seed: [20.0, 15.0,
10.0]`) instead of the turned part's top, and the stylus law convicts the
same path with `Gouge` at moves 3 to 6. It passes with the fix, and the
baked twin posts the byte-identical program. The conviction test holds in
both builds: it pins the safety law that makes the wrong frame visible
rather than silent. The quick gate ran the touched app test binaries and
clippy.

**Why:** Generation already speaks the setup frame: `operation::generate`
passes the setup-frame facets of the source to `probe_op::generate_probe`,
and the stylus law judges the path against those facets. Only the pick's
seed read the model frame. The two reads disagreed, so a Probe in an
aligned setup posted a cycle that sought a surface 10 mm inside or in front
of the real one, with no diagnostic - the silent wrong answer the Prime
Directive forbids. Probe's `KindInfo` sits in `Section::Drilling`, so its
family is Mill: `verified_mill` admits it on `Mill` and `MillTurn`, which
is where probe ops already run.

**Dropped:** Transforming the seed result inside `probe_pick` after the
seed - that would apply the pose twice, because the facets the seed queries
would already carry it. Adding Probe to `verified_turn` - Probe is not a
turning kind, and the lathe voice refusal for it stays. Removing the
refusal before the turned-versus-baked posting proof existed - the safety
floor of #2276. Re-tuning the stylus slack to catch a deep burial - the
protected-move law already convicts the descent from the safe plane, which
crosses the turned part's top face.

**Cost to change later:** A new Probe world read must keep the setup-frame
contract: the facets it queries and the placement it applies must both come
from `setup_align`, and the proof tests must cover each new source or mode.
The verification entry can be withdrawn if that contract cannot be proved;
the refusal then blocks aligned generation by name.
