## #4062 - Sweep envelope-path vertical rapids against the finished part

Use the existing finished-part solid sweep for straight vertical rapids in
Envelope mode. That mode skips the ordinary solid law because its cuts are
judged by the finish envelope; the envelope scan omits rapids. Reuse the
named RapidInPart verdict and the actual cutter, shank and holder atoms.
The stock-column descent check stays separate. Do not extrapolate a stock
column upward through the tool's cut kerf.

Area, Boundary and Undercut already run the full finished-part solid law
when the verifier has a model. Mark and chamfer allowances and
radial/rotary/turning verification keep their separate laws. The synthetic
tests use a thin disc (T-slot/keyseat-style geometry) and a dovetail
frustum; this does not add a separate saw definition or operation.

Turning paths in Envelope mode carry the insert's nose center and use the
existing revolved verifier. The rapid sweep models the milling tool's tip
and axial stack.

Saved milling-envelope operations need a fresh verdict before posting. A scoped
generation stamp makes those caches stale until the operations are regenerated.
