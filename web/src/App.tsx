import { useEffect, useMemo, useRef, useState } from 'react';

import InfoDialog from './components/InfoDialog';
import PlanPanel from './components/PlanPanel';
import PlaybackBar, { type Speed } from './components/PlaybackBar';
import SpecControls, { type SpecDraft } from './components/SpecControls';
import WallScene from './components/WallScene';
import { planWall } from './planner';
import type { Plan, PlanError, WallSpec } from './types';

// NL waalformaat; brick size is deliberately not editable — the demo is
// about the wall, and the planner treats brick dims as data regardless.
const WAALFORMAAT = { length: 210, height: 50, width: 100 };
const JOINT = 10;

// Steps per second at 1×; a pick and a place both count as one step.
const BASE_STEPS_PER_SECOND = 8;

function draftToSpec(draft: SpecDraft): WallSpec {
  return {
    width: draft.wallWidth,
    length: draft.wallLength,
    height: draft.wallHeight,
    brick: WAALFORMAAT,
    joint: JOINT,
    openings: draft.openings.map((op) => ({
      wall: op.wall,
      x: op.x,
      width: op.width,
      sill_height: op.kind === 'door' ? 0 : op.sill,
      height: op.height,
    })),
  };
}

function describeError(err: PlanError): string {
  switch (err.kind) {
    case 'InvalidDimension':
      return `${err.field} must be a positive number`;
    case 'WallSmallerThanBrick':
      return 'the wall must fit at least one brick';
    case 'OpeningOutOfBounds':
      return 'an opening leaves its wall or crosses a corner';
    case 'OpeningsOverlap':
      return 'two openings on the same wall overlap';
    case 'UnsupportedPlacement':
      return `planner invariant broken: placement ${err.placement_id} is unsupported`;
    case 'MalformedSpec':
      return `malformed spec: ${err.message}`;
  }
}

export default function App() {
  const [draft, setDraft] = useState<SpecDraft>({
    wallWidth: 3000,
    wallLength: 2200,
    wallHeight: 2400,
    openings: [
      { id: 1, wall: 'South', kind: 'door', x: 400, width: 900, sill: 0, height: 2000 },
      { id: 2, wall: 'South', kind: 'window', x: 1700, width: 800, sill: 600, height: 600 },
      { id: 3, wall: 'East', kind: 'window', x: 700, width: 800, sill: 600, height: 600 },
    ],
  });
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

  // Every edit re-plans synchronously through the WASM boundary; at
  // <10ms for walls far larger than the slider ranges allow, there is
  // nothing to debounce.
  const spec = useMemo(() => draftToSpec(draft), [draft]);
  const result = useMemo(() => planWall(spec), [spec]);
  // While the spec is invalid, keep showing the last buildable plan with
  // the error overlaid. This is kinder than a blank stage — and it means
  // the WebGPU canvas is never unmounted, whose teardown/re-init race
  // used to corrupt the renderer. (State adjusted during render, the
  // documented pattern for remembering previous renders' values.)
  const [lastGoodPlan, setLastGoodPlan] = useState<Plan | null>(null);
  if (result.ok && result.ok !== lastGoodPlan) {
    setLastGoodPlan(result.ok);
  }
  const plan = result.ok ?? lastGoodPlan;
  const totalSteps = plan?.steps.length ?? 0;

  // Editing the spec keeps the playback position; a shorter plan clamps it.
  // Clamping is derived rather than stored, so no state cascades on edit.
  const clampedStep = Math.min(stepIndex, totalSteps);

  // The step index is derived from elapsed time, not incremented per tick:
  // playback speed stays correct even when the browser throttles timers,
  // and high speeds advance several steps per frame.
  useEffect(() => {
    if (!playing) return;
    const rate = BASE_STEPS_PER_SECOND * speed;
    const startedAt = performance.now();
    const startStep = Math.min(stepIndexRef.current, totalSteps);
    const timer = setInterval(() => {
      const elapsed = (performance.now() - startedAt) / 1000;
      const next = startStep + Math.floor(elapsed * rate);
      if (next >= totalSteps) {
        setStepIndex(totalSteps);
        setPlaying(false); // reached the end of the plan
      } else {
        setStepIndex(next);
      }
    }, 33);
    return () => clearInterval(timer);
  }, [playing, speed, totalSteps]);

  const togglePlay = () => {
    if (!playing && clampedStep >= totalSteps) setStepIndex(0); // replay from the start
    setPlaying(!playing);
  };

  return (
    <main className="app-frame">
      <header className="masthead">
        <h1>BrickPlan</h1>
        <p>a wall spec becomes a plan becomes a wall</p>
        <InfoDialog />
      </header>
      <div className="app-body">
        <section className="content">
          {plan && <WallScene plan={plan} stepIndex={clampedStep} />}
          {result.err && (
            <div className={plan ? 'plan-error plan-error-overlay' : 'plan-error'} role="alert">
              <p>No plan: {describeError(result.err)}.</p>
              <p>
                Adjust the spec — the planner rejects anything it cannot build.
                {plan && ' Showing the last buildable state.'}
              </p>
            </div>
          )}
        </section>
        <aside className="sidebar">
          <SpecControls
            draft={draft}
            onChange={(patch) => setDraft((d) => ({ ...d, ...patch }))}
          />
          {plan && <PlanPanel plan={plan} />}
        </aside>
      </div>
      {plan && (
        <footer className="playback-footer">
          <PlaybackBar
            stepIndex={clampedStep}
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
        </footer>
      )}
    </main>
  );
}
