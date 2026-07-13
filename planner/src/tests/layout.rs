use super::*;
use crate::layout::{
    BrickKind, MIN_CUT_LENGTH, WallSide, course_count, half_brick_length, layout_walls,
    plan_rects_overlap,
};
use crate::plan::plan;

const SIDES: [WallSide; 4] = [
    WallSide::South,
    WallSide::East,
    WallSide::North,
    WallSide::West,
];

#[test]
fn course_count_is_derived_from_brick_and_joint_height() {
    // Course module is 50 + 10 = 60mm; a course only counts if the brick
    // itself (top course needs no joint above) fits inside the wall height.
    assert_eq!(course_count(&room(1000.0, 1000.0, 2500.0)), 41); // 40 * 60 + 50 = 2450
    assert_eq!(course_count(&room(1000.0, 1000.0, 50.0)), 1);
    assert_eq!(course_count(&room(1000.0, 1000.0, 109.0)), 1);
    assert_eq!(course_count(&room(1000.0, 1000.0, 110.0)), 2);
}

#[test]
fn half_brick_plus_joint_is_half_the_bond_module() {
    let spec = room(1000.0, 1000.0, 1000.0);
    assert!((half_brick_length(&spec) - 100.0).abs() < EPS);
}

#[test]
fn every_course_has_bricks_on_all_four_walls() {
    let spec = room(2200.0, 1500.0, 300.0);
    let placements = layout_walls(&spec);
    for course in 0..course_count(&spec) {
        for side in SIDES {
            assert!(
                placements
                    .iter()
                    .any(|p| p.course == course && p.wall == side),
                "course {course} is missing bricks on {side:?}"
            );
        }
    }
}

#[test]
fn corners_alternate_which_wall_runs_through() {
    let spec = room(2200.0, 1500.0, 300.0);
    let placements = layout_walls(&spec);
    let m = spec.corner_return(); // 110mm
    let min_along = |course, side| {
        placements
            .iter()
            .filter(|p| p.course == course && p.wall == side)
            .map(|p| p.along())
            .fold(f64::INFINITY, f64::min)
    };
    // Even course: front wall reaches the outer corner, side walls recede.
    assert!(min_along(0, WallSide::South).abs() < EPS);
    assert!((min_along(0, WallSide::West) - m).abs() < EPS);
    // Odd course: side walls reach the corner, front wall recedes by
    // exactly half a module — the corner IS the stretcher stagger.
    assert!(min_along(1, WallSide::West).abs() < EPS);
    assert!((min_along(1, WallSide::South) - m).abs() < EPS);
}

#[test]
fn through_walls_reach_both_outer_corners() {
    let spec = room(2200.0, 1500.0, 300.0);
    let placements = layout_walls(&spec);
    let max_edge = |course, side: WallSide| {
        placements
            .iter()
            .filter(|p| p.course == course && p.wall == side)
            .map(|p| p.along() + p.kind.length(&spec))
            .fold(f64::NEG_INFINITY, f64::max)
    };
    // Even course: south spans the full width (within joint tolerance).
    assert!(max_edge(0, WallSide::South) > spec.width - spec.joint - MIN_CUT_LENGTH);
    // Odd course: west spans the full length.
    assert!(max_edge(1, WallSide::West) > spec.length - spec.joint - MIN_CUT_LENGTH);
}

#[test]
fn no_two_placements_overlap_in_plan_within_a_course() {
    let specs = [
        room(2500.0, 2000.0, 400.0),
        with_opening(room(3000.0, 2400.0, 2400.0), door(900.0, 800.0, 2000.0)),
        with_opening(
            room(3000.0, 2400.0, 2400.0),
            window(1000.0, 800.0, 600.0, 600.0),
        ),
    ];
    for spec in &specs {
        let placements = layout_walls(spec);
        for (i, a) in placements.iter().enumerate() {
            for b in &placements[i + 1..] {
                if a.course != b.course {
                    continue;
                }
                assert!(
                    !plan_rects_overlap(a.plan_rect(spec), b.plan_rect(spec)),
                    "{a:?} overlaps {b:?}"
                );
            }
        }
    }
}

#[test]
fn courses_cover_each_wall_span() {
    let spec = room(2530.0, 1870.0, 1200.0);
    let placements = layout_walls(&spec);
    let m = spec.corner_return();
    for course in 0..course_count(&spec) {
        for side in SIDES {
            let through = (course % 2 == 0) == side.along_x();
            let extent = if side.along_x() {
                spec.width
            } else {
                spec.length
            };
            let (start, end) = if through {
                (0.0, extent)
            } else {
                (m, extent - m)
            };
            let mut bricks: Vec<_> = placements
                .iter()
                .filter(|p| p.course == course && p.wall == side)
                .collect();
            bricks.sort_by(|a, b| a.along().total_cmp(&b.along()));
            assert!(
                (bricks[0].along() - start).abs() < EPS,
                "course {course} {side:?} starts at {}",
                bricks[0].along()
            );
            for pair in bricks.windows(2) {
                let gap = pair[1].along() - (pair[0].along() + pair[0].kind.length(&spec));
                assert!(
                    (gap - spec.joint).abs() < EPS,
                    "course {course} {side:?}: gap {gap} != joint"
                );
            }
            let last = bricks.last().unwrap();
            let right = last.along() + last.kind.length(&spec);
            assert!(right <= end + EPS);
            assert!(right > end - spec.joint - MIN_CUT_LENGTH);
        }
    }
}

