# 878 - refusals say what a setup owns in plain words

- Chose: the refusals that said "furniture" now say what the thing is.
  Wording per site, checked against each site's condition:
  - repair_model / update_model (`!model::is_model`): "'{name}' is not a model -
    it is a setup's stock or work offset, a setup, a tool, an operation or
    other CAM data". `is_model` also excludes tools, operations, setups and
    program entries, so the first draft ("part of a setup ... fixtures") was
    false for them (review of #886). Fixtures never reach this branch.
  - fixture.rs TagFixture (same `is_model` condition): aligned to the same
    sentence, with "so it cannot be marked as a fixture".
  - setup.rs name '/' refusal: "names with '/' are reserved for a setup's own
    stock and work offset" (the check refuses every '/').
  - setup.rs and tool.rs reserved names: "is a name reserved for a setup's
    stock and work offsets".
- Considered and dropped: renaming the internal `is_furniture_*` identifiers
  (identifiers stay; the issue scopes this to user-visible text).
- Guards: `setup_refusals_never_say_furniture` (tests/setups.rs); in
  tests/repair_model.rs, `tools_and_operations_are_refused_as_not_models_not_setup_parts`
  checks a tool and an operation through RepairModel and UpdateModel.
