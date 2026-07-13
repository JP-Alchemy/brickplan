import { brickLength } from '../geometry';
import type { Plan } from '../types';

// The SVG coordinate space is the wall's own millimeter space, so brick
// positions pass through untouched; only y flips (SVG grows downward).

interface WallCanvasProps {
  plan: Plan;
}

export default function WallCanvas({ plan }: WallCanvasProps) {
  const { spec, placements } = plan;
  const margin = 20;

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
      {placements.map((p) => (
        <rect
          key={p.id}
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
