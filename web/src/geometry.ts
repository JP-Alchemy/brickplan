// Small geometry helpers shared by the rendering components. These mirror
// definitions in the planner; the plan carries positions, so the UI only
// ever needs lengths, never bond logic.

import type { BrickKind, WallSpec } from './types';

export function brickLength(kind: BrickKind, spec: WallSpec): number {
  switch (kind.type) {
    case 'Full':
      return spec.brick.length;
    case 'Half':
      return (spec.brick.length - spec.joint) / 2;
    case 'Cut':
      return kind.length;
  }
}
