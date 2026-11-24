## #3633 - Check the rapid between Waterline rings

Use the exact ball sweep from #3627 to check same-level ring links below the
model top. Keep the programmed retract when it clears the model. Use the
programmed clearance when the sweep finds a collision. Check the actual
destination for both plunge and ramp entries.

Always using clearance would lengthen links that already clear the model.
Reusing the sweep keeps the current moves on those links and leaves the
post verifier in place. A future change to the collision test can be made in
the existing sweep without changing the ring planner.
