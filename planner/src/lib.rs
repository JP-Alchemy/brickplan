//! BrickPlan planner: a wall spec goes in, a serialized placement plan
//! comes out. All logic lives in plain modules that compile natively, so
//! `cargo test` needs no WASM tooling. The WASM boundary is added on top.

pub mod layout;
pub mod plan;
pub mod sequence;
pub mod spec;

pub use plan::{Plan, PlanStats, plan};
pub use spec::{BrickDims, Opening, PlanError, WallSpec};

#[cfg(test)]
mod tests;
