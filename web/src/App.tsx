import { useEffect, useRef, useState } from 'react';

import PlaybackBar, { type Speed } from './components/PlaybackBar';
import WallCanvas from './components/WallCanvas';
import { planWall } from './planner';
import type { WallSpec } from './types';

// M4: playback over a hardcoded spec. Live editing follows.
const demoSpec: WallSpec = {
  width: 3000,
  height: 2400,
  brick: { length: 210, height: 50, width: 100 },
  joint: 10,
  opening: { x: 1000, width: 800, sill_height: 600, height: 600 },
};

const result = planWall(demoSpec);

// Steps per second at 1×; a pick and a place both count as one step.
const BASE_STEPS_PER_SECOND = 8;

export default function App() {
  const [stepIndex, setStepIndexState] = useState(0);
  const [playing, setPlaying] = useState(false);
  const [speed, setSpeed] = useState<Speed>(4);

  // Mirrored in a ref so the playback effect can rebase from the current
  // position (on play or speed change) without re-running per step.
  const stepIndexRef = useRef(0);
  const setStepIndex = (i: number) => {
    stepIndexRef.current = i;
    setStepIndexState(i);
  };

  const plan = result.ok ?? null;
  const totalSteps = plan?.steps.length ?? 0;

  // The step index is derived from elapsed time, not incremented per tick:
  // playback speed stays correct even when the browser throttles timers,
  // and high speeds advance several steps per frame.
  useEffect(() => {
    if (!playing) return;
    const rate = BASE_STEPS_PER_SECOND * speed;
    const startedAt = performance.now();
    const startStep = stepIndexRef.current;
    const timer = setInterval(() => {
      const elapsed = (performance.now() - startedAt) / 1000;
      setStepIndex(Math.min(startStep + Math.floor(elapsed * rate), totalSteps));
    }, 33);
    return () => clearInterval(timer);
  }, [playing, speed, totalSteps]);

  useEffect(() => {
    if (stepIndex >= totalSteps) setPlaying(false);
  }, [stepIndex, totalSteps]);

  if (!plan) {
    return <p>Planning failed: {JSON.stringify(result.err)}</p>;
  }

  const togglePlay = () => {
    if (!playing && stepIndex >= totalSteps) setStepIndex(0); // replay from the start
    setPlaying(!playing);
  };

  const { stats } = plan;
  return (
    <main>
      <header className="masthead">
        <h1>BrickPlan</h1>
        <p>a wall spec becomes a plan becomes a wall</p>
      </header>
      <WallCanvas plan={plan} placedCount={Math.floor(stepIndex / 2)} />
      <PlaybackBar
        stepIndex={stepIndex}
        totalSteps={totalSteps}
        playing={playing}
        speed={speed}
        onTogglePlay={togglePlay}
        onScrub={(i) => {
          setPlaying(false);
          setStepIndex(i);
        }}
        onSpeedChange={setSpeed}
      />
      <p className="stats-line">
        {plan.spec.width} × {plan.spec.height} mm · {stats.courses} courses ·{' '}
        {plan.placements.length} bricks ({stats.full_bricks} full, {stats.half_bricks} half,{' '}
        {stats.cut_bricks} cut) · {plan.steps.length} steps
      </p>
    </main>
  );
}
