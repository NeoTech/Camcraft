## #3544 - Detect holes on a stock source names the stock and the models to pick

**What was decided**

A Drill, Bore or Thread operation reads the stock when no model is picked
as its source. When Detect holes finds nothing there, each of its three
"nothing found" refusals now names the stock and adds the fix:

    no drilled holes found in the stock - Z or radial. 'D1' reads the
    stock, not a model: pick a model as the source (source: "disc")

- The place is "the stock" for a stock source and "the source model" for
  each other source.
- The fix names each model of the document as `source: "<model>"`, the
  key of the MCP `add_operation` tool. The models are joined with "or".
- A document with no model gets "import a model and pick it as the
  source".
- A model source keeps its old refusal, word for word.

The rule applies to the three refusals of `detect_holes`: drilled holes
(Drill), Z-axis holes (Bore and internal Thread), and upright round
bosses (external Thread). The words before the place do not change, so
the existing tests and agents that match on them still match.

The helpers are in a new module, `operation/detect_source.rs`, because
`operation/material.rs` is already longer than 400 lines.

**Why**

The issue reports that local models in the MCP benchmark of 2026-11-22
stopped at this refusal. The old text said "the source model", but the
operation read the stock. The new text says what was read and what to
change.

**Considered and dropped**

- Fix only the Drill refusal. Dropped: the Bore and Thread refusals have
  the same defect.
- Name the MCP tool in the fix ("add_operation with source: ..."). Dropped:
  the app's Detect holes button shows the same text, and there the source
  is a pick list. `source: "<model>"` reads in both places.

**Cost to change later**

Low. The wording is in one module. The MCP test
`detect_holes_on_a_stock_source_names_the_stock_and_the_models_to_pick`
pins it.
