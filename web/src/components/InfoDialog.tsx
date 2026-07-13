import { useRef } from 'react';

// A native <dialog>: Escape, the backdrop, and the close button all
// dismiss it, and focus handling comes for free.

export default function InfoDialog() {
  const ref = useRef<HTMLDialogElement>(null);
  return (
    <>
      <button
        className="info-button"
        aria-label="About this page"
        onClick={() => ref.current?.showModal()}
      >
        i
      </button>
      <dialog
        ref={ref}
        className="info-dialog"
        aria-label="About BrickPlan"
        onClick={(e) => {
          // Clicks on the backdrop land on the dialog element itself.
          if (e.target === ref.current) ref.current?.close();
        }}
      >
        <div className="info-body">
          <h2>About</h2>
          <p>
            BrickPlan is a small homage to Monumental&rsquo;s{' '}
            <a
              href="https://buildmonumental.substack.com/p/plans-as-data"
              target="_blank"
              rel="noreferrer"
            >
              &ldquo;Plans as Data&rdquo;
            </a>{' '}
            idea. A Rust planner — compiled to WebAssembly, running entirely in your browser —
            turns four walls and an opening into a step-by-step placement plan as plain JSON, and
            this page builds that plan the way a robot would execute it. Nothing leaves your
            machine; there is no backend.{' '}
            <a href="https://github.com/JP-Alchemy/brickplan" target="_blank" rel="noreferrer">
              Source on GitHub
            </a>
            .
          </p>
          <h2>How to use</h2>
          <ul>
            <li>Shape the walls with the sliders — every change re-plans instantly.</li>
            <li>
              Add doors and windows to any wall, as many as fit; bricks are cut to land flush
              against each one.
            </li>
            <li>Drag to orbit, scroll to zoom — the corners are worth a close look.</li>
            <li>Press Play to watch the placement sequence, scrub freely, change speed.</li>
            <li>Open &ldquo;Plan JSON&rdquo; to inspect the raw plan, or download all of it.</li>
            <li>
              Ask for something unbuildable — an opening crossing a corner, say — and the planner
              will tell you no.
            </li>
          </ul>
          <button className="info-close" onClick={() => ref.current?.close()}>
            Close
          </button>
        </div>
      </dialog>
    </>
  );
}
