## #2302 - the rest group UI test follows the Zigzag rest decision of #1490

**What was decided.** Update the UI test
`rest_group_warns_and_refuses_incompatible_strategy_and_predrilled_on_apply`
to the behaviour that PR #2037 merged: Zigzag with rest machining shows no
refusal and Apply commits it. The incompatible-strategy part of the test now
uses Adaptive, which `rest2d::rest_strategy` still refuses by name. The
pre-drilled refusal part is unchanged.

**Why.** PR #2037 made Zigzag legal on purpose
(`docs/decisions/1490-rest-strategies.md`). It added closing boundary passes
against wall cusps and pinned them with coverage, verifier and rapid-motion
tests (`crates/motion/tests/rest_zigzag.rs`). It also changed the app and
motion refusal tests to assert that Zigzag is accepted. The UI test came in
through a different PR and was not updated. Thus the test, not the code, was
wrong. No refusal that guards a wrong toolpath is weakened: Adaptive and
pre-drilled entry stay refused, and the test asserts each of them.

**What was considered and dropped.** Restoring the Zigzag refusal in
`rest_strategy` would reverse a reviewed and proven decision and disable a
supported strategy. Deleting the Zigzag part of the test would lose the
coverage of the strategy-refusal warning on the page.
