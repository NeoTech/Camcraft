## #4106 - release site tests red on v0.27.0

**Decided:** fix the guide, not the checks. Copy the two new icons and the
icon notice from `assets/ui/`, add a Rotary Flow section and its picture,
and point the `New tool…` label of the two 4-axis tutorial tests at
`crates/ui/src/egui_app/tool_library_dialog/list.rs`, where the button
string now is.

**Why:** each red test reported a true drift between the shipped app and
the site. The moved UI file is a stale path in the test map, so the path
changes and the check stays as strong as before.

**Dropped:** removing the `New tool…` row from the label map (weaker
check); a regex search of the whole UI crate for the label (it would
accept a string from any file).

**Cost to change later:** none; site files only. The same site commit goes
on `site/v0.27.0`, so the 0.27.0 docs can deploy with a `site_only`
dispatch.
