## #2855 - Catalog metadata and operation example links

**Decided.** Each catalog case has a required `case.toml`. `kinds` uses the
canonical display names read from the bounded `KindInfo::ALL` source table.
`machine` and `expect` are required descriptive strings, not executable checks.
The flat TOML format accepts quoted strings and arrays, comments and multiline
arrays. It refuses escapes, unsupported syntax, duplicate or empty values,
unknown fields and unknown kinds.

Every known operation-kind heading gets a deterministic marked Example cases
block, including a placeholder when no case exists. Turning headings use an
explicit alias map where their prose titles differ from registry names. Links
point to the canonical GitHub source folder for each case. The generator
preserves surrounding prose and refuses malformed blocks or missing headings
before writing partial guide updates.

Metadata and links are checked across the whole catalog even with `--case`.
The default and rebuild commands regenerate the lists. Screenshot-only and
PDF-only commands leave the guide unchanged. `--check` validates the metadata
and links without writing, before building the CLI. Existing NC rebuilding and
byte-for-byte parity checks stay unchanged.

**Why.** A separate metadata file keeps case READMEs readable and gives the
catalog one source for guide links. Reading the registry source avoids a second
kind list and an app dependency in the small process tool. Early diagnostics
name metadata and link problems before an expensive CLI build. Validating all
links avoids making a filtered case command silently accept broken guide pages.

**Considered and dropped.** Embedding machine-readable fields in READMEs would
mix prose with tooling input. A new app dependency or TOML dependency would
change Cargo dependencies for a small flat schema. A separate links-only CLI
would add another workflow when the existing catalog commands can generate and
check the lists. Treating `machine` or `expect` as executable assertions would
duplicate the build script and its verification checks.

**Cost to change later.** The metadata reader, guide generator and turning alias
map are isolated in xtask. Supporting richer TOML later requires extending or
replacing the reader and its tests; existing flat metadata remains valid.
Changing link destinations requires regenerating the marked guide blocks.
