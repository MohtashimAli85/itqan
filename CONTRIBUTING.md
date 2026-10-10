# Contributing to Itqan

Thanks for helping. Itqan is a local-first desktop app built with Tauri 2, a Rust core and a React + TypeScript frontend. The plan in [docs/PLAN.md](docs/PLAN.md) is the source of truth; read it before starting.

## Setup

Requirements: Node (see `.nvmrc`), pnpm (see `packageManager` in `package.json`), and the Rust toolchain pinned in `rust-toolchain.toml`. On macOS, install the Xcode command line tools. See the [Tauri prerequisites](https://tauri.app/start/prerequisites/) for other platforms.

```
pnpm install
pnpm tauri dev
```

pnpm is the only supported package manager.

## Checks

Run all of these before opening a pull request:

```
pnpm lint
pnpm format:check
pnpm typecheck
pnpm test
pnpm format:rust:check
pnpm lint:rust
pnpm test:rust
```

Git hooks (lefthook) run lint, format checks and commitlint automatically after `pnpm install`.

## Workflow

- One plan step is one branch and one pull request. Do not combine steps.
- Branch names: `feat/…`, `fix/…`, `chore/…`, `docs/…`, `refactor/…`.
- Commits follow [Conventional Commits](https://www.conventionalcommits.org/), small and focused.
- Never push to `main`.
- Tick the step in `docs/PLAN.md` in the same pull request. If a decision changes, update the plan and say so in the description.
- Open an issue or ask before adding a dependency or making a decision the plan does not cover.

## Code rules

The full list is in section 11 of the plan. The short version:

- No comments unless something is truly non-obvious.
- TypeScript is strict, with no `any`. Colours, spacing and fonts come from Tailwind tokens, never raw hex in components.
- Rust has no `unwrap()` or `expect()` outside tests; logic lives in `domain/` and commands stay thin.
- Store all times in UTC.
- Keys live only in the OS keychain. Never log secrets or captured text.
- The overlay must be idle when nothing moves.
- Generated bindings in `src/shared/bindings/` are never edited by hand.

## Privacy

Itqan has no telemetry and no backend. Changes must not add network calls other than the user's own AI provider. See [PRIVACY.md](PRIVACY.md).
