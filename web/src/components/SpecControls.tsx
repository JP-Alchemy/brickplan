// Spec editing state. The draft is UI-shaped (opening kind is explicit
// rather than inferred from sill height); App converts it to a WallSpec.

export type OpeningKind = 'none' | 'door' | 'window';

export interface OpeningDraft {
  x: number;
  width: number;
  sill: number;
  height: number;
}

export interface SpecDraft {
  wallWidth: number;
  wallHeight: number;
  openingKind: OpeningKind;
  opening: OpeningDraft;
}

const OPENING_DEFAULTS: Record<'door' | 'window', Pick<OpeningDraft, 'sill' | 'height'>> = {
  door: { sill: 0, height: 2000 },
  window: { sill: 600, height: 600 },
};

interface SpecControlsProps {
  draft: SpecDraft;
  onChange: (patch: Partial<SpecDraft>) => void;
}

export default function SpecControls({ draft, onChange }: SpecControlsProps) {
  const { opening } = draft;
  const patchOpening = (patch: Partial<OpeningDraft>) =>
    onChange({ opening: { ...opening, ...patch } });

  const switchKind = (kind: OpeningKind) => {
    if (kind === 'none' || kind === draft.openingKind) {
      onChange({ openingKind: kind });
    } else {
      // Keep position and width; reset heights to something sensible.
      onChange({ openingKind: kind, opening: { ...opening, ...OPENING_DEFAULTS[kind] } });
    }
  };

  return (
    <div className="spec-controls">
      <h2>Wall</h2>
      <SliderField
        label="Width"
        value={draft.wallWidth}
        min={500}
        max={6000}
        onChange={(wallWidth) => onChange({ wallWidth })}
      />
      <SliderField
        label="Height"
        value={draft.wallHeight}
        min={300}
        max={3000}
        onChange={(wallHeight) => onChange({ wallHeight })}
      />

      <h2>Opening</h2>
      <div className="kind-toggle" role="group" aria-label="Opening type">
        {(['none', 'door', 'window'] as const).map((kind) => (
          <button
            key={kind}
            className={kind === draft.openingKind ? 'active' : ''}
            onClick={() => switchKind(kind)}
          >
            {kind}
          </button>
        ))}
      </div>
      {draft.openingKind !== 'none' && (
        <>
          <SliderField
            label="Position"
            value={opening.x}
            min={0}
            max={draft.wallWidth}
            onChange={(x) => patchOpening({ x })}
          />
          <SliderField
            label="Width"
            value={opening.width}
            min={300}
            max={2400}
            onChange={(width) => patchOpening({ width })}
          />
          <SliderField
            label="Height"
            value={opening.height}
            min={300}
            max={2800}
            onChange={(height) => patchOpening({ height })}
          />
          {draft.openingKind === 'window' && (
            <SliderField
              label="Sill height"
              value={opening.sill}
              min={100}
              max={2000}
              onChange={(sill) => patchOpening({ sill })}
            />
          )}
        </>
      )}
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
