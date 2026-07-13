import { useEffect, useMemo, useRef, useState } from 'react';

import InfoDialog from './components/InfoDialog';
import PlanPanel from './components/PlanPanel';
import PlaybackBar, { type Speed } from './components/PlaybackBar';
import SpecControls, { type SpecDraft } from './components/SpecControls';
import WallCanvas from './components/WallCanvas';
import { planWall } from './planner';
import type { PlanError, WallSpec } from './types';

// NL waalformaat; brick size is deliberately not editable — the demo is
// about the wall, and the planner treats brick dims as data regardless.
const WAALFORMAAT = { length: 210, height: 50, width: 100 };
const JOINT = 10;

// Steps per second at 1×; a pick and a place both count as one step.
const BASE_STEPS_PER_SECOND = 8;

function draftToSpec(draft: SpecDraft): WallSpec {
  return {
    width: draft.wallWidth,
    height: draft.wallHeight,
    brick: WAALFORMAAT,
    joint: JOINT,
    opening:
      draft.openingKind === 'none'
        ? null
        : {
            x: draft.opening.x,
            width: draft.opening.width,
            sill_height: draft.openingKind === 'door' ? 0 : draft.opening.sill,
            height: draft.opening.height,
          },
  };
}

function describeError(err: PlanError): string {
  switch (err.kind) {
    case 'InvalidDimension':
      return `${err.field} must be a positive number`;
    case 'WallSmallerThanBrick':
      return 'the wall must fit at least one brick';
    case 'OpeningOutOfBounds':
      return 'the opening extends outside the wall';
    case 'UnsupportedPlacement':
      return `planner invariant broken: placement ${err.placement_id} is unsupported`;
    case 'MalformedSpec':
      return `malformed spec: ${err.message}`;
  }
}

export default function App() {
  const [draft, setDraft] = useState<SpecDraft>({
    wallWidth: 3000,
    wallHeight: 2400,
    openingKind: 'window',
    opening: { x: 1000, width: 800, sill: 600, height: 600 },
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
  const plan = result.ok ?? null;
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
      <div className="layout">
        <div className="stage">
          {plan ? (
            <>
              <WallCanvas plan={plan} placedCount={Math.floor(clampedStep / 2)} />
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
              <p className="stats-line">
                {plan.spec.width} × {plan.spec.height} mm · {plan.stats.courses} courses ·{' '}
                {plan.placements.length} bricks ({plan.stats.full_bricks} full,{' '}
                {plan.stats.half_bricks} half, {plan.stats.cut_bricks} cut) · {plan.steps.length}{' '}
                steps
              </p>
              <PlanPanel plan={plan} />
            </>
          ) : (
            <div className="plan-error" role="alert">
              <p>No plan: {describeError(result.err!)}.</p>
              <p>Adjust the spec — the planner rejects anything it cannot build.</p>
            </div>
          )}
        </div>
        <SpecControls draft={draft} onChange={(patch) => setDraft((d) => ({ ...d, ...patch }))} />
      </div>
    </main>
  );
}
