import WallCanvas from './components/WallCanvas';
import { planWall } from './planner';
import type { WallSpec } from './types';

// M3: a hardcoded spec rendered as a completed wall. Live controls follow.
const demoSpec: WallSpec = {
  width: 3000,
  height: 2400,
  brick: { length: 210, height: 50, width: 100 },
  joint: 10,
  opening: { x: 1000, width: 800, sill_height: 600, height: 600 },
};

const result = planWall(demoSpec);

export default function App() {
  if (result.err) {
    return <p>Planning failed: {JSON.stringify(result.err)}</p>;
  }
  const plan = result.ok;
  const { stats } = plan;
  return (
    <main>
      <header className="masthead">
        <h1>BrickPlan</h1>
        <p>a wall spec becomes a plan becomes a wall</p>
      </header>
      <WallCanvas plan={plan} />
      <p className="stats-line">
        {plan.spec.width} × {plan.spec.height} mm · {stats.courses} courses ·{' '}
        {plan.placements.length} bricks ({stats.full_bricks} full, {stats.half_bricks} half,{' '}
        {stats.cut_bricks} cut) · {plan.steps.length} steps
      </p>
    </main>
  );
}
