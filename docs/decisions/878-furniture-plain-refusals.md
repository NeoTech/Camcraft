# 878 - refusals say what a setup owns in plain words

- Chose: the five refusal strings that said "furniture" now name what the
  thing is (a setup's stock, work offset or fixtures; a name reserved for a
  setup's stock and work offsets). Wording taken from the issue's suggestions,
  checked against each site's condition so the refusal stays true.
- Considered and dropped: renaming the internal `is_furniture_*` identifiers
  (identifiers stay; the issue scopes this to user-visible text).
- Guard: `setup_refusals_never_say_furniture` in `crates/app/tests/setups.rs`,
  plus a message check in `crates/app/tests/repair_model.rs`.
