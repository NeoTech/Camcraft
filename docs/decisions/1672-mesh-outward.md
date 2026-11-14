## #1672 - Turn closed inward STL and OBJ bodies outward

Chosen: reuse the existing STEP per-body orientation helper in both buffered mesh imports and the streaming STL load path. Return its note through Graded so the existing command and status-line plumbing reports the change. Keep the helper filename to avoid an unnecessary module rename.

Why: the helper already distinguishes closed, consistently wound negative-volume bodies from open, mixed, non-manifold and zero-volume bodies. It can correct individual bodies without letting a larger outward body hide an inward one.

Dropped: refusing every inward body would require users to repair a winding that can be corrected deterministically. A whole-mesh signed-volume flip would mishandle multiple bodies. Changing the pencil and deburr generators would duplicate an import concern and broaden this fix.

Cost to change later: replace the two dispatcher normalization calls if import policy changes. The pins cover both formats and the separate streaming STL route. Body classification and its limits remain the existing winding policy.
