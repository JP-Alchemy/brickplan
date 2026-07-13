use super::*;
use crate::layout::{BrickKind, MIN_CUT_LENGTH, course_count, half_brick_length, layout_wall};
use crate::plan::plan;

#[test]
fn course_count_is_derived_from_brick_and_joint_height() {
    // Course module is 50 + 10 = 60mm; a course only counts if the brick
    // itself (top course needs no joint above) fits inside the wall height.
    assert_eq!(course_count(&wall(1000.0, 2500.0)), 41); // 40 * 60 + 50 = 2450
    assert_eq!(course_count(&wall(1000.0, 50.0)), 1);
    assert_eq!(course_count(&wall(1000.0, 109.0)), 1); // second course would top out at 110
    assert_eq!(course_count(&wall(1000.0, 110.0)), 2);
}

#[test]
fn half_brick_plus_joint_is_half_the_bond_module() {
    let spec = wall(1000.0, 1000.0);
    assert!((half_brick_length(&spec) - 100.0).abs() < EPS);
}

#[test]
fn even_courses_start_with_a_full_brick() {
    let placements = layout_wall(&wall(2200.0, 300.0));
    let first = placements.iter().find(|p| p.course == 0).unwrap();
    assert_eq!(first.kind, BrickKind::Full);
    assert!(first.x.abs() < EPS);
}

#[test]
fn odd_courses_start_with_a_half_brick() {
    let placements = layout_wall(&wall(2200.0, 300.0));
    let first = placements.iter().find(|p| p.course == 1).unwrap();
    assert_eq!(first.kind, BrickKind::Half);
    assert!(first.x.abs() < EPS);
}

#[test]
fn no_two_placements_overlap() {
    let specs = [
        wall(2500.0, 2000.0),
        with_opening(wall(3000.0, 2400.0), door(900.0, 800.0, 2000.0)),
        with_opening(wall(3000.0, 2400.0), window(1000.0, 800.0, 600.0, 600.0)),
    ];
    for spec in &specs {
        let placements = layout_wall(spec);
        for (i, a) in placements.iter().enumerate() {
            for b in &placements[i + 1..] {
                assert!(
                    !rects_overlap(rect(a, spec), rect(b, spec)),
                    "{a:?} overlaps {b:?}"
                );
            }
        }
    }
}

#[test]
fn plain_courses_cover_the_wall_width() {
    let spec = wall(2530.0, 1200.0);
    let placements = layout_wall(&spec);
    for course in 0..course_count(&spec) {
        let mut bricks: Vec<_> = placements.iter().filter(|p| p.course == course).collect();
        bricks.sort_by(|a, b| a.x.total_cmp(&b.x));
        assert!(bricks[0].x.abs() < EPS, "course {course} starts at x=0");
        for pair in bricks.windows(2) {
            let gap = pair[1].x - (pair[0].x + pair[0].kind.length(&spec));
            assert!(
                (gap - spec.joint).abs() < EPS,
                "course {course}: gap between bricks is {gap}, expected joint"
            );
        }
        let last = bricks.last().unwrap();
        let right = last.x + last.kind.length(&spec);
        // The course either fills the width exactly or absorbs a sliver
        // shorter than MIN_CUT_LENGTH into joint tolerance.
        assert!(right <= spec.width + EPS);
        assert!(right > spec.width - spec.joint - MIN_CUT_LENGTH);
    }
}

#[test]
fn no_placement_intersects_the_opening() {
    let specs = [
        with_opening(wall(3000.0, 2400.0), door(900.0, 800.0, 2000.0)),
        with_opening(wall(3000.0, 2400.0), window(1000.0, 800.0, 600.0, 600.0)),
    ];
    for spec in &specs {
        let op = spec.opening.as_ref().unwrap();
        let op_rect = (op.x, op.sill_height, op.right(), op.top());
        for p in layout_wall(spec) {
            assert!(
                !rects_overlap(rect(&p, spec), op_rect),
                "{p:?} intersects the opening"
            );
        }
    }
}

#[test]
fn cut_bricks_abut_the_opening_edges() {
    let spec = with_opening(wall(3000.0, 2400.0), window(1000.0, 800.0, 600.0, 600.0));
    let op = spec.opening.as_ref().unwrap();
    let placements = layout_wall(&spec);
    // Across the courses that intersect the opening, cut bricks must land
    // flush against both opening edges. (Not every course has both: a
    // course whose cut piece would be a sliver absorbs it instead.)
    let in_opening: Vec<_> = placements
        .iter()
        .filter(|p| p.y < op.top() - EPS && p.y + spec.brick.height > op.sill_height + EPS)
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
    let spec = with_opening(wall(3000.0, 2400.0), window(1000.0, 800.0, 600.0, 600.0));
    let op = spec.opening.as_ref().unwrap();
    let lintel: Vec<_> = layout_wall(&spec)
        .into_iter()
        .filter(|p| p.course == 20)
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
        wall(2500.0, 2000.0),
        wall(680.0, 500.0), // remainder of 20mm per even course must be absorbed
        with_opening(wall(3000.0, 2400.0), door(215.0, 800.0, 2000.0)),
    ];
    for spec in &specs {
        for p in layout_wall(spec) {
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
fn all_placements_stay_inside_the_wall() {
    let specs = [
        wall(2500.0, 2000.0),
        with_opening(wall(3000.0, 2400.0), window(1000.0, 800.0, 600.0, 600.0)),
    ];
    for spec in &specs {
        for p in layout_wall(spec) {
            let (x0, y0, x1, y1) = rect(&p, spec);
            assert!(x0 >= -EPS && y0 >= -EPS);
            assert!(x1 <= spec.width + EPS && y1 <= spec.height + EPS);
        }
    }
}

#[test]
fn stats_match_the_placements() {
    let spec = with_opening(wall(3000.0, 2400.0), window(1000.0, 800.0, 600.0, 600.0));
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
