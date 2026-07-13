import { planWall } from './planner';
import type { WallSpec } from './types';

// M2 proof: a hardcoded spec planned live via WASM, plan on the console.
const demoSpec: WallSpec = {
  width: 3000,
  height: 2400,
  brick: { length: 210, height: 50, width: 100 },
  joint: 10,
  opening: { x: 1000, width: 800, sill_height: 600, height: 600 },
};

const result = planWall(demoSpec);
console.log('planWall result', result);

export default function App() {
  if (result.err) {
    return <p>Planning failed: {JSON.stringify(result.err)}</p>;
  }
  const { stats, placements, steps } = result.ok;
  return (
    <main>
      <h1>BrickPlan</h1>
      <p>
        Planned a {demoSpec.width}×{demoSpec.height}mm wall in Rust/WASM: {stats.courses} courses,{' '}
        {placements.length} bricks ({stats.full_bricks} full, {stats.half_bricks} half,{' '}
        {stats.cut_bricks} cut), {steps.length} steps. Full plan on the console.
      </p>
    </main>
  );
}
