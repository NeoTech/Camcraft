## #3449 - list_operation_types names every accepted field

**Decided.** `list_operation_types` finds the fields of a kind with a serde
trace of `ParamsDto` (`crates/app/src/mcp/kind_fields/trace.rs`). The trace
drives the DTO's own `Deserialize` with a deserializer that records each
struct's field names, each enum's variant names and each `Option`. A field
that the defaults do not write gets `"optional": true`. An enum field gets
`"values"`. An object or array field gets a short `"shape"` text. The
existing hand rows (break_through, the Wire Contour extras, cavity_outline,
minimum_cone_radius) stay, because they give real defaults. A second
small trace (`kind_fields/written.rs`) records the keys that the kind's
params can write. A key that the type reads but never writes
(`#[serde(skip_serializing)]`, for example the legacy wire `start` that
the load migrates into `picks`) is not offered.

**Why.** One rule for all kinds, read from the types themselves: a field
that a params struct gains is listed without a hand edit (the issue's Done
clause). No new dependency: `serde` is already a dependency of the crate.

**Limit.** Serde names no keys for a struct with `#[serde(flatten)]`. Two
kinds have one: `thread` (flattens `ThreadPassesDto`) and
`sinker_side_burn` (flattens `SinkerDto`). `kind_fields::flattened` traces
the flattened type by itself and names the thread variant's own optional
keys by hand. A kind whose params the trace cannot read, and that has no
entry there, gets `"fields_incomplete": true`; the test
`list_operation_types_reads_every_kind_completely` fails on it. A new
optional key on the outer thread variant is the one gap that the test does
not see. A type that reads itself (`HeightDto`, untagged and internally
tagged enums) is typed `"any"` when the defaults do not show it.

**Dropped.** `schemars` JSON Schema output: a new dependency, which waits
for the maintainer. A per-kind hand list of optional fields: it goes stale,
which is the defect. Units per field: serde has no unit; not done here.

**Cost to change.** The listing is MCP output only. No project file, no
command and no posted program changes.
