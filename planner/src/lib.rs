//! BrickPlan planner: a wall spec goes in, a serialized placement plan
//! comes out. All logic lives in plain modules that compile natively, so
//! `cargo test` needs no WASM tooling. The WASM boundary is added on top.

pub mod layout;
pub mod plan;
pub mod sequence;
pub mod spec;

pub use layout::{BrickKind, Placement, WallSide};
pub use plan::{Plan, PlanStats, plan};
pub use spec::{BrickDims, Opening, PlanError, WallSpec};

use wasm_bindgen::prelude::wasm_bindgen;

/// The envelope crossing the WASM boundary. Errors travel as data
/// (`{ ok: Plan } | { err: PlanError }`) instead of being thrown across
/// the FFI: to the UI an invalid spec is a normal state to render, not
/// an exception, and the envelope keeps the wire format explicit.
#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum PlanResult {
    Ok { ok: Plan },
    Err { err: PlanError },
}

/// Plan a wall from a JS `WallSpec` object. The only WASM export; all
/// logic stays in plain modules that compile and test natively.
#[wasm_bindgen]
pub fn plan_wall(spec: wasm_bindgen::JsValue) -> wasm_bindgen::JsValue {
    let result = match serde_wasm_bindgen::from_value::<WallSpec>(spec) {
        Ok(spec) => match plan::plan(spec) {
            Ok(plan) => PlanResult::Ok { ok: plan },
            Err(err) => PlanResult::Err { err },
        },
        Err(parse) => PlanResult::Err {
            err: PlanError::MalformedSpec {
                message: parse.to_string(),
            },
        },
    };
    // The json-compatible serializer keeps the JS-side shape identical to
    // serde_json (e.g. a missing opening becomes null, not undefined), so
    // the snapshot fixture in the test suite also guards this boundary.
    // Our types always serialize; a failure here is a planner regression,
    // so abort loudly.
    use serde::Serialize;
    result
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .expect("PlanResult serializes to JsValue")
}

#[cfg(test)]
mod tests;
