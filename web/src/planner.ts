// Typed wrapper around the WASM planner. The module initializes at import
// time (top-level await), so by the time any component calls planWall the
// planner is ready and every call is synchronous.

import init, { plan_wall } from 'planner';

import type { PlanResult, WallSpec } from './types';

await init();

export function planWall(spec: WallSpec): PlanResult {
  return plan_wall(spec) as PlanResult;
}
