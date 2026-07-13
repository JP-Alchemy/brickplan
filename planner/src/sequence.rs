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

/// Emit alternating pick/place steps: bottom-up by course, walls in
/// front-right-back-left order, and along each wall's axis.
///
/// Runs the support check first even though bottom-up ordering makes it
/// trivially true: a real robot planner validates preconditions instead of
/// trusting that upstream ordering happened to be correct.
pub fn sequence(spec: &WallSpec, placements: &[Placement]) -> Result<Vec<Step>, PlanError> {
    validate_support(spec, placements)?;
    let mut ordered: Vec<&Placement> = placements.iter().collect();
    ordered.sort_by(|a, b| {
        a.course
            .cmp(&b.course)
            .then(a.wall.order().cmp(&b.wall.order()))
            .then(a.along().total_cmp(&b.along()))
    });
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
/// at least part of a brick in the course below (on any wall — corners
/// rest on the perpendicular wall's bricks), or spans the opening as part
/// of the lintel course above it.
pub fn validate_support(spec: &WallSpec, placements: &[Placement]) -> Result<(), PlanError> {
    const EPS: f64 = 1e-6;
    for p in placements {
        if p.course == 0 {
            continue;
        }
        let rect = p.plan_rect(spec);
        let on_brick = placements
            .iter()
            .filter(|b| b.course == p.course - 1)
            .any(|b| crate::layout::plan_rects_overlap(rect, b.plan_rect(spec)));
        // The lintel simplification: the front-wall course directly above
        // the opening spans it, so a brick over the opening counts as
        // supported. A real planner would require an actual lintel element.
        let on_opening = p.wall == crate::layout::WallSide::South
            && spec.opening.as_ref().is_some_and(|op| {
                let below_z = f64::from(p.course - 1) * crate::layout::course_height(spec);
                let below_in_opening =
                    below_z < op.top() - EPS && below_z + spec.brick.height > op.sill_height + EPS;
                below_in_opening && p.x < op.right() - EPS && p.x + p.kind.length(spec) > op.x + EPS
            });
        if !on_brick && !on_opening {
            return Err(PlanError::UnsupportedPlacement { placement_id: p.id });
        }
    }
    Ok(())
}
