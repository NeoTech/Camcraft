# 4152 - Guard probe measure check with task parameter and filter spaced comment keywords

- **Chosen:**
  1. Guard the probe tolerance measure check (`M2`) in `assets/posts/linuxcnc.cps` with `[[#<_task> EQ 1] AND [ABS[#dev] GT tol]]`.
  2. Strip whitespace before testing comment strings against `ACTIVE` active-comment keywords, and add `PYRELOAD` to `ACTIVE`.
  3. Add pinning tests in `crates/post/tests/linuxcnc_post.rs`.
- **Why:**
  1. During backplot preview in LinuxCNC, `#<_task>` is 0 and measured probe values are simulated or zero. Without the `#<_task> EQ 1` guard, a large deviation triggers `M2` and stops the toolpath preview early.
  2. LinuxCNC permits whitespace and lower-case letters in active comment commands like `(MSG, ...)` and `(PYRELOAD)`. Stripping whitespace in the comment test ensures user comments like `(M S G)` stay inert with a `- ` prefix, while regular words like `Message` or `Python` stay unchanged.
- **Dropped:**
  Guarding the rotation check at line 1088 with `#<_task>`: that check tests active coordinate system rotation (`#5210 NE 0`), not a measured probe deviation, and is pinned across both Tormach and LinuxCNC by `probe_rotated_wcs.rs`.
