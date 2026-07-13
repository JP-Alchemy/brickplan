import { useMemo } from 'react';

import type { Plan } from '../types';

// Makes "plans as data" literally visible: the same JSON the planner
// hands the simulator, inspectable and downloadable.

const PREVIEW_STEPS = 24;

interface PlanPanelProps {
  plan: Plan;
}

export default function PlanPanel({ plan }: PlanPanelProps) {
  const preview = useMemo(
    () =>
      JSON.stringify(
        {
          spec: plan.spec,
          stats: plan.stats,
          steps: plan.steps.slice(0, PREVIEW_STEPS),
        },
        null,
        2,
      ),
    [plan],
  );

  const download = () => {
    const blob = new Blob([JSON.stringify(plan, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'plan.json';
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <details className="plan-panel">
      <summary>Plan JSON</summary>
      <div className="plan-panel-body">
        <p className="plan-panel-note">
          spec, stats, and the first {Math.min(PREVIEW_STEPS, plan.steps.length)} of{' '}
          {plan.steps.length} steps — placements omitted here, included in the download
        </p>
        <pre>{preview}</pre>
        <button onClick={download}>Download full plan</button>
      </div>
    </details>
  );
}
