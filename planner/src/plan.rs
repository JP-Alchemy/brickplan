//! The planner entry point and its output: a plan is pure data.

use serde::{Deserialize, Serialize};

use crate::layout::{BrickKind, Placement};
use crate::sequence::Step;
use crate::spec::{PlanError, WallSpec};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct PlanStats {
    pub courses: u32,
    pub full_bricks: u32,
    pub half_bricks: u32,
    pub cut_bricks: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Plan {
    pub spec: WallSpec,
    pub placements: Vec<Placement>,
    pub steps: Vec<Step>,
    pub stats: PlanStats,
}

/// Turn a wall spec into an executable plan: validate, lay out, sequence.
pub fn plan(spec: WallSpec) -> Result<Plan, PlanError> {
    spec.validate()?;
    let placements = crate::layout::layout_walls(&spec);
    let steps = crate::sequence::sequence(&spec, &placements)?;
    let stats = PlanStats {
        courses: crate::layout::course_count(&spec),
        full_bricks: count_kind(&placements, |k| matches!(k, BrickKind::Full)),
        half_bricks: count_kind(&placements, |k| matches!(k, BrickKind::Half)),
        cut_bricks: count_kind(&placements, |k| matches!(k, BrickKind::Cut { .. })),
    };
    Ok(Plan {
        spec,
        placements,
        steps,
        stats,
    })
}

fn count_kind(placements: &[Placement], pred: fn(&BrickKind) -> bool) -> u32 {
    placements.iter().filter(|p| pred(&p.kind)).count() as u32
}
