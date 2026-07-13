# BrickPlan build tasks. `just --list` shows this menu.

default: test

# Run the Rust test suite (no WASM tooling needed)
test:
    cargo test --manifest-path planner/Cargo.toml

# Lint and format checks for both halves
lint:
    cargo clippy --manifest-path planner/Cargo.toml --all-targets -- -D warnings
    cargo fmt --manifest-path planner/Cargo.toml --check
    cd web && npx tsc -b && npm run lint

# Build the WASM package the web app imports
wasm:
    wasm-pack build planner --target web --out-dir pkg

# Start the dev server (rebuild WASM first so it is never stale)
dev: wasm
    cd web && npm run dev

# Production build into web/dist
build: wasm
    cd web && npm run build

# Deploy: push main; the ci workflow tests, builds, and publishes to
# GitHub Pages (see .github/workflows/ci.yml)
deploy: test lint build
    git push origin main
