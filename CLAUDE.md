# Itqan

Desktop companion app: Tauri 2 + Rust core + React/TS (Vite), pnpm only.
Source of truth: docs/PLAN.md. Read it before starting any task.

## Workflow
- One plan step = one branch = one PR (GitHub, `gh`). Never push to main.
- Never merge: open the PR, run an independent review (`/code-review`), fix or explain every finding, then stop. The owner merges unless they grant a one-off exception for a named run.
- Ask before adding dependencies or making decisions not in the plan.
- Tick the step in docs/PLAN.md when done.

## Commands
- pnpm dev / pnpm build
- pnpm lint / pnpm typecheck / pnpm test
- cargo fmt --check / cargo clippy -- -D warnings / cargo test (in src-tauri)
- cargo test also regenerates src/shared/bindings/bindings.ts; commit it with Rust command changes

## Rules
- No comments unless truly needed.
- TS strict, no any. Tailwind tokens only, no raw hex.
- Rust: no unwrap outside tests; logic in domain/, commands stay thin.
- Structs crossing IPC use `#[serde(rename_all = "camelCase")]`.
- Times in UTC. Keys only in the OS keychain. Never log secrets or captured text.
- Overlay must be idle when nothing moves.
- All Day.js imports via src/shared/lib/dayjs.ts.

See docs/PLAN.md sections 11 (code rules) and 6 (architecture).
