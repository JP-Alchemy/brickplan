use super::*;
use crate::plan::plan;
use crate::spec::PlanError;

#[test]
fn accepts_plain_wall() {
    assert_eq!(wall(2500.0, 2000.0).validate(), Ok(()));
}

#[test]
fn rejects_zero_wall_width() {
    assert_eq!(
        wall(0.0, 2000.0).validate(),
        Err(PlanError::InvalidDimension {
            field: "wall width".into()
        })
    );
}

#[test]
fn rejects_negative_wall_height() {
    assert_eq!(
        wall(2500.0, -1.0).validate(),
        Err(PlanError::InvalidDimension {
            field: "wall height".into()
        })
    );
}

#[test]
fn rejects_non_finite_dimension() {
    assert_eq!(
        wall(f64::NAN, 2000.0).validate(),
        Err(PlanError::InvalidDimension {
            field: "wall width".into()
        })
    );
}

#[test]
fn rejects_zero_joint() {
    let mut spec = wall(2500.0, 2000.0);
    spec.joint = 0.0;
    assert_eq!(
        spec.validate(),
        Err(PlanError::InvalidDimension {
            field: "joint".into()
        })
    );
}

#[test]
fn rejects_non_positive_brick_dims() {
    let mut spec = wall(2500.0, 2000.0);
    spec.brick.height = 0.0;
    assert_eq!(
        spec.validate(),
        Err(PlanError::InvalidDimension {
            field: "brick height".into()
        })
    );
}

#[test]
fn rejects_wall_narrower_than_one_brick() {
    assert_eq!(
        wall(150.0, 2000.0).validate(),
        Err(PlanError::WallSmallerThanBrick)
    );
}

#[test]
fn rejects_wall_shorter_than_one_brick() {
    assert_eq!(
        wall(2500.0, 30.0).validate(),
        Err(PlanError::WallSmallerThanBrick)
    );
}

#[test]
fn rejects_opening_past_right_edge() {
    let spec = with_opening(wall(2500.0, 2000.0), door(2000.0, 800.0, 1500.0));
    assert_eq!(spec.validate(), Err(PlanError::OpeningOutOfBounds));
}

#[test]
fn rejects_opening_above_wall_top() {
    let spec = with_opening(wall(2500.0, 2000.0), window(500.0, 800.0, 1500.0, 600.0));
    assert_eq!(spec.validate(), Err(PlanError::OpeningOutOfBounds));
}

#[test]
fn rejects_opening_with_negative_x() {
    let spec = with_opening(wall(2500.0, 2000.0), door(-10.0, 800.0, 1500.0));
    assert_eq!(spec.validate(), Err(PlanError::OpeningOutOfBounds));
}

#[test]
fn rejects_zero_width_opening() {
    let spec = with_opening(wall(2500.0, 2000.0), door(500.0, 0.0, 1500.0));
    assert_eq!(
        spec.validate(),
        Err(PlanError::InvalidDimension {
            field: "opening width".into()
        })
    );
}

#[test]
fn accepts_opening_flush_with_left_edge() {
    let spec = with_opening(wall(2500.0, 2000.0), door(0.0, 800.0, 1500.0));
    assert_eq!(spec.validate(), Ok(()));
}

#[test]
fn plan_surfaces_validation_errors() {
    assert_eq!(
        plan(wall(100.0, 100.0)),
        Err(PlanError::WallSmallerThanBrick)
    );
}
