import { useState } from 'react';

import type { WallSide } from '../types';

// Spec editing state. The draft is UI-shaped (opening kind is explicit
// rather than inferred from sill height); App converts it to a WallSpec.

export type OpeningKind = 'door' | 'window';

export interface OpeningDraft {
  id: number;
  wall: WallSide;
  kind: OpeningKind;
  x: number;
  width: number;
  sill: number;
  height: number;
}

export interface SpecDraft {
  wallWidth: number;
  wallLength: number;
  wallHeight: number;
  openings: OpeningDraft[];
}

/// The first 110mm at each end of a wall belong to the corner return of
/// the perpendicular walls; openings must stay clear of them.
const CORNER_RETURN = 110;

const WALL_LABELS: Record<WallSide, string> = {
  South: 'front',
  East: 'right',
  North: 'back',
  West: 'left',
};

const KIND_DEFAULTS: Record<OpeningKind, Pick<OpeningDraft, 'sill' | 'height'>> = {
  door: { sill: 0, height: 2000 },
  window: { sill: 600, height: 600 },
};

interface SpecControlsProps {
  draft: SpecDraft;
  onChange: (patch: Partial<SpecDraft>) => void;
}

export default function SpecControls({ draft, onChange }: SpecControlsProps) {
  const [selectedId, setSelectedId] = useState<number | null>(
    draft.openings[0]?.id ?? null,
  );

  const wallExtent = (wall: WallSide) =>
    wall === 'South' || wall === 'North' ? draft.wallWidth : draft.wallLength;

  const patchOpening = (id: number, patch: Partial<OpeningDraft>) =>
    onChange({
      openings: draft.openings.map((op) => (op.id === id ? { ...op, ...patch } : op)),
    });

  const addOpening = () => {
    const id = Math.max(0, ...draft.openings.map((op) => op.id)) + 1;
    const width = 800;
    // First wall with room for it, scanning front, right, back, left;
    // if the building is full, land on the front and let the planner say so.
    const slot = (['South', 'East', 'North', 'West'] as const)
      .flatMap((wall) => {
        const taken = draft.openings.filter((op) => op.wall === wall);
        const candidates = [
          Math.round((wallExtent(wall) - width) / 20) * 10, // centered first
          CORNER_RETURN,
          ...taken.map((op) => op.x + op.width + 100),
        ];
        return candidates
          .filter(
            (x) =>
              x >= CORNER_RETURN &&
              x + width <= wallExtent(wall) - CORNER_RETURN &&
              taken.every((op) => x >= op.x + op.width || x + width <= op.x),
          )
          .slice(0, 1)
          .map((x) => ({ wall, x }));
      })
      .at(0) ?? { wall: 'South' as WallSide, x: CORNER_RETURN };
    onChange({
      openings: [
        ...draft.openings,
        { id, wall: slot.wall, kind: 'door', x: slot.x, width, ...KIND_DEFAULTS.door },
      ],
    });
    setSelectedId(id);
  };

  const removeOpening = (id: number) =>
    onChange({ openings: draft.openings.filter((op) => op.id !== id) });

  return (
    <div className="spec-controls">
      <h2>Walls</h2>
      <SliderField
        label="Width"
        value={draft.wallWidth}
        min={500}
        max={6000}
        onChange={(wallWidth) => onChange({ wallWidth })}
      />
      <SliderField
        label="Length"
        value={draft.wallLength}
        min={500}
        max={6000}
        onChange={(wallLength) => onChange({ wallLength })}
      />
      <SliderField
        label="Height"
        value={draft.wallHeight}
        min={300}
        max={3000}
        onChange={(wallHeight) => onChange({ wallHeight })}
      />

      <h2>Openings</h2>
      <ul className="opening-list">
        {draft.openings.map((op) => {
          const selected = op.id === selectedId;
          return (
            <li key={op.id} className={selected ? 'opening selected' : 'opening'}>
              <div className="opening-row">
                <button
                  className="opening-title"
                  onClick={() => setSelectedId(selected ? null : op.id)}
                  aria-expanded={selected}
                >
                  {op.kind} · {WALL_LABELS[op.wall]} wall
                </button>
                <button
                  className="opening-remove"
                  aria-label={`Remove ${op.kind}`}
                  onClick={() => removeOpening(op.id)}
                >
                  ×
                </button>
              </div>
              {selected && (
                <div className="opening-editor">
                  <div className="kind-toggle" role="group" aria-label="Wall">
                    {(['South', 'East', 'North', 'West'] as const).map((wall) => (
                      <button
                        key={wall}
                        className={wall === op.wall ? 'active' : ''}
                        onClick={() => patchOpening(op.id, { wall })}
                      >
                        {WALL_LABELS[wall]}
                      </button>
                    ))}
                  </div>
                  <div className="kind-toggle" role="group" aria-label="Opening type">
                    {(['door', 'window'] as const).map((kind) => (
                      <button
                        key={kind}
                        className={kind === op.kind ? 'active' : ''}
                        onClick={() =>
                          patchOpening(op.id, { kind, ...KIND_DEFAULTS[kind] })
                        }
                      >
                        {kind}
                      </button>
                    ))}
                  </div>
                  <SliderField
                    label="Position"
                    value={op.x}
                    min={CORNER_RETURN}
                    max={wallExtent(op.wall)}
                    onChange={(x) => patchOpening(op.id, { x })}
                  />
                  <SliderField
                    label="Width"
                    value={op.width}
                    min={300}
                    max={2400}
                    onChange={(width) => patchOpening(op.id, { width })}
                  />
                  <SliderField
                    label="Height"
                    value={op.height}
                    min={300}
                    max={2800}
                    onChange={(height) => patchOpening(op.id, { height })}
                  />
                  {op.kind === 'window' && (
                    <SliderField
                      label="Sill height"
                      value={op.sill}
                      min={100}
                      max={2000}
                      onChange={(sill) => patchOpening(op.id, { sill })}
                    />
                  )}
                </div>
              )}
            </li>
          );
        })}
      </ul>
      <button className="opening-add" onClick={addOpening}>
        + Add opening
      </button>
    </div>
  );
}

interface SliderFieldProps {
  label: string;
  value: number;
  min: number;
  max: number;
  onChange: (value: number) => void;
}

function SliderField({ label, value, min, max, onChange }: SliderFieldProps) {
  const handle = (raw: string) => {
    const n = Number(raw);
    if (Number.isFinite(n)) onChange(n);
  };
  return (
    <div className="field">
      <div className="field-head">
        <label>{label}</label>
        <span className="field-value">
          <input type="number" value={value} step={10} onChange={(e) => handle(e.target.value)} />
          mm
        </span>
      </div>
      <input
        type="range"
        value={value}
        min={min}
        max={max}
        step={10}
        aria-label={label}
        onChange={(e) => handle(e.target.value)}
      />
    </div>
  );
}
