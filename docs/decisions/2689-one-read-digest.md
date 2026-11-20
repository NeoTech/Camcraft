## #2689 - A linked model's digest is of the bytes the import parsed

**What was decided.**

- `ImportModel::within` reads the file once
  (`geometry/import/sourced.rs`, `load_model_open_sourced`). A buffered
  format (OBJ, STEP, `.sc`) hashes the buffer it parses. STL keeps its
  streaming parse (the M5.3 E law: the file never coexists with its
  triangles) and is hashed as the stream reads it, through a reader that
  hashes every byte it hands out. The ASCII path rewinds to byte 0 once
  after the sniff; the rewind starts the hash again. Any other seek is
  refused, so the digest is always of one pass. Bytes after the last
  binary facet, if the file grew, are read into the same pass.
- So a file that changes during the import records the digest of the
  bytes that made the mesh. The link check then sees the newer file as
  changed and imports it again. Before, the digest came from a second
  read after the parse, and a change between the two reads was missed.
- The kernel's SHA-256 takes bytes in pieces (`Sha256`, `update`,
  `finish_hex`); `hex_digest` is that over one piece. Pinned against the
  one-shot digest for every split around the padding edges.
- The link check (`model_source::read`) hashes the file in 1 MiB pieces.
  Before, it read the whole file into memory every time a linked file
  changed, which for a large STL broke the same memory law.
- The browser build is unchanged: it records no link.

**Why.** Found in the review of #2563 (#2689). Solidcraft's Send to
CamCraft (tangency-labs/solidcraft#18936) saves the design while CamCraft
may still be importing the previous save; that change must not be lost.

**Considered and dropped.**

- Read STL into memory like the other formats: one line, but it breaks
  the streaming law for the largest files.
- Compare the file's size and time before and after the parse, and
  record no digest on a change: it would unlink the model, or need a
  marker digest that is not a real one.

**Cost to change later.** Low: one module, one kernel type; the tests in
`crates/app/tests/model_link.rs` and `sha256.rs` pin the behaviour.
