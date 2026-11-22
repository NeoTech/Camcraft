## #2874 - Probe before Face catalog case

**Decided:** Add one small case with Probe followed by Face. Use an original
100 x 60 x 12 mm block in 100 x 60 x 13 mm stock. Probe the nominal stock top
at Z13 with `PRB-0400`, select G54, then face to Z12 with `EM-1000-4FL`.
Use the default 3-axis mill and the generic Fanuc post. Embed the model in
the project. Keep the two catalog rebuild checks and pin the operation order
and the probe cycle before the first cut.

**Why:** A plain block makes the sequence and the 1 mm facing allowance easy
to inspect. Existing tools and the default machine keep the case focused on
the Probe-before-Face workflow. The embedded mesh keeps the project portable.

**Dropped:** More complex geometry, other probing modes, and machine changes.
They do not help show this sequence and would add unrelated setup work.
This case uses nominal stock contact; it does not simulate a measured offset.

**Cost to change later:** Change the generator or the script, regenerate the
case, and update its program pin and explanation. No new dependency or
production behavior is required.
