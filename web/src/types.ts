// Hand-written mirrors of the planner's Rust types (planner/src/spec.rs,
// layout.rs, sequence.rs, plan.rs). Kept small and in sync by the JSON
// snapshot test on the Rust side; field names are the serde defaults.

export interface BrickDims {
  length: number;
  height: number;
  width: number;
}

export interface Opening {
  x: number;
  width: number;
  sill_height: number;
  height: number;
}

export interface WallSpec {
  width: number;
  height: number;
  brick: BrickDims;
  joint: number;
  opening: Opening | null;
}

export type BrickKind =
  | { type: 'Full' }
  | { type: 'Half' }
  | { type: 'Cut'; length: number };

export interface Placement {
  id: number;
  course: number;
  kind: BrickKind;
  x: number;
  y: number;
}

export type Action =
  | { type: 'PickBrick'; kind: BrickKind }
  | { type: 'PlaceBrick'; placement_id: number };

export interface Step {
  seq: number;
  action: Action;
}

export interface PlanStats {
  courses: number;
  full_bricks: number;
  half_bricks: number;
  cut_bricks: number;
}

export interface Plan {
  spec: WallSpec;
  placements: Placement[];
  steps: Step[];
  stats: PlanStats;
}

export type PlanError =
  | { kind: 'InvalidDimension'; field: string }
  | { kind: 'WallSmallerThanBrick' }
  | { kind: 'OpeningOutOfBounds' }
  | { kind: 'UnsupportedPlacement'; placement_id: number }
  | { kind: 'MalformedSpec'; message: string };

export type PlanResult = { ok: Plan; err?: never } | { ok?: never; err: PlanError };
