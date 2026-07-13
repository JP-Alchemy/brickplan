//! Input types: the wall specification the planner consumes.
//!
//! Units are millimeters throughout. The building is a rectangular
//! enclosure in plan view: origin at the south-west outer corner, x
//! pointing east (width), y pointing north (length), z pointing up.

use serde::{Deserialize, Serialize};

/// Cut-brick pieces shorter than this are absorbed into joint tolerance
/// instead of being placed. Cutting and handling slivers under ~40mm is
/// not worth it on a real wall either.
pub const MIN_CUT_LENGTH: f64 = 40.0;

const EPS: f64 = 1e-6;

/// Brick dimensions in mm. Defaults to the Dutch waalformaat (210 x 50 x 100).
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct BrickDims {
    pub length: f64,
    pub height: f64,
    pub width: f64,
}

impl Default for BrickDims {
    fn default() -> Self {
        Self {
            length: 210.0,
            height: 50.0,
            width: 100.0,
        }
    }
}

/// Which wall of the footprint something belongs to. South is the front
/// wall (y = 0).
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

/// A rectangular opening in one wall. `x` is measured along that wall's
/// own axis from its low corner (west end for front/back walls, south
/// end for side walls). A door is an opening with `sill_height` 0.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Opening {
    pub wall: WallSide,
    pub x: f64,
    pub width: f64,
    pub sill_height: f64,
    pub height: f64,
}

impl Opening {
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn top(&self) -> f64 {
        self.sill_height + self.height
    }

    /// Overlap in the wall's elevation plane; touching edges are allowed.
    fn overlaps(&self, other: &Opening) -> bool {
        self.wall == other.wall
            && self.x < other.right() - EPS
            && self.right() > other.x + EPS
            && self.sill_height < other.top() - EPS
            && self.top() > other.sill_height + EPS
    }
}

/// The full building specification: four walls on a rectangular
/// footprint, with any number of openings.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WallSpec {
    /// Outer footprint extent along x (the front and back walls).
    pub width: f64,
    /// Outer footprint extent along y (the side walls).
    pub length: f64,
    pub height: f64,
    pub brick: BrickDims,
    /// Mortar joint thickness in mm.
    pub joint: f64,
    pub openings: Vec<Opening>,
}

/// Why a spec could not be turned into a plan. Serializable so the UI can
/// show errors coming out of the WASM boundary.
#[derive(thiserror::Error, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum PlanError {
    #[error("{field} must be a positive, finite number")]
    InvalidDimension { field: String },
    #[error("the footprint must fit corner returns and at least one brick")]
    WallSmallerThanBrick,
    #[error("an opening extends outside its wall or into a corner")]
    OpeningOutOfBounds,
    #[error("two openings on the same wall overlap")]
    OpeningsOverlap,
    #[error("placement {placement_id} has no support below it")]
    UnsupportedPlacement { placement_id: u32 },
    #[error("spec could not be parsed: {message}")]
    MalformedSpec { message: String },
}

impl WallSpec {
    /// Half the bond module: a brick's width plus one joint. The corner
    /// bond works because this equals (length + joint) / 2 — the corner
    /// return of the perpendicular wall shifts a course by exactly half
    /// a module.
    pub fn corner_return(&self) -> f64 {
        self.brick.width + self.joint
    }

    /// The outer extent of a given wall along its own axis.
    pub fn wall_extent(&self, wall: WallSide) -> f64 {
        if wall.along_x() {
            self.width
        } else {
            self.length
        }
    }

    /// Check the spec is geometrically meaningful before planning.
    pub fn validate(&self) -> Result<(), PlanError> {
        let dims = [
            ("wall width", self.width),
            ("wall length", self.length),
            ("wall height", self.height),
            ("brick length", self.brick.length),
            ("brick height", self.brick.height),
            ("brick width", self.brick.width),
            ("joint", self.joint),
        ];
        for (field, value) in dims {
            check_positive(field, value)?;
        }
        // The corner bond relies on width + joint being half the module.
        // Waalformaat has this by design; reject bricks that don't, rather
        // than emitting corners that quietly fail to line up.
        if (self.brick.length - (2.0 * self.brick.width + self.joint)).abs() > EPS {
            return Err(PlanError::InvalidDimension {
                field: "brick proportions (corner bond needs length = 2 x width + joint)".into(),
            });
        }
        let min_extent = 2.0 * self.corner_return() + MIN_CUT_LENGTH;
        if self.width < min_extent || self.length < min_extent || self.height < self.brick.height {
            return Err(PlanError::WallSmallerThanBrick);
        }
        for op in &self.openings {
            check_positive("opening width", op.width)?;
            check_positive("opening height", op.height)?;
            for (field, value) in [("opening x", op.x), ("opening sill height", op.sill_height)] {
                if !value.is_finite() {
                    return Err(PlanError::InvalidDimension {
                        field: field.into(),
                    });
                }
            }
            // Openings must stay clear of the corner returns, where the
            // perpendicular walls' bricks turn into this wall's bond.
            if op.x < self.corner_return() - EPS
                || op.sill_height < 0.0
                || op.right() > self.wall_extent(op.wall) - self.corner_return() + EPS
                || op.top() > self.height
            {
                return Err(PlanError::OpeningOutOfBounds);
            }
        }
        for (i, a) in self.openings.iter().enumerate() {
            if self.openings[i + 1..].iter().any(|b| a.overlaps(b)) {
                return Err(PlanError::OpeningsOverlap);
            }
        }
        Ok(())
    }
}

fn check_positive(field: &str, value: f64) -> Result<(), PlanError> {
    if !value.is_finite() || value <= 0.0 {
        return Err(PlanError::InvalidDimension {
            field: field.into(),
        });
    }
    Ok(())
}
