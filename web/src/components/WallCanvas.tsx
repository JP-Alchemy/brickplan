import { useMemo } from 'react';

import { brickLength } from '../geometry';
import type { Plan } from '../types';

// The SVG coordinate space is the wall's own millimeter space, so brick
// positions pass through untouched; only y flips (SVG grows downward).

interface WallCanvasProps {
  plan: Plan;
  /** How many bricks are on the wall at the current playback position. */
  placedCount: number;
}

export default function WallCanvas({ plan, placedCount }: WallCanvasProps) {
  const { spec } = plan;
  const margin = 20;

  // Replay order comes from the steps, not from the placements array:
  // the plan is the contract, and the simulator only executes it.
  const placeOrder = useMemo(() => {
    const byId = new Map(plan.placements.map((p) => [p.id, p]));
    return plan.steps.flatMap((s) =>
      s.action.type === 'PlaceBrick' ? [byId.get(s.action.placement_id)!] : [],
    );
  }, [plan]);

  const visible = placeOrder.slice(0, placedCount);

  return (
    <svg
      className="wall-canvas"
      viewBox={`${-margin} ${-margin} ${spec.width + 2 * margin} ${spec.height + 2 * margin}`}
      role="img"
      aria-label={`Brick wall, ${spec.width} by ${spec.height} millimeters`}
    >
      <rect x={0} y={0} width={spec.width} height={spec.height} fill="var(--mortar)" />
      {spec.opening && (
        <rect
          x={spec.opening.x}
          y={spec.height - spec.opening.sill_height - spec.opening.height}
          width={spec.opening.width}
          height={spec.opening.height}
          fill="var(--bg)"
          stroke="var(--hairline)"
          strokeWidth={2}
        />
      )}
      {visible.map((p, i) => (
        <rect
          key={p.id}
          className={i === placedCount - 1 ? 'brick just-placed' : 'brick'}
          x={p.x}
          y={spec.height - p.y - spec.brick.height}
          width={brickLength(p.kind, spec)}
          height={spec.brick.height}
          fill={p.kind.type === 'Cut' ? 'var(--brick-cut)' : 'var(--brick)'}
        />
      ))}
    </svg>
  );
}
