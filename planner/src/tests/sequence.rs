use super::*;
use crate::layout::{BrickKind, Placement, layout_wall};
use crate::plan::plan;
use crate::sequence::{Action, sequence, validate_support};
use crate::spec::PlanError;

#[test]
fn steps_alternate_pick_and_place() {
    let plan = plan(wall(2500.0, 600.0)).unwrap();
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
        wall(3000.0, 1200.0),
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
fn placements_are_ordered_bottom_up_then_left_to_right() {
    let plan = plan(with_opening(
        wall(3000.0, 2400.0),
        window(1000.0, 800.0, 600.0, 600.0),
    ))
    .unwrap();
    let order: Vec<u32> = plan
        .steps
        .iter()
        .filter_map(|s| match s.action {
            Action::PlaceBrick { placement_id } => Some(placement_id),
            _ => None,
        })
        .collect();
    let placed: Vec<&Placement> = order
        .iter()
        .map(|id| plan.placements.iter().find(|p| p.id == *id).unwrap())
        .collect();
    for pair in placed.windows(2) {
        let (a, b) = (pair[0], pair[1]);
        assert!(
            a.course < b.course || (a.course == b.course && a.x < b.x),
            "placement {} must come before {}",
            a.id,
            b.id
        );
    }
}

#[test]
fn support_check_accepts_generated_layouts() {
    let specs = [
        wall(2500.0, 2000.0),
        with_opening(wall(3000.0, 2400.0), door(900.0, 800.0, 2000.0)),
        // Opening wide enough that some lintel bricks sit entirely over it.
        with_opening(wall(3000.0, 2400.0), window(1000.0, 800.0, 600.0, 600.0)),
    ];
    for spec in &specs {
        let placements = layout_wall(spec);
        assert_eq!(validate_support(spec, &placements), Ok(()));
    }
}

#[test]
fn support_check_rejects_a_floating_brick() {
    let spec = wall(2500.0, 2000.0);
    let placements = vec![
        Placement {
            id: 0,
            course: 0,
            kind: BrickKind::Full,
            x: 0.0,
            y: 0.0,
        },
        // Course 2 with nothing in course 1 underneath it.
        Placement {
            id: 1,
            course: 2,
            kind: BrickKind::Full,
            x: 0.0,
            y: 120.0,
        },
    ];
    assert_eq!(
        validate_support(&spec, &placements),
        Err(PlanError::UnsupportedPlacement { placement_id: 1 })
    );
}

#[test]
fn support_check_rejects_a_brick_with_no_footprint_overlap() {
    let spec = wall(2500.0, 2000.0);
    let placements = vec![
        Placement {
            id: 0,
            course: 0,
            kind: BrickKind::Full,
            x: 0.0,
            y: 0.0,
        },
        Placement {
            id: 1,
            course: 1,
            kind: BrickKind::Full,
            x: 1000.0,
            y: 60.0,
        },
    ];
    assert_eq!(
        validate_support(&spec, &placements),
        Err(PlanError::UnsupportedPlacement { placement_id: 1 })
    );
}

#[test]
fn sequence_propagates_support_failures() {
    let spec = wall(2500.0, 2000.0);
    let floating = vec![Placement {
        id: 7,
        course: 3,
        kind: BrickKind::Full,
        x: 0.0,
        y: 180.0,
    }];
    assert_eq!(
        sequence(&spec, &floating),
        Err(PlanError::UnsupportedPlacement { placement_id: 7 })
    );
}
