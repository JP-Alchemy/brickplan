const SPEEDS = [1, 4, 16] as const;
export type Speed = (typeof SPEEDS)[number];

interface PlaybackBarProps {
  stepIndex: number;
  totalSteps: number;
  playing: boolean;
  speed: Speed;
  onTogglePlay: () => void;
  onScrub: (stepIndex: number) => void;
  onSpeedChange: (speed: Speed) => void;
}

export default function PlaybackBar({
  stepIndex,
  totalSteps,
  playing,
  speed,
  onTogglePlay,
  onScrub,
  onSpeedChange,
}: PlaybackBarProps) {
  return (
    <div className="playback-bar">
      <button className="play-toggle" onClick={onTogglePlay}>
        {playing ? 'Pause' : 'Play'}
      </button>
      <div className="speed-toggle" role="group" aria-label="Playback speed">
        {SPEEDS.map((s) => (
          <button
            key={s}
            className={s === speed ? 'active' : ''}
            onClick={() => onSpeedChange(s)}
          >
            {s}×
          </button>
        ))}
      </div>
      <input
        type="range"
        min={0}
        max={totalSteps}
        value={stepIndex}
        onChange={(e) => onScrub(Number(e.target.value))}
        aria-label="Step position"
      />
      <span className="step-counter">
        Step {stepIndex} / {totalSteps}
      </span>
    </div>
  );
}
