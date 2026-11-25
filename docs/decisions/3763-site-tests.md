## #3763 - Waterjet Profile gets its own guide page, outside the catalog page list

**Decided:** Waterjet Profile is documented on a new page,
`site/content/guide/operations/operations-waterjet.md`, as the family
`waterjet` of `site/tests/test_operation_reference.py`. The page is not
added to `PAGES` in `xtask/src/catalog_links.rs`, and it has no
`catalog:examples` block.

**Why:** the ribbon puts the kind in its own Waterjet group, and the
plasma page is about plasma only. The same site-only fix must go onto the
branch `site/v0.26.0` from the tag, which takes no change outside `site/`.
No catalog case names the kind yet, so the catalog check loses nothing.

**Dropped:** a section on the plasma page (wrong title and machine); an
`xtask` change to list the page (a process change, and not site-only).

**Cost to change:** add the page to `PAGES` when the first waterjet catalog
case arrives; `cargo xtask catalog` then writes the block.
