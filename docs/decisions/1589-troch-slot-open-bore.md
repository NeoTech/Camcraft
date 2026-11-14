## #1589 - The stand-alone trochoidal slot opens its bore on a band wider than the tool radius

**What was decided.** `plan_trochoidal_slot` asks the shared lane planner
for the opened bore (`trochoid_bore::bore` with `open: true`, from #1509)
when the loop band's half-width `b` is more than the tool radius `r`. When
`b <= r`, it keeps the plain bore, so those slots post byte-identically.

**Why.** A helix of radius `b` clears only the ring `b +- r`. When `b > r`,
a post of radius `b - r` stays at the start at full depth. No front arc
reaches it, and the verifier does not flag uncut material. The opened bore
already exists and is read by the same engagement measurement. The 2D
Adaptive trochoidal method uses it, and #1509 pinned it.

**Considered and dropped.**
- Open the bore on every band (`open: true` always). This narrows the
  helix to half the tool radius also when `b` is between `0.5 r` and `r`.
  That changes the output of slots that leave no post, including the
  golden `trochoid_slot_small.cl`, for no gain.
- A refusal or a note for `b > r`. That leaves the defect to the user,
  and the fix is already in the tree.

**Cost to change later.** The threshold is one comparison in
`plan_trochoidal_slot`. A note: the opened helix is narrower (half the
tool radius), so it needs more turns for the same stepdown. A very deep
stepdown on a wide slot can now hit the helix turn cap, which is refused
by name ("half the tool radius - use a smaller stepdown").
