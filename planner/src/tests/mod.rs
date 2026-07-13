//! Test helpers shared across the suite.

mod layout;
mod sequence;
mod snapshot;
mod validation;

use crate::layout::Placement;
use crate::spec::{BrickDims, Opening, WallSpec};

pub const EPS: f64 = 1e-6;

/// A plain wall with default waalformaat bricks and 10mm joints.
pub fn wall(width: f64, height: f64) -> WallSpec {
    WallSpec {
        width,
        height,
        brick: BrickDims::default(),
        joint: 10.0,
        opening: None,
    }
}

pub fn with_opening(mut spec: WallSpec, opening: Opening) -> WallSpec {
    spec.opening = Some(opening);
    spec
}

pub fn door(x: f64, width: f64, height: f64) -> Opening {
    Opening {
        x,
        width,
        sill_height: 0.0,
        height,
    }
}

pub fn window(x: f64, width: f64, sill_height: f64, height: f64) -> Opening {
    Opening {
        x,
        width,
        sill_height,
        height,
    }
}

/// Axis-aligned bounding box of a placement: (x0, y0, x1, y1).
pub fn rect(p: &Placement, spec: &WallSpec) -> (f64, f64, f64, f64) {
    (p.x, p.y, p.x + p.kind.length(spec), p.y + spec.brick.height)
}

/// Strict interior overlap of two AABBs; touching edges do not count.
pub fn rects_overlap(a: (f64, f64, f64, f64), b: (f64, f64, f64, f64)) -> bool {
    a.0 < b.2 - EPS && a.2 > b.0 + EPS && a.1 < b.3 - EPS && a.3 > b.1 + EPS
}
