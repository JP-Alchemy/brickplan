//! Test helpers shared across the suite.

mod layout;
mod sequence;
mod snapshot;
mod validation;

use crate::spec::{BrickDims, Opening, WallSide, WallSpec};

pub const EPS: f64 = 1e-6;

/// A plain four-walled room with default waalformaat bricks, 10mm joints.
pub fn room(width: f64, length: f64, height: f64) -> WallSpec {
    WallSpec {
        width,
        length,
        height,
        brick: BrickDims::default(),
        joint: 10.0,
        openings: vec![],
    }
}

pub fn with_openings(mut spec: WallSpec, openings: Vec<Opening>) -> WallSpec {
    spec.openings = openings;
    spec
}

pub fn with_opening(spec: WallSpec, opening: Opening) -> WallSpec {
    with_openings(spec, vec![opening])
}

pub fn door(wall: WallSide, x: f64, width: f64, height: f64) -> Opening {
    Opening {
        wall,
        x,
        width,
        sill_height: 0.0,
        height,
    }
}

pub fn window(wall: WallSide, x: f64, width: f64, sill_height: f64, height: f64) -> Opening {
    Opening {
        wall,
        x,
        width,
        sill_height,
        height,
    }
}
