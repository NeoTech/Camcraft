## #3377 - Test the two other origin-axes switches

**Decided:** Add one behavioral test to the existing Browser view-switch suite and one to the viewport Display suite. Use the seeded shell harness. Click the Browser's real checkbox by its accessible label. Click the actual viewport axes button at its live HUD anchor. In each test, check that axes change from on to off, Grid stays on, and the production `ground_grid_lines` result remains nonempty.

**Why:** The Display menu already has coverage. The other two controls need their own UI dispatch checks and the actual grid generator, not another copy of its visibility condition.

**Considered and dropped:** Direct field writes or calls to the button action alone do not test control dispatch. A new integration binary or test-only production seam is not needed. Renderer changes and pixel goldens are outside this test-only change. Keep the existing Display-menu and reverse grid-off/axes-on tests unchanged.

**Cost to change later:** Only these tests need to follow a changed control label or HUD anchor. No production behavior or dependency changes.
