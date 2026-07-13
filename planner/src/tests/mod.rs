//! Test helpers shared across the suite.

mod layout;
mod sequence;
mod snapshot;
mod validation;

use crate::spec::{BrickDims, Opening, WallSpec};

pub const EPS: f64 = 1e-6;

/// A plain four-walled room with default waalformaat bricks, 10mm joints.
pub fn room(width: f64, length: f64, height: f64) -> WallSpec {
    WallSpec {
        width,
        length,
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
