//! Layout engine: turns a validated `WallSpec` into brick placements.
//!
//! Pure geometry, no I/O, no state. Everything here also runs natively so
//! the test suite needs no WASM tooling.

use serde::{Deserialize, Serialize};

use crate::spec::WallSpec;

/// Cut-brick pieces shorter than this are absorbed into joint tolerance
/// instead of being placed. Cutting and handling slivers under ~40mm is
/// not worth it on a real wall either.
pub const MIN_CUT_LENGTH: f64 = 40.0;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum BrickKind {
    Full,
    Half,
    Cut { length: f64 },
}

impl BrickKind {
    /// The as-laid length of this brick along the x axis.
    pub fn length(&self, spec: &WallSpec) -> f64 {
        match self {
            BrickKind::Full => spec.brick.length,
            BrickKind::Half => half_brick_length(spec),
            BrickKind::Cut { length } => *length,
        }
    }
}

/// A half brick plus half a joint equals half the bond module, so odd
/// courses shift by exactly (length + joint) / 2.
pub fn half_brick_length(spec: &WallSpec) -> f64 {
    (spec.brick.length - spec.joint) / 2.0
}

/// One brick, placed. `x`/`y` is the bottom-left corner.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Placement {
    pub id: u32,
    pub course: u32,
    pub kind: BrickKind,
    pub x: f64,
    pub y: f64,
}

const EPS: f64 = 1e-6;

/// Vertical distance from one course's bottom to the next one's.
pub fn course_height(spec: &WallSpec) -> f64 {
    spec.brick.height + spec.joint
}

/// Number of courses that fit in the wall height. The top course needs no
/// joint above it, so only the brick itself has to fit.
pub fn course_count(spec: &WallSpec) -> u32 {
    ((spec.height - spec.brick.height + EPS) / course_height(spec)) as u32 + 1
}

/// Lay out the whole wall in stretcher bond, cutting around the opening.
/// Placements are emitted bottom-up, left-to-right.
pub fn layout_wall(spec: &WallSpec) -> Vec<Placement> {
    let mut placements = Vec::new();
    let mut id = 0;
    for course in 0..course_count(spec) {
        let y = f64::from(course) * course_height(spec);
        for (x, length) in course_segments(spec, course) {
            for (px, plen) in cut_around_opening(spec, x, length, y) {
                placements.push(Placement {
                    id,
                    course,
                    kind: classify(spec, plen),
                    x: px,
                    y,
                });
                id += 1;
            }
        }
    }
    placements
}

/// The bond pattern for one course, ignoring the opening: a list of
/// (x, length) segments. Odd courses start with a half brick so vertical
/// joints shift by half a module.
fn course_segments(spec: &WallSpec, course: u32) -> Vec<(f64, f64)> {
    let mut segments = Vec::new();
    let mut cursor = 0.0;
    if course % 2 == 1 {
        let half = half_brick_length(spec);
        segments.push((cursor, half));
        cursor += half + spec.joint;
    }
    loop {
        let remaining = spec.width - cursor;
        if remaining + EPS >= spec.brick.length {
            segments.push((cursor, spec.brick.length));
            cursor += spec.brick.length + spec.joint;
        } else {
            // End-of-course remainder becomes a cut brick, unless it is a
            // sliver below MIN_CUT_LENGTH — then we absorb it into joint
            // tolerance rather than asking anyone to cut and lay it.
            if remaining + EPS >= MIN_CUT_LENGTH {
                segments.push((cursor, remaining));
            }
            break;
        }
    }
    segments
}

/// Cut one bond segment around the opening. Returns the pieces to keep:
/// the whole segment if it misses the opening, otherwise the parts left
/// and right of it (cut flush against the opening edges), dropping any
/// piece below MIN_CUT_LENGTH.
fn cut_around_opening(spec: &WallSpec, x: f64, length: f64, y: f64) -> Vec<(f64, f64)> {
    let Some(op) = &spec.opening else {
        return vec![(x, length)];
    };
    let intersects_vertically = y < op.top() - EPS && y + spec.brick.height > op.sill_height + EPS;
    let intersects_horizontally = x < op.right() - EPS && x + length > op.x + EPS;
    if !intersects_vertically || !intersects_horizontally {
        return vec![(x, length)];
    }
    let mut pieces = Vec::new();
    let left = op.x - x;
    if left + EPS >= MIN_CUT_LENGTH {
        pieces.push((x, left));
    }
    let right = (x + length) - op.right();
    if right + EPS >= MIN_CUT_LENGTH {
        pieces.push((op.right(), right));
    }
    pieces
}

fn classify(spec: &WallSpec, length: f64) -> BrickKind {
    if (length - spec.brick.length).abs() < EPS {
        BrickKind::Full
    } else if (length - half_brick_length(spec)).abs() < EPS {
        BrickKind::Half
    } else {
        BrickKind::Cut { length }
    }
}
