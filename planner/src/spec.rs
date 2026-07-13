//! Input types: the wall specification the planner consumes.
//!
//! Units are millimeters throughout. The wall coordinate system has its
//! origin at the bottom-left corner, x pointing right, y pointing up.

use serde::{Deserialize, Serialize};

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

/// A rectangular opening in the wall. A door is an opening with
/// `sill_height` 0; a window sits higher up.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Opening {
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
}

/// The full wall specification: everything the planner needs as input.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct WallSpec {
    pub width: f64,
    pub height: f64,
    pub brick: BrickDims,
    /// Mortar joint thickness in mm.
    pub joint: f64,
    pub opening: Option<Opening>,
}

/// Why a spec could not be turned into a plan. Serializable so the UI can
/// show errors coming out of the WASM boundary.
#[derive(thiserror::Error, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "kind")]
pub enum PlanError {
    #[error("{field} must be a positive, finite number")]
    InvalidDimension { field: String },
    #[error("wall must be at least one brick wide and one brick tall")]
    WallSmallerThanBrick,
    #[error("opening extends outside the wall")]
    OpeningOutOfBounds,
    #[error("placement {placement_id} has no support below it")]
    UnsupportedPlacement { placement_id: u32 },
    #[error("spec could not be parsed: {message}")]
    MalformedSpec { message: String },
}

impl WallSpec {
    /// Check the spec is geometrically meaningful before planning.
    pub fn validate(&self) -> Result<(), PlanError> {
        let dims = [
            ("wall width", self.width),
            ("wall height", self.height),
            ("brick length", self.brick.length),
            ("brick height", self.brick.height),
            ("brick width", self.brick.width),
            ("joint", self.joint),
        ];
        for (field, value) in dims {
            check_positive(field, value)?;
        }
        if self.width < self.brick.length || self.height < self.brick.height {
            return Err(PlanError::WallSmallerThanBrick);
        }
        if let Some(op) = &self.opening {
            check_positive("opening width", op.width)?;
            check_positive("opening height", op.height)?;
            for (field, value) in [("opening x", op.x), ("opening sill height", op.sill_height)] {
                if !value.is_finite() {
                    return Err(PlanError::InvalidDimension {
                        field: field.into(),
                    });
                }
            }
            if op.x < 0.0
                || op.sill_height < 0.0
                || op.right() > self.width
                || op.top() > self.height
            {
                return Err(PlanError::OpeningOutOfBounds);
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
