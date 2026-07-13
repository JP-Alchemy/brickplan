//! Layout engine: turns a validated `WallSpec` into brick placements on
//! all four walls of the footprint.
//!
//! Pure geometry, no I/O, no state. Everything here also runs natively so
//! the test suite needs no WASM tooling.
//!
//! Corners: on even courses the front and back walls run through to the
//! outer corners and the side walls butt against them; on odd courses the
//! side walls run through. Because a brick's width plus a joint is exactly
//! half the bond module, this alternation staggers every vertical joint —
//! including at the corners — with no corner cuts at all.

use serde::{Deserialize, Serialize};

pub use crate::spec::MIN_CUT_LENGTH;
use crate::spec::WallSpec;

const EPS: f64 = 1e-6;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum BrickKind {
    Full,
    Half,
    Cut { length: f64 },
}

impl BrickKind {
    /// The as-laid length of this brick along its wall's axis.
    pub fn length(&self, spec: &WallSpec) -> f64 {
        match self {
            BrickKind::Full => spec.brick.length,
            BrickKind::Half => half_brick_length(spec),
            BrickKind::Cut { length } => *length,
        }
    }
}

/// A half brick plus half a joint equals half the bond module.
pub fn half_brick_length(spec: &WallSpec) -> f64 {
    (spec.brick.length - spec.joint) / 2.0
}

/// Which wall of the footprint a brick belongs to. South is the front
/// wall (y = 0), and the only wall that can carry an opening.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum WallSide {
    South,
    East,
    North,
    West,
}

impl WallSide {
    /// Build order within a course: front, right, back, left.
    pub fn order(&self) -> u32 {
        match self {
            WallSide::South => 0,
            WallSide::East => 1,
            WallSide::North => 2,
            WallSide::West => 3,
        }
    }

    /// Front and back walls lay bricks along x; side walls along y.
    pub fn along_x(&self) -> bool {
        matches!(self, WallSide::South | WallSide::North)
    }
}

/// One brick, placed. `x`/`y` is the plan-view minimum corner of the
/// brick's footprint; `z` is the bottom of the course it sits in.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Placement {
    pub id: u32,
    pub course: u32,
    pub wall: WallSide,
    pub kind: BrickKind,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Placement {
    /// Plan-view footprint: (x0, y0, x1, y1).
    pub fn plan_rect(&self, spec: &WallSpec) -> (f64, f64, f64, f64) {
        let t = spec.brick.width;
        let len = self.kind.length(spec);
        if self.wall.along_x() {
            (self.x, self.y, self.x + len, self.y + t)
        } else {
            (self.x, self.y, self.x + t, self.y + len)
        }
    }

    /// Position along the wall's own axis, for ordering.
    pub fn along(&self) -> f64 {
        if self.wall.along_x() { self.x } else { self.y }
    }
}

/// Strict plan-view overlap; touching edges (shared joints) do not count.
pub fn plan_rects_overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 < b.2 - EPS && a.2 > b.0 + EPS && a.1 < b.3 - EPS && a.3 > b.1 + EPS
}

/// Vertical distance from one course's bottom to the next one's.
pub fn course_height(spec: &WallSpec) -> f64 {
    spec.brick.height + spec.joint
}

/// Number of courses that fit in the wall height. The top course needs no
/// joint above it, so only the brick itself has to fit.
pub fn course_count(spec: &WallSpec) -> u32 {
    ((spec.height - spec.brick.height + EPS) / course_height(spec)) as u32 + 1
}

/// Lay out all four walls in stretcher bond with alternating corner
/// returns, cutting the front wall around the opening.
pub fn layout_walls(spec: &WallSpec) -> Vec<Placement> {
    let t = spec.brick.width;
    let m = spec.corner_return();
    let mut placements = Vec::new();
    let mut id = 0;
    let mut push = |wall: WallSide, kind: BrickKind, x: f64, y: f64, z: f64, course: u32| {
        placements.push(Placement {
            id,
            course,
            wall,
            kind,
            x,
            y,
            z,
        });
        id += 1;
    };
    for course in 0..course_count(spec) {
        let z = f64::from(course) * course_height(spec);
        // Even courses: front/back run through to the outer corners and
        // the side walls recede by one corner return; odd courses swap.
        // The return equals half the bond module, so this alternation IS
        // the stretcher stagger — corners need no special bricks.
        let through_x = course % 2 == 0;
        let (x_span, y_span) = if through_x {
            ((0.0, spec.width), (m, spec.length - m))
        } else {
            ((m, spec.width - m), (0.0, spec.length))
        };
        for (x, len) in run_segments(spec, x_span.0, x_span.1) {
            // Only the front wall carries the opening.
            for (px, plen) in cut_around_opening(spec, x, len, z) {
                push(WallSide::South, classify(spec, plen), px, 0.0, z, course);
            }
            push(
                WallSide::North,
                classify(spec, len),
                x,
                spec.length - t,
                z,
                course,
            );
        }
        for (y, len) in run_segments(spec, y_span.0, y_span.1) {
            push(
                WallSide::East,
                classify(spec, len),
                spec.width - t,
                y,
                z,
                course,
            );
            push(WallSide::West, classify(spec, len), 0.0, y, z, course);
        }
    }
    placements
}

/// Fill one wall span with stretcher bricks: full bricks from the start,
/// then the remainder as a cut brick — unless it is a sliver below
/// MIN_CUT_LENGTH, which is absorbed into joint tolerance rather than
/// asking anyone to cut and lay it.
fn run_segments(spec: &WallSpec, start: f64, end: f64) -> Vec<(f64, f64)> {
    let mut segments = Vec::new();
    let mut cursor = start;
    loop {
        let remaining = end - cursor;
        if remaining + EPS >= spec.brick.length {
            segments.push((cursor, spec.brick.length));
            cursor += spec.brick.length + spec.joint;
        } else {
            if remaining + EPS >= MIN_CUT_LENGTH {
                segments.push((cursor, remaining));
            }
            break;
        }
    }
    segments
}

/// Cut one front-wall segment around the opening. Returns the pieces to
/// keep: the whole segment if it misses the opening, otherwise the parts
/// left and right of it (cut flush against the opening edges), dropping
/// any piece below MIN_CUT_LENGTH.
fn cut_around_opening(spec: &WallSpec, x: f64, length: f64, z: f64) -> Vec<(f64, f64)> {
    let Some(op) = &spec.opening else {
        return vec![(x, length)];
    };
    let intersects_vertically = z < op.top() - EPS && z + spec.brick.height > op.sill_height + EPS;
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
