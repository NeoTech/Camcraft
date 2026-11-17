# 2297: give the stock-in-vise test its LocateSpec body

## Decision

`crates/app/tests/stock_in_vise_sim.rs` sets `body: Default::default()`
(`LocateBody::Part`) in its `LocateSpec` literal.

## Why

The test's mates pick faces on the entity `part`. `LocateBody::Stock`
would refuse them: `plan` checks that each mate's part is on the held
body and refuses other entities by name. The part mates are the right
input for this test. The stock follows the part (the one stock-follows
law), so the test still runs on a stock in the vise.

## Dropped

- `LocateBody::Stock` with stock faces as mates. That changes what the
  test asserts (the #2177 turn and simulation checks), so it is a new
  test, not this fix.
