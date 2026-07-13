use super::*;
use crate::plan::plan;
use crate::spec::{BrickDims, PlanError, WallSide};

#[test]
fn accepts_plain_room() {
    assert_eq!(room(2500.0, 2000.0, 2000.0).validate(), Ok(()));
}

#[test]
fn rejects_zero_wall_width() {
    assert_eq!(
        room(0.0, 2000.0, 2000.0).validate(),
        Err(PlanError::InvalidDimension {
            field: "wall width".into()
        })
    );
}

#[test]
fn rejects_negative_wall_length() {
    assert_eq!(
        room(2500.0, -1.0, 2000.0).validate(),
        Err(PlanError::InvalidDimension {
            field: "wall length".into()
        })
    );
}

#[test]
fn rejects_non_finite_dimension() {
    assert_eq!(
        room(f64::NAN, 2000.0, 2000.0).validate(),
        Err(PlanError::InvalidDimension {
            field: "wall width".into()
        })
    );
}

#[test]
fn rejects_zero_joint() {
    let mut spec = room(2500.0, 2000.0, 2000.0);
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
    let mut spec = room(2500.0, 2000.0, 2000.0);
    spec.brick.height = 0.0;
    assert_eq!(
        spec.validate(),
        Err(PlanError::InvalidDimension {
            field: "brick height".into()
        })
    );
}

#[test]
fn rejects_bricks_that_cannot_bond_at_corners() {
    // Corner returns need length = 2 x width + joint; 200 breaks that.
    let mut spec = room(2500.0, 2000.0, 2000.0);
    spec.brick = BrickDims {
        length: 200.0,
        height: 50.0,
        width: 100.0,
    };
    assert!(matches!(
        spec.validate(),
        Err(PlanError::InvalidDimension { field }) if field.starts_with("brick proportions")
    ));
}

#[test]
fn rejects_footprint_too_small_for_corner_returns() {
    // Minimum extent is two corner returns (110 each) plus one minimum cut.
    assert_eq!(
        room(250.0, 2000.0, 2000.0).validate(),
        Err(PlanError::WallSmallerThanBrick)
    );
    assert_eq!(
        room(2000.0, 250.0, 2000.0).validate(),
        Err(PlanError::WallSmallerThanBrick)
    );
    assert_eq!(room(260.0, 260.0, 2000.0).validate(), Ok(()));
}

#[test]
fn rejects_wall_shorter_than_one_brick() {
    assert_eq!(
        room(2500.0, 2000.0, 30.0).validate(),
        Err(PlanError::WallSmallerThanBrick)
    );
}

#[test]
fn rejects_opening_past_the_corner_zone() {
    // Front wall is 2500 wide; the last 110mm belong to the corner return.
    let spec = with_opening(
        room(2500.0, 2000.0, 2000.0),
        door(WallSide::South, 1700.0, 800.0, 1500.0),
    );
    assert_eq!(spec.validate(), Err(PlanError::OpeningOutOfBounds));
}

#[test]
fn rejects_opening_in_the_left_corner_zone() {
    let spec = with_opening(
        room(2500.0, 2000.0, 2000.0),
        door(WallSide::South, 50.0, 800.0, 1500.0),
    );
    assert_eq!(spec.validate(), Err(PlanError::OpeningOutOfBounds));
}

#[test]
fn side_wall_openings_use_that_walls_extent() {
    // Length is 2000, so x = 1200, width 800 runs into the corner return
    // on a side wall — but would be fine on the 2500-wide front wall.
    let spec = with_opening(
        room(2500.0, 2000.0, 2000.0),
        door(WallSide::East, 1200.0, 800.0, 1500.0),
    );
    assert_eq!(spec.validate(), Err(PlanError::OpeningOutOfBounds));
    let spec = with_opening(
        room(2500.0, 2000.0, 2000.0),
        door(WallSide::East, 1000.0, 800.0, 1500.0),
    );
    assert_eq!(spec.validate(), Ok(()));
}

#[test]
fn accepts_opening_flush_with_the_corner_return() {
    let spec = with_opening(
        room(2500.0, 2000.0, 2000.0),
        door(WallSide::South, 110.0, 800.0, 1500.0),
    );
    assert_eq!(spec.validate(), Ok(()));
}

#[test]
fn rejects_opening_above_wall_top() {
    let spec = with_opening(
        room(2500.0, 2000.0, 2000.0),
        window(WallSide::South, 500.0, 800.0, 1500.0, 600.0),
    );
    assert_eq!(spec.validate(), Err(PlanError::OpeningOutOfBounds));
}

#[test]
fn rejects_zero_width_opening() {
    let spec = with_opening(
        room(2500.0, 2000.0, 2000.0),
        door(WallSide::South, 500.0, 0.0, 1500.0),
    );
    assert_eq!(
        spec.validate(),
        Err(PlanError::InvalidDimension {
            field: "opening width".into()
        })
    );
}

#[test]
fn rejects_overlapping_openings_on_the_same_wall() {
    let spec = with_openings(
        room(2500.0, 2000.0, 2000.0),
        vec![
            door(WallSide::South, 400.0, 800.0, 1500.0),
            window(WallSide::South, 1000.0, 600.0, 900.0, 600.0),
        ],
    );
    assert_eq!(spec.validate(), Err(PlanError::OpeningsOverlap));
}

#[test]
fn accepts_stacked_openings_that_do_not_overlap() {
    // Same x range, but the window sits fully above the door's top.
    let spec = with_openings(
        room(2500.0, 2000.0, 2000.0),
        vec![
            door(WallSide::South, 400.0, 800.0, 1200.0),
            window(WallSide::South, 400.0, 800.0, 1300.0, 500.0),
        ],
    );
    assert_eq!(spec.validate(), Ok(()));
}

#[test]
fn accepts_identical_openings_on_different_walls() {
    let spec = with_openings(
        room(2500.0, 2000.0, 2000.0),
        vec![
            door(WallSide::South, 400.0, 800.0, 1500.0),
            door(WallSide::North, 400.0, 800.0, 1500.0),
        ],
    );
    assert_eq!(spec.validate(), Ok(()));
}

#[test]
fn plan_surfaces_validation_errors() {
    assert_eq!(
        plan(room(100.0, 100.0, 100.0)),
        Err(PlanError::WallSmallerThanBrick)
    );
}
