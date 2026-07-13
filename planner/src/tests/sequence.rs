use super::*;
use crate::layout::{BrickKind, Placement, WallSide, layout_walls};
use crate::plan::plan;
use crate::sequence::{Action, sequence, validate_support};
use crate::spec::PlanError;

fn brick(id: u32, course: u32, wall: WallSide, x: f64, y: f64) -> Placement {
    Placement {
        id,
        course,
        wall,
        kind: BrickKind::Full,
        x,
        y,
        z: f64::from(course) * 60.0,
    }
}

#[test]
fn steps_alternate_pick_and_place() {
    let plan = plan(room(2500.0, 2000.0, 600.0)).unwrap();
    assert_eq!(plan.steps.len(), plan.placements.len() * 2);
    for (i, step) in plan.steps.iter().enumerate() {
        assert_eq!(step.seq as usize, i);
        match &step.action {
            Action::PickBrick { .. } => assert_eq!(i % 2, 0),
            Action::PlaceBrick { .. } => assert_eq!(i % 2, 1),
        }
    }
}

#[test]
fn each_pick_matches_the_following_place() {
    let plan = plan(with_opening(
        room(3000.0, 2000.0, 1200.0),
        door(900.0, 800.0, 1000.0),
    ))
    .unwrap();
    for pair in plan.steps.chunks(2) {
        let (Action::PickBrick { kind }, Action::PlaceBrick { placement_id }) =
            (&pair[0].action, &pair[1].action)
        else {
            panic!("steps must alternate pick/place");
        };
        let placement = plan
            .placements
            .iter()
            .find(|p| p.id == *placement_id)
            .expect("place step references a real placement");
        assert_eq!(&placement.kind, kind);
    }
}

#[test]
fn placements_are_ordered_bottom_up_then_by_wall_then_along_it() {
    let plan = plan(with_opening(
        room(3000.0, 2400.0, 2400.0),
        window(1000.0, 800.0, 600.0, 600.0),
    ))
    .unwrap();
    let placed: Vec<&Placement> = plan
        .steps
        .iter()
        .filter_map(|s| match s.action {
            Action::PlaceBrick { placement_id } => {
                plan.placements.iter().find(|p| p.id == placement_id)
            }
            _ => None,
        })
        .collect();
    for pair in placed.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        let key = |p: &Placement| (p.course, p.wall.order());
        assert!(
            key(a) < key(b) || (key(a) == key(b) && a.along() < b.along()),
            "placement {} must come before {}",
            a.id,
            b.id
        );
    }
}

#[test]
fn support_check_accepts_generated_layouts() {
    let specs = [
        room(2500.0, 2000.0, 2000.0),
        with_opening(room(3000.0, 2400.0, 2400.0), door(900.0, 800.0, 2000.0)),
        // Opening wide enough that some lintel bricks sit entirely over it.
        with_opening(
            room(3000.0, 2400.0, 2400.0),
            window(1000.0, 800.0, 600.0, 600.0),
        ),
    ];
    for spec in &specs {
        let placements = layout_walls(spec);
        assert_eq!(validate_support(spec, &placements), Ok(()));
    }
}

#[test]
fn a_corner_brick_is_supported_by_the_perpendicular_wall() {
    let spec = room(2500.0, 2000.0, 2000.0);
    // Course 0: a south through-brick at the SW corner. Course 1: a west
    // through-brick whose only support is that south brick's footprint.
    let placements = vec![
        brick(0, 0, WallSide::South, 0.0, 0.0),
        brick(1, 1, WallSide::West, 0.0, 0.0),
    ];
    assert_eq!(validate_support(&spec, &placements), Ok(()));
}

#[test]
fn support_check_rejects_a_floating_brick() {
    let spec = room(2500.0, 2000.0, 2000.0);
    let placements = vec![
        brick(0, 0, WallSide::South, 0.0, 0.0),
        // Course 2 with nothing in course 1 underneath it.
        brick(1, 2, WallSide::South, 0.0, 0.0),
    ];
    assert_eq!(
        validate_support(&spec, &placements),
        Err(PlanError::UnsupportedPlacement { placement_id: 1 })
    );
}

#[test]
fn support_check_rejects_a_brick_with_no_footprint_overlap() {
    let spec = room(2500.0, 2000.0, 2000.0);
    let placements = vec![
        brick(0, 0, WallSide::South, 0.0, 0.0),
        brick(1, 1, WallSide::South, 1000.0, 0.0),
    ];
    assert_eq!(
        validate_support(&spec, &placements),
        Err(PlanError::UnsupportedPlacement { placement_id: 1 })
    );
}

#[test]
fn sequence_propagates_support_failures() {
    let spec = room(2500.0, 2000.0, 2000.0);
    let floating = vec![brick(7, 3, WallSide::South, 0.0, 0.0)];
    assert_eq!(
        sequence(&spec, &floating),
        Err(PlanError::UnsupportedPlacement { placement_id: 7 })
    );
}