#[test]
fn no_placement_intersects_the_opening() {
    let specs = [
        with_opening(room(3000.0, 2400.0, 2400.0), door(900.0, 800.0, 2000.0)),
        with_opening(
            room(3000.0, 2400.0, 2400.0),
            window(1000.0, 800.0, 600.0, 600.0),
        ),
    ];
    for spec in &specs {
        let op = spec.opening.as_ref().unwrap();
        for p in layout_walls(spec) {
            if p.wall != WallSide::South {
                continue;
            }
            // Front-wall elevation: x along the wall, z up.
            let intersects = p.x < op.right() - EPS
                && p.x + p.kind.length(spec) > op.x + EPS
                && p.z < op.top() - EPS
                && p.z + spec.brick.height > op.sill_height + EPS;
            assert!(!intersects, "{p:?} intersects the opening");
        }
    }
}

#[test]
fn cut_bricks_abut_the_opening_edges() {
    let spec = with_opening(
        room(3000.0, 2400.0, 2400.0),
        window(1000.0, 800.0, 600.0, 600.0),
    );
    let op = spec.opening.as_ref().unwrap();
    let placements = layout_walls(&spec);
    let in_opening: Vec<_> = placements
        .iter()
        .filter(|p| {
            p.wall == WallSide::South
                && p.z < op.top() - EPS
                && p.z + spec.brick.height > op.sill_height + EPS
        })
        .collect();
    assert!(
        in_opening
            .iter()
            .any(|p| (p.x + p.kind.length(&spec) - op.x).abs() < EPS),
        "a brick should end flush against the opening's left edge"
    );
    assert!(
        in_opening.iter().any(|p| (p.x - op.right()).abs() < EPS),
        "a brick should start flush against the opening's right edge"
    );
}

#[test]
fn course_above_a_window_spans_the_opening() {
    // Opening top is 600 + 600 = 1200, exactly the bottom of course 20.
    let spec = with_opening(
        room(3000.0, 2400.0, 2400.0),
        window(1000.0, 800.0, 600.0, 600.0),
    );
    let op = spec.opening.as_ref().unwrap();
    let lintel: Vec<_> = layout_walls(&spec)
        .into_iter()
        .filter(|p| p.wall == WallSide::South && p.course == 20)
        .collect();
    let spans_opening = lintel
        .iter()
        .any(|p| p.x < op.right() - EPS && p.x + p.kind.length(&spec) > op.x + EPS);
    assert!(
        spans_opening,
        "lintel course should run uninterrupted over the window"
    );
}

#[test]
fn cut_bricks_are_never_shorter_than_the_minimum() {
    let specs = [
        room(2500.0, 2000.0, 2000.0),
        room(680.0, 460.0, 500.0),
        with_opening(room(3000.0, 2400.0, 2400.0), door(215.0, 800.0, 2000.0)),
    ];
    for spec in &specs {
        for p in layout_walls(spec) {
            if let BrickKind::Cut { length } = p.kind {
                assert!(
                    length >= MIN_CUT_LENGTH - EPS,
                    "cut of {length}mm is below the minimum"
                );
            }
        }
    }
}

#[test]
fn all_placements_stay_inside_the_footprint() {
    let specs = [
        room(2500.0, 2000.0, 2000.0),
        with_opening(
            room(3000.0, 2400.0, 2400.0),
            window(1000.0, 800.0, 600.0, 600.0),
        ),
    ];
    for spec in &specs {
        for p in layout_walls(spec) {
            let (x0, y0, x1, y1) = p.plan_rect(spec);
            assert!(x0 >= -EPS && y0 >= -EPS);
            assert!(x1 <= spec.width + EPS && y1 <= spec.length + EPS);
            assert!(p.z >= -EPS && p.z + spec.brick.height <= spec.height + EPS);
        }
    }
}

#[test]
fn stats_match_the_placements() {
    let spec = with_opening(
        room(3000.0, 2400.0, 2400.0),
        window(1000.0, 800.0, 600.0, 600.0),
    );
    let plan = plan(spec.clone()).unwrap();
    let count = |pred: fn(&BrickKind) -> bool| {
        plan.placements.iter().filter(|p| pred(&p.kind)).count() as u32
    };
    assert_eq!(plan.stats.courses, course_count(&spec));
    assert_eq!(
        plan.stats.full_bricks,
        count(|k| matches!(k, BrickKind::Full))
    );
    assert_eq!(
        plan.stats.half_bricks,
        count(|k| matches!(k, BrickKind::Half))
    );
    assert_eq!(
        plan.stats.cut_bricks,
        count(|k| matches!(k, BrickKind::Cut { .. }))
    );
}
