# BrickPlan

Define a small building — four walls on a rectangular footprint, with a
door or window in the front. A Rust planner — compiled to WebAssembly,
running in your browser — turns it into a brick-by-brick placement plan
as plain JSON. A React simulator builds the plan in 3D, one step at a time.

Live at [jp-alchemy.github.io/brickplan](https://jp-alchemy.github.io/brickplan/).

Built after reading Monumental's ["Plans as Data"](https://buildmonumental.substack.com/p/plans-as-data)
post, to feel the shared Rust/WASM stack idea firsthand rather than just nod
at it. It is a deliberately small homage, not a clone: one bond pattern, one
opening, no physics — but the architecture is the real thing in miniature.

## Architecture

```
  WallSpec (footprint, height, brick, joint, optional opening)
      |
      v
  planner/ ................. Rust crate, pure functions
      |   layout.rs          stretcher bond on four walls, corner
      |   sequence.rs        returns, cut bricks, openings; then
      |   lib.rs             bottom-up ordering + support validation;
      |                      lib.rs is the only WASM-facing code
      |   compiled natively  -> cargo test, no WASM tooling needed
      |   compiled to WASM   -> runs live in the browser
      v
  Plan (JSON: placements, steps, stats)  <-- inspect it in the UI
      |
      v
  web/ ..................... Vite + React + TypeScript
          WallScene          react-three-fiber simulator: one instanced
                             mesh, playback just moves the draw count
          SpecControls       edit the spec; every change re-plans (<10ms)
          PlaybackBar        play / scrub / 1-4-16x
          PlanPanel          the raw plan, visible and downloadable
```

## Corners

Corners are where bonds earn their keep. The planner alternates which
pair of walls "runs through" to the outer corner each course: front and
back on even courses, the sides on odd ones. The receding wall starts one
corner return — brick width plus a joint, 110mm — in from the corner.
Waalformaat bricks are proportioned so that this return is exactly half
the bond module (210 = 2 x 100 + 10), so the alternation *is* the
stretcher stagger: every vertical joint, including those at the corners,
lands mid-brick on the courses above and below, with no corner cuts. The
planner rejects brick dimensions that break this proportion rather than
quietly emitting corners that don't line up.

The planner never draws and the simulator never plans. Everything that
crosses between them is serializable data, so the same plan that animates
the SVG could drive anything else that speaks JSON. Errors are data too:
the boundary returns `{ ok: Plan } | { err: PlanError }`, and an invalid
spec renders as a normal UI state, not an exception.

A step is deliberately small:

```json
{ "seq": 12, "action": { "type": "PickBrick", "kind": { "type": "Half" } } }
{ "seq": 13, "action": { "type": "PlaceBrick", "placement_id": 6 } }
```

## Honest simplifications

A real masonry planner would treat each of these differently:

- **One bond pattern.** Stretcher bond only. Flemish, English, or header
  bonds would turn the per-course layout into a strategy the spec selects.
- **One opening, front wall only.** And it must stay clear of the corner
  returns; the planner rejects anything else. Openings per wall would be
  a spec change, not an engine change.
- **Naive lintels.** The course above an opening spans it uninterrupted,
  and the support check knowingly accepts bricks resting on that gap. A
  real wall gets a lintel element with its own placement and lead time.
- **Horizontal cuts only.** An opening whose sill or top lands mid-course
  effectively snaps to course boundaries. Real walls cut bricks lengthwise.
- **No execution constraints.** Reachability, mortar cure time, collision
  avoidance, error recovery — none modeled. This planner proves ordering
  and geometry; a robot planner also proves feasibility.
- **Slivers are absorbed.** End-of-course remainders under 40mm widen the
  neighboring joint instead of becoming unmanageable cut bricks.

## Running it

Prerequisites: Rust (with the `wasm32-unknown-unknown` target),
[wasm-pack](https://rustwasm.github.io/wasm-pack/), Node 22+, and
[just](https://github.com/casey/just).

```sh
just test     # cargo test — the planner suite runs natively
just lint     # clippy -D warnings, rustfmt, tsc, eslint
just dev      # build WASM, start the Vite dev server
just build    # production build into web/dist
```

From a fresh clone, `just dev` is enough: it builds the WASM package that
`web/` imports and serves the app on `localhost:5173`.

## Tests

The interesting properties are all covered natively — no WASM tooling in
the test loop:

- course counts as a function of wall, brick, and joint dimensions
- corners alternate their through-wall and land on the half-module stagger
- pairwise plan-view non-overlap per course, across all four walls
- cut bricks land flush against opening edges; no cut below the minimum
- the support rule rejects floating bricks and accepts corner bricks
  resting on the perpendicular wall below
- spec validation errors (zero dims, out-of-bounds or corner-crossing
  openings, un-bondable brick proportions) fire correctly
- a committed JSON fixture pins the wire format — the same shape the WASM
  boundary emits (regenerate intentionally with `UPDATE_FIXTURES=1 cargo test`)

## Deploying

`just deploy` runs the checks, builds, and pushes `main`; the GitHub
Actions workflow (`.github/workflows/ci.yml`) re-runs tests and lints,
rebuilds everything, and publishes `web/dist` to GitHub Pages. The build
is fully static — any static host works.

## Future work

- Openings on any wall, and more than one per wall.
- Alternative bond patterns as planner strategies.
- Robot-flavored constraints: reachability windows, cure-time-aware
  sequencing, replanning after a failed placement.
