//! The `stock_carry` suite of `camcraft-sim` (#3188): the machined body carried between setups: lattice and side-on turns, yaw, prior stock.
//! Provenance: our own test layout (AGENTS.md "New tests join a directory suite", #919); each
//! module keeps its own provenance and Gate map.

mod carried_boundary;
mod carried_stock;
mod carry_engine_agreement;
mod prior_columns;
mod stock_carry;
mod stock_carry_side;
mod stock_carry_side_exact;
mod stock_carry_side_limits;
mod stock_carry_side_replay;
mod stock_rotation;
