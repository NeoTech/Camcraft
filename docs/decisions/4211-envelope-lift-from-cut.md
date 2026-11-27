## #4211 - The envelope vertical-rapid sweep keeps the lift-out-of-the-kerf rule

#4108 (#4062) added a finished-part sweep of the vertical rapids for
Envelope paths. That sweep judged the whole cutter at the start pose of
each rapid. An upward retract that starts at the end of a cut starts in the
pose of that cut. The finish envelope judges that pose with the finish
tolerance, so the cutter can touch the model there by up to that tolerance.
The sweep read this contact as a rapid in the part, and each 3D finish hop
was refused (`finish3d_hop_height`, motion).

Chosen: an upward vertical rapid that starts where a cut ended uses the
same rule as `stack_hits` (the Area / Boundary reading). The shank and
holder are swept over the full rapid. The cutting edge is judged by the
column that it rises into above its own top, at its largest radius. The
start pose stays with the law of the cut.

Why: the #4062 fault is an upward rapid through a roof. The rise column
of the edge finds that roof (the canopy tests stay red-to-refused). The
contact at the start pose is not new travel of the tool.

Dropped:
- Looser allowance (the finish tolerance) for all vertical rapids: it
  also loosens the descents and the rapids that do not start at a cut.
- A change of the motion tests: their hops are correct; the finding was
  false.
