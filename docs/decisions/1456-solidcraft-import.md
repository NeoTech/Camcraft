## #1456 - Solidcraft `.sc` import: the kernel-side reader first

**Decided:** `camcraft_kernel::solidcraft::import_solidcraft` reads a
Solidcraft design's cached final body (`cache/model.json` +
`cache/bodies.bin`) into the same `StepSolid` the solidcore STEP reader
hands out, so the machining mesh (#940) takes it unchanged. The app wiring
(Model > Open filter, drop list, MCP tool) is not in this PR: no crate
depends on `camcraft-kernel` yet (the app still imports through truck, #917
is open), and the first app -> kernel edge belongs to that switch, not to a
new file format. The PR is `Refs #1456`; the issue stays open for the
wiring.

**Read only what is provably this document's.** The cache must name a
format we read (`solidcraft-model-cache/8`, the maintainer's sample, and
`/10`, the published source; every other version is refused - the format
has changed ten times), the producer `solidcraft-truck-adapter/4`, the
kernel `solidcore`, and SHA-256 digests that match `document.json` (else
"stale") and `cache/bodies.bin`. The final state is Solidcraft's own rule
(last feature with `skipped == false`). Refused by name and not guessed:
several bodies, a placed copy (assembly instance), a mesh body (`SCMESH01`),
a design in units other than mm (the published docs do not say how such a
design stores its bodies), a cache-less design ("save it with the model
cache on").

**New third-party dependency, needs the maintainer's yes:** `flate2` 1.1
for the container's deflate entries. It is already in `Cargo.lock` (png,
tiff, usvg and vtkio pull it), pure Rust through miniz_oxide, so nothing new
is downloaded or built. `serde_json` is the workspace's own. Considered and
dropped: the `zip` crate (a new tree of crates, and its extract-to-path API
is the risk the container rules guard against) and `sha2` (new crates for
one function): the ZIP directory reader (APPNOTE.TXT, stored and deflate
only, no ZIP64, no multi-disk, 10,000 entries) and SHA-256 (FIPS 180-4,
pinned by the standard's test vectors) are written here, ~300 lines.

**No new camcraft-* edge.** The reader lives in `camcraft-kernel`, which
already holds solidcore and solidcore-record (the body codec it decodes
with); `build_record` was split out of the STEP reader so both share the
same checks.

**Fixture:** the maintainer's sample is `testdata/models/
solidcraft_for_beginners.sc` (5 MB, beside the STEP fixtures), marked
binary in `.gitattributes`. Corrupted copies are built in the tests, never
committed.

**Cost to change later:** a new cache format version is one entry in
`CACHE_FORMATS` after reading its change; multi-body import needs the
app's multi-model path; other units need Solidcraft to document them.
