use super::*;
use crate::PlanResult;
use crate::plan::plan;
use crate::spec::PlanError;

/// The boundary envelope is part of the wire format too.
#[test]
fn plan_result_envelope_serializes_as_ok_or_err() {
    let ok = PlanResult::Ok {
        ok: plan(wall(900.0, 230.0)).unwrap(),
    };
    let ok_json = serde_json::to_value(&ok).unwrap();
    assert!(ok_json.get("ok").is_some_and(|p| p.get("steps").is_some()));

    let err = PlanResult::Err {
        err: PlanError::MalformedSpec {
            message: "bad".into(),
        },
    };
    assert_eq!(
        serde_json::to_value(&err).unwrap(),
        serde_json::json!({ "err": { "kind": "MalformedSpec", "message": "bad" } })
    );
}

/// Guards the wire format: the UI and any future consumer parse this JSON.
/// On an intentional format change, run `UPDATE_FIXTURES=1 cargo test`
/// and review the fixture diff before committing.
#[test]
fn small_plan_serializes_to_the_committed_fixture() {
    let spec = with_opening(wall(900.0, 230.0), door(340.0, 220.0, 170.0));
    let plan = plan(spec).unwrap();
    let actual = serde_json::to_value(&plan).unwrap();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/tests/fixtures/small_plan.json"
    );
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        let pretty = serde_json::to_string_pretty(&actual).unwrap();
        std::fs::write(path, pretty + "\n").unwrap();
    }
    let expected: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(actual, expected);
}
