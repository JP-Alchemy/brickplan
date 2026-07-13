//! Sequencer: turns placements into an ordered, executable step list.

use serde::{Deserialize, Serialize};

use crate::layout::{BrickKind, Placement};
use crate::spec::{PlanError, WallSpec};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type")]
pub enum Action {
    PickBrick { kind: BrickKind },
    PlaceBrick { placement_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Step {
    pub seq: u32,
    pub action: Action,
}

/// Emit alternating pick/place steps, bottom-up and left-to-right.
///
/// Runs the support check first even though bottom-up ordering makes it
/// trivially true: a real robot planner validates preconditions instead of
/// trusting that upstream ordering happened to be correct.
pub fn sequence(spec: &WallSpec, placements: &[Placement]) -> Result<Vec<Step>, PlanError> {
    validate_support(spec, placements)?;
    let mut ordered: Vec<&Placement> = placements.iter().collect();
    ordered.sort_by(|a, b| a.course.cmp(&b.course).then(a.x.total_cmp(&b.x)));
    let mut steps = Vec::with_capacity(ordered.len() * 2);
    for p in ordered {
        steps.push(Step {
            seq: steps.len() as u32,
            action: Action::PickBrick {
                kind: p.kind.clone(),
            },
        });
        steps.push(Step {
            seq: steps.len() as u32,
            action: Action::PlaceBrick { placement_id: p.id },
        });
    }
    Ok(steps)
}

/// Support rule: a brick may only be placed if it is in course 0, rests on
/// at least part of a brick in the course below, or spans the opening as
/// part of the lintel course above it.
pub fn validate_support(spec: &WallSpec, placements: &[Placement]) -> Result<(), PlanError> {
    const EPS: f64 = 1e-6;
    let x_overlaps = |ax0: f64, ax1: f64, bx0: f64, bx1: f64| ax0 < bx1 - EPS && ax1 > bx0 + EPS;
    for p in placements {
        if p.course == 0 {
            continue;
        }
        let (px0, px1) = (p.x, p.x + p.kind.length(spec));
        let on_brick = placements
            .iter()
            .filter(|b| b.course == p.course - 1)
            .any(|b| x_overlaps(px0, px1, b.x, b.x + b.kind.length(spec)));
        // The lintel simplification: the course row directly below spans
        // the opening, so a brick over the opening counts as supported.
        // A real planner would require an actual lintel element here.
        let on_opening = spec.opening.as_ref().is_some_and(|op| {
            let below_y = f64::from(p.course - 1) * crate::layout::course_height(spec);
            let below_in_opening =
                below_y < op.top() - EPS && below_y + spec.brick.height > op.sill_height + EPS;
            below_in_opening && x_overlaps(px0, px1, op.x, op.right())
        });
        if !on_brick && !on_opening {
            return Err(PlanError::UnsupportedPlacement { placement_id: p.id });
        }
    }
    Ok(())
}
