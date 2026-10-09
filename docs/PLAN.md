# Itqan: Project Plan

Itqan (إتقان, "doing your work with excellence") is an open-source desktop companion that lives on your screen as a small animated orb. It acts like a personal manager and coach: it keeps your tasks and reminders, notices when you drift into doomscrolling, pushes you toward the skills and projects you care about, looks after your health so you can keep building for decades, and makes progress feel rewarding.

This document is the source of truth for the project. Every decision below was made deliberately. If something is not covered here, ask before deciding.

---

## 0. How to work on this repo

These rules apply to every Claude session in this repo.

1. **Go slow and do it properly.** Quality over speed. One small, reviewable step at a time.
2. **One MR per step.** Each step in section 13 is one branch and one MR. Do not combine steps.
3. **Never push to `main`.** Work on a branch, commit, and stop. The owner reviews every MR.
4. **Create the MR with `glab` only when the owner asks.**
5. **Ask before adding any dependency** not listed in section 9.
6. **Ask before any decision this plan does not cover.** Do not guess on architecture.
7. **Follow the code rules in section 11** and `CLAUDE.md`.
8. **Keep this plan current.** When a step is done, tick it in section 13 in the same MR. If a decision changes, update this file in the same MR and say so in the MR description.
9. **Run every check before saying a step is done:** lint, typecheck, tests, `cargo fmt --check`, `cargo clippy`. Report failures honestly.

---

## 1. Vision

### The problem
People lose hours to drifting (Shorts, Reels, Reddit), forget small but important things, and rarely make steady progress on the skills and side projects that would change their lives. Productivity apps are lists you have to remember to open. Nobody is in your corner during the day.

### The product
A small orb with eyes that floats over every app, follows your cursor like a buddy, and talks to you in short speech bubbles. It:

- keeps your tasks and reminders across work, personal life and health
- notices drift and points you at your real next step, not a generic "get back to work"
- acts like a manager or trainer: morning standup, evening check-in, weekly 1:1
- builds roadmaps for the skills and projects you want, and pushes you to **ship**, not just consume tutorials
- looks after your energy and health, framed as fuel for building for decades
- respects prayer times and rest, and never guilt-trips
- makes progress feel good: XP, levels, streaks, sound, confetti

Inspired by heyclicky (an AI buddy that lives next to your cursor), but built for focus, growth and accountability.

### Who it is for
First, the owner: a full-time developer whose main motivators are **learning, building and health**, not money. Then anyone else who installs it from the open-source repo and brings their own AI API key.

---

## 2. Principles

1. **Local-first.** All data lives on the user's machine in SQLite. No accounts, no backend, no telemetry.
2. **Open source, bring your own key.** Users plug in their own AI provider key. The app never ships a key.
3. **Privacy is a feature.** Capture is opt-in per app, encrypted at rest, auto-deleted, and pausable. Only the minimum text is ever sent to an AI provider, after redaction.
4. **One voice.** Many internal agents, but only the Coach talks to the user, within a strict interruption budget.
5. **Motivation is personal.** What gets rewarded and how nudges are worded comes from each user's motivation profile, never hard-coded.
6. **Outputs over inputs.** Shipping, publishing and applying are worth more than watching and reading.
7. **Supportive, never guilt.** A missed day leaves a gap, never wipes progress. Rest is allowed.
8. **Faith-aware, never faith-scored.** Prayer times pause nudges and focus. Worship is never tracked for XP, streaks or leaderboards.
9. **Light on the machine.** It runs all day, so idle CPU must be near zero.
10. **Islamic identity lives in behaviour, not decoration.** The visual design is modern. No classical Islamic patterns, lanterns or mosaics.

---

## 3. The motivation profile

Built during onboarding, visible and editable at any time, refined by weekly reviews.

| Field | Example (the owner) |
|---|---|
| Motivators, weighted | Learning 40%, Building 35%, Health 25%, Money 0% |
| Situation | Full-time job, about 8 free hours a week |
| Age | Optional. Used only for age-aware health emphasis |
| Goals | Learn Rust; ship Itqan v1; stay fit enough to code for 30 more years |
| Coach style | Mentor (supportive), Manager (direct, deadline-focused) or Trainer (no excuses) |
| Boundaries | Prayer times, family time, rest days, work hours |

What the profile drives:

- **Reward weights.** A builder earns most for shipping; a money-driven user sees an income tracker first.
- **Nudge wording.** "Ship the parser tonight" versus "This skill is worth more in job listings".
- **Suggestions.** Project ideas for builders, courses and practice for learners.
- **Health emphasis by age.** Posture and eye strain in the 20s; movement and strength added in the 30s; sleep and check-up reminders later. Habits and nudges only, never medical advice.

Other motivators users can weight: money, recognition, family, faith-related personal goals (never scored), status, freedom.

---

## 4. Features by phase

### Phase 0: Foundation (no product features)
Repo, tooling, rules, design system, app skeleton, CI. See section 13.

### Phase 1: The coach and the basics

**The orb (overlay)**
- Liquid orb with eyes that track the cursor, blink, and tilt as it moves
- States: idle, happy, alert, focus, resting, critical, listening
- Follow modes: follow cursor, sit in a corner, stay hidden
- Automatically docks while typing, hides during screen share and over full-screen video
- Progress ring around the orb: today's tasks, or the focus countdown during a session
- Speech bubbles with up to three actions
- Click the orb, the tray icon, or press `⌥ Space` to open the compact panel

**Compact panel**
- Greeting, level and XP bar, streak
- Tabs: Today, Health
- "Add a task or ask anything" input with simple local parsing ("buy dahi at 7pm") and a mic button (voice comes later)
- Focus session row while a session runs

**Tasks and reminders**
- Categories: Work, Personal, Health, Learning, Building (plus custom)
- Task kind: output, learning, deep work, habit. Drives reward weighting
- Top 3 for today, subtasks, notes, due date, priority
- One-off and recurring reminders (rrule), snooze, done from the bubble
- Critical reminders (medicine, must-dos) break through focus and keep the orb red until handled
- Reminders arrive through the orb; system notifications are the fallback when the orb is hidden or Do Not Disturb is on

**Goals and roadmaps (basic)**
- Goals with target dates
- The Planner agent breaks a goal into milestones and suggests daily tasks
- Skills with levels per skill

**Coach rhythm**
- Morning standup: "What are you shipping today?"
- Evening check-in: what got done, what got built
- Coach style from the profile

**Health (optional, rule-based)**
- Sitting-time stretch nudges, eye rest, water, medicine
- Age-aware emphasis from the profile
- Easy to turn off entirely

**Modes and schedule**
- Work hours, evening, rest, focus
- Overtime: if the user is still working after hours, ask before switching modes
- Prayer times calculated locally; nudges and focus pause around salah; Jumu'ah break on Fridays

**Focus sessions**
- 25 minutes by default; the orb turns green, docks, and the ring counts down

**Rewards**
- XP weighted by the motivation profile and task kind
- Levels, streak with freeze days, badges
- Sounds, confetti, an "all done today" celebration

**Product shell**
- Tray/menu bar icon with dropdown
- Main window: Today, Progress, Settings, Onboarding
- Launch at login, global hotkey, single instance

**Foundations for later phases**
- Event system that agents subscribe to
- Coach nudge budget and priority rules
- Outcome logging for every nudge

### Phase 2: Accountability
- Focus guardian: front app and window title tracking, user blocklist, time thresholds, then AI judgment of "is this on-task for my goal?"
- Drift nudges tied to the user's current goal and next task
- Learning agent: skill roadmaps, project ideas, small quizzes, "watched but did not build" checks
- Reviewer: weekly 1:1 using real data, adjusts the plan and profile weights
- Adaptive nudges: timing, tone and frequency learned from outcomes
- Browser URL detection

### Phase 3: Itqan Memory
- Opt-in per-app screen text capture (Slack first), stored locally and auto-deleted
- "Make todos from my Slack today", "anything about the Ravi invoice?"
- End-of-day summary of promises and requests found in captured text
- Learning versus doing analysis

### Later
- Voice input and spoken replies
- Windows and Linux polish if Phase 1 was Mac-first
- Rive character for the orb
- Plugin system for community agents

---

## 5. UX and visual design

### Mockups
- Clickable bot prototype: https://claude.ai/artifact/Lwza9UU63i1VW2ZCgR1ZBZ
- Design canvas with every surface: https://claude.ai/artifact/JY8aZdztRamcok2x87tb4h

### Surfaces
1. **Overlay:** the orb, its bubbles and the compact panel. Floats over every app.
2. **Tray/menu bar:** tasks left or focus timer; dropdown with next task, start focus, quick add, pause nudges, mode switch, orb placement (Follow / Corner / Hidden), open Itqan, settings, quit.
3. **Main window:** opened rarely.
   - **Today:** sidebar (Today, Upcoming, Work, Personal, Health, Progress, Settings, next prayer card), quick add, Top 3, sections, task detail panel with subtasks and notes
   - **Progress:** level and XP, streak, weekly focus hours chart, 5-week streak calendar, badges, skill levels
   - **Settings:** orb behaviour and personality, work hours, prayer times, nudges and distracting apps, AI provider and keys, rewards and sound, privacy
   - **Onboarding:** welcome, motivation profile, work hours and city, orb placement, AI key, notification permission

### Orb states
| State | Look | When |
|---|---|---|
| Idle | Orange orb, normal eyes | Following the cursor |
| Happy | Arc eyes (^ ^), hop, confetti | Task done, goal reached |
| Alert | Wide eyes, hop | Drift noticed |
| Focus | Green orb, flat eyes, ring counts down | Focus session |
| Resting | Blue orb, closed eyes, floating "z" | Rest mode |
| Critical | Red orb, wiggles | Critical reminder |
| Listening | Pulsing ring | Voice input |
| Evening | Pink-violet orb | Evening mode |

### Visual language
- **Modern, clean, light and dark themes.** No classical Islamic visuals.
- **Fonts (bundled, not loaded from the web):** Bricolage Grotesque for headings, Geist for UI text, Geist Mono for numbers and timers.
- **Colours (tokens, light theme):** ink `#101217`, secondary text `#4A505C`, captions `#5F6570`, background `#F3F4F6`, accent orange `#FF5B2E`, amber `#F5A000`, Work blue `#3D7BFF`, Personal green `#0FA97A`, Health pink `#E8407A`, critical red `#E5281E`. Dark theme values are defined alongside. Final tokens come from the shadcn preset plus these brand additions.
- Primary buttons are ink on light, never white text on orange (contrast).
- Nearly solid surfaces with soft shadows in the overlay (see section 6, gotchas).

### Tone of voice
Short, specific, warm. Every nudge names the real next step. Uses "MashaAllah" for celebrations. Never shames.

---

## 6. Architecture

### Processes and windows
- **Rust core (Tauri 2):** source of truth. Owns SQLite, the scheduler, reminders, prayer times, agents' event bus, AI calls, keychain access, cursor tracking, activity tracking, capture.
- **Overlay window:** one transparent, frameless, always-on-top window per display, hidden from the taskbar and Dock, visible on all Spaces and over full-screen apps.
- **Main window:** normal window for Today, Progress, Settings, Onboarding.
- **Tray:** native menu.

### Click-through overlay
- The overlay ignores mouse events by default, so clicks reach the apps below.
- Rust polls the global cursor position (Tauri 2 `cursor_position()`, about 60 Hz, only while the overlay is visible).
- The frontend reports the hit areas of the orb, bubble and panel. When the cursor enters one, Rust turns click-through off; when it leaves, back on.
- macOS: the panel must not steal focus from the user's app (non-activating panel via `tauri-nspanel`); window collection behaviour must allow showing over full-screen apps.
- Multiple displays: the orb moves to the display under the cursor.

### Data flow
- Frontend calls typed Rust commands (tauri-specta) through TanStack Query.
- Rust emits events when data changes; every window updates.
- Each window has its own small Zustand store for UI-only state.

### Scheduler
- Runs in Rust (tokio), never in JS timers, because hidden webviews throttle timers.
- Computes the next due reminders from rrule, fires through the orb or native notification.
- Stores all times in UTC plus the user's IANA timezone.

### Prayer times
- Calculated locally from city coordinates, calculation method and Asr school (default for the owner: Karachi, University of Islamic Sciences Karachi, Hanafi).
- Evaluated `salah` against `adhan`: it panics on real Karachi dates, so the algorithm is ported in-house and tested against adhan-js output (ADR 0002).

### Agents
- **Coach:** the only agent that speaks. Decides what, when and how.
- **Planner:** goals into milestones into daily tasks.
- **Focus guardian:** drift detection (Phase 2).
- **Learning agent:** roadmaps, project ideas, quizzes (Phase 2).
- **Health agent:** sitting time, sleep patterns, age-aware nudges.
- **Reviewer:** weekly 1:1 (Phase 2).
- **Memory agent:** captured text into todos and insights (Phase 3).

Rules:
- Agents react to events (app switched, task done, 50 minutes sitting, Friday evening, schedule ticks) and send **suggestions** to the Coach. They never talk to the user or to each other directly.
- **Nudge budget:** at most N interruptions per hour (configurable).
- **Priority:** critical reminder > drift > health > learning tip.
- **Silence:** during salah windows, focus sessions (except critical), rest mode, screen share.
- Agents can start as rules and later switch to an LLM without changing the interface.

### Learning from outcomes
- Every nudge is logged: agent, type, time, mode, wording style, outcome (accepted, snoozed, dismissed, ignored).
- Simple adaptation, no model training: shift timing, tone and frequency toward what the user accepts.
- The weekly review proposes profile changes; the user confirms.
- Prompts are versioned. Each agent has a small saved test set of scenarios with expected behaviour; run it before changing a prompt.

### Data model (conceptual)
| Table | Holds |
|---|---|
| profile | motivator weights, situation, optional age, coach style, boundaries |
| settings | theme, sound, orb placement, size, hotkeys, nudge budget, AI provider config (no keys) |
| schedules | work hours per day, modes |
| prayer_settings | city, coordinates, method, Asr school, pause windows, Jumu'ah break |
| categories | name, colour, icon, built-in or custom |
| goals | title, target date, motivator link, status |
| skills | name, level, XP |
| milestones | goal, title, week, status |
| tasks | title, notes, category, kind, priority, due (UTC), top-3 flag, status, completed at, parent task, goal or skill link |
| reminders | task or habit, fire time (UTC), rrule, critical flag, snoozed until |
| habits | type, target, schedule rule, enabled |
| habit_logs | habit, logged at, amount |
| focus_sessions | start, end, planned length, task link, completed |
| xp_events | source, amount, at |
| streaks | current, best, freezes left, last active day |
| nudges | agent, type, text, fired at, outcome, outcome at |
| reviews | week, summary, proposed changes, accepted |
| activity_events (Phase 2) | app, window title, URL, start, end |
| memory_chunks (Phase 3) | app, window, channel, text, author flag, captured at, expires at |
| memory_vectors (Phase 3) | chunk, embedding (sqlite-vec) |

Migrations are versioned and run at startup.

---

## 7. AI layer

### Providers
- Any **OpenAI-compatible** endpoint: base URL, key, and model per role.
- Presets:
  - **Groq** (default for the owner): `openai/gpt-oss-20b` as the fast model, `openai/gpt-oss-120b` as the smart model. Confirm exact model IDs against Groq's docs when implementing.
  - **Local:** Ollama or llama.cpp running `gpt-oss-20b` (needs about 16 GB RAM). Nothing leaves the machine.
  - Others users may add: OpenAI, OpenRouter.
- Keys live in the OS keychain (macOS Keychain, Windows Credential Manager) via the `keyring` crate. Never in config files or SQLite.
- "Test key" button makes one cheap call.

### Routing
| Job | Model |
|---|---|
| Drift judgment, quick-add parsing, bubble wording, commitment scanning | Fast (20B), low reasoning effort |
| Planning, roadmaps, weekly review, todo extraction from memory, answering questions | Smart (120B), higher reasoning effort |

### Rules
- Use structured output or tool calling for anything parsed (todos, plans). Validate with schemas in Rust.
- Captured text is **untrusted**. Wrap it clearly as data in prompts. The AI never acts on its own: anything it proposes (todos, plan changes) needs user confirmation.
- Redact before sending: phone numbers, CNIC, card numbers, emails, OTPs, API keys. Restore placeholders locally.
- Send only retrieved chunks, never a whole day's capture.
- Handle rate limits, timeouts and offline: queue, retry with backoff, fall back to the local model if configured. The orb never breaks because AI failed.
- Embeddings are always local.

---

## 8. Itqan Memory (Phase 3 design)

### Capture
- Triggers: app or window switch, content change in the focused window (checked every few seconds), idle on a screen.
- **Accessibility tree first:** macOS AXUIElement (Accessibility permission), Windows UI Automation. Exact text with structure.
- Electron apps like Slack hide their accessibility tree by default; on macOS set `AXManualAccessibility` on the app element to expose it.
- **OCR fallback:** macOS ScreenCaptureKit + Vision; Windows Graphics Capture + Windows.Media.Ocr.
- **Never a keylogger.** Secure fields are skipped by the OS; never read them.
- Read only the focused window, limit tree depth, pause on battery or idle.

### Processing
- Dedupe by hash, keep only new lines per window.
- Tag with app, window or channel, time, own-message flag.
- Redact secrets before storing.

### Storage
- SQLite with **SQLCipher** encryption.
- **FTS5** for keyword search, **sqlite-vec** for vectors.
- Retention configurable; default "today only", wiped at end of day.

### Retrieval
- Start simple: filter by app and time, FTS5, send to the smart model.
- Add hybrid search (BM25 + vector similarity) once retention is longer than a day.
- Local embeddings: `bge-small` or `nomic-embed-text` via ONNX Runtime (`ort` or `fastembed` crate).

### Safety
- Every app is off until the user opts in.
- Visible "Itqan is reading" indicator, one-click pause, "delete today" button.
- Never capture by default: password managers, banking apps, private/incognito windows.
- Learn from Microsoft Recall's launch criticism: encryption and short retention from day one.
- Capturing colleagues' messages may break workplace policy. Say so clearly in onboarding for Memory.

---

## 9. Tech stack

### Desktop and Rust
| Need | Choice |
|---|---|
| Shell | Tauri 2 |
| Async runtime | tokio |
| Database | SQLite via `rusqlite` (bundled) + versioned migrations; SQLCipher and sqlite-vec in Phase 3 |
| Bindings to TS | `specta` + `tauri-specta` |
| Errors | `thiserror` in library code, no `unwrap()` outside tests |
| Logging | `tracing` + `tauri-plugin-log` |
| Dates | `chrono`, `chrono-tz`, `rrule` crate for recurrence in the scheduler |
| HTTP for AI | `reqwest` with streaming |
| Keychain | `keyring` |
| Tauri plugins | notification, autostart, global-shortcut, single-instance, updater, log; tray from core |
| macOS | `tauri-nspanel` for the non-activating panel; `window-vibrancy` for the main window sidebar |

Choose between `rusqlite` and `sqlx` in step 0.6 and record why. Recommendation: `rusqlite`, because loading the sqlite-vec extension and SQLCipher is simpler.

### Frontend
| Need | Choice |
|---|---|
| Framework | React + TypeScript (strict) + Vite |
| Package manager | **pnpm** only |
| Styling | Tailwind CSS v4, tokens as CSS variables, light and dark |
| Components | shadcn/ui, installed with `pnpm dlx shadcn@latest apply --preset b1FSRLMOG` |
| Icons | Lucide |
| Fonts | Bricolage Grotesque, Geist, Geist Mono, bundled as files |
| Animation | Motion |
| Orb | CSS/SVG now, Rive later |
| Confetti | canvas-confetti |
| Sound | Howler.js |
| Routing (main window) | TanStack Router |
| Server state | TanStack Query over Tauri commands |
| UI state | Zustand, one store per window |
| Forms | React Hook Form + Zod |
| Command palette | cmdk (shadcn Command) |
| Drag and drop | dnd-kit |
| Charts | Recharts via shadcn charts |
| Toasts | Sonner (main window only) |
| Dates | Day.js with plugins (utc, timezone, relativeTime, isoWeek, isBetween, isSameOrBefore, duration, customParseFormat) loaded once in `src/shared/lib/dayjs.ts`; rrule.js for editing and display only |

Not used: date-fns (Day.js chosen instead), GSAP, Magic UI, Aceternity, Three.js.

### Quality tooling
| Need | Choice |
|---|---|
| Lint | ESLint flat config with typescript-eslint, react-hooks, jsx-a11y |
| Format | Prettier + prettier-plugin-tailwindcss |
| Rust | rustfmt, clippy with warnings as errors |
| Git hooks | lefthook |
| Commits | Conventional Commits, checked with commitlint |
| Unit tests | Vitest + Testing Library; `cargo test` |
| E2E (later) | WebdriverIO + tauri-driver (works on Windows and Linux; macOS WebDriver is not supported) |
| CI | GitLab CI: install, lint, typecheck, test, fmt, clippy, build check |
| Node | Pinned in `.nvmrc` and `packageManager` field |
| Rust | Pinned in `rust-toolchain.toml` |

---

## 10. Repo structure

```
itqan/
  CLAUDE.md
  README.md
  CONTRIBUTING.md
  PRIVACY.md
  LICENSE
  docs/
    PLAN.md                 this file
    decisions/              one short ADR per decision that changes the plan
  .claude/
    skills/                 project skills (see 12)
  .gitlab/
    merge_request_templates/Default.md
  .gitlab-ci.yml
  src/
    overlay/                orb, bubbles, compact panel (own Vite entry, kept light)
    main/                   Today, Progress, Settings, Onboarding (own Vite entry)
    shared/
      components/ui/        shadcn components
      components/           shared components
      lib/                  dayjs, utils
      hooks/
      bindings/             generated tauri-specta types
      styles/               tokens, fonts
  src-tauri/
    src/
      main.rs
      lib.rs
      commands/             thin Tauri command layer
      db/                   connection, migrations, repositories
      domain/               tasks, reminders, goals, profile, rewards
      scheduler/
      prayer/
      agents/               coach, planner, health, ...
      ai/                   providers, routing, redaction
      overlay/              cursor tracking, click-through, panels
      platform/             macos/, windows/ behind small traits
    migrations/
    capabilities/
    tauri.conf.json
  index.html / overlay.html entry points
```

Platform-specific Rust lives behind small traits in `platform/` so Windows support can be added without touching domain code.

---

## 11. Code rules (also summarised in CLAUDE.md)

### General
- No comments unless something is truly non-obvious. Names should explain the code.
- Small files, small functions, one responsibility each.
- No dead code, no commented-out code, no TODOs without an issue link.
- Match the style of surrounding code.

### TypeScript and React
- `strict: true`, `noUncheckedIndexedAccess: true`. No `any`, no non-null `!` without a reason.
- Function components only, named exports.
- Components in PascalCase files; hooks start with `use`.
- No `useEffect` for derived state or data fetching; use TanStack Query and derived values.
- Colours, spacing, radii and fonts only through Tailwind tokens. No raw hex in components.
- Accessible by default: real buttons and inputs, labels, focus states, `aria-label` on icon-only buttons, text contrast 4.5:1.
- Respect `prefers-reduced-motion`.
- All Day.js imports go through `src/shared/lib/dayjs.ts`.
- Generated bindings in `src/shared/bindings/` are never edited by hand.

### Rust
- `cargo fmt` and `cargo clippy -- -D warnings` clean.
- No `unwrap()` or `expect()` outside tests and startup invariants.
- Errors with `thiserror`, converted to a serialisable error type at the command boundary.
- Commands are thin; logic lives in `domain/`.
- All times stored in UTC.
- Never log secrets, keys or captured text.

### Overlay performance
- Animation loops only while something moves; stop when settled.
- Pause CSS animations when the orb is docked, hidden or the window is not visible.
- Animate only `transform` and `opacity`.
- No heavy libraries in the overlay bundle.
- Use nearly solid surfaces: CSS `backdrop-filter` cannot blur other apps behind a transparent window.

### Security and privacy
- Keys only in the OS keychain.
- Captured text is untrusted input to prompts.
- AI suggestions always need user confirmation.
- No telemetry.

### Git
- Branches: `feat/…`, `fix/…`, `chore/…`, `docs/…`, `refactor/…`.
- Conventional Commits, small and focused.
- One MR per plan step, with the template filled in.
- Never push to `main`.

---

## 12. CLAUDE.md, skills and MR template

### CLAUDE.md (draft, keep it short)
```
# Itqan

Desktop companion app: Tauri 2 + Rust core + React/TS (Vite), pnpm only.
Source of truth: docs/PLAN.md. Read it before starting any task.

## Workflow
- One plan step = one branch = one MR. Never push to main.
- Create MRs with glab only when asked.
- Ask before adding dependencies or making decisions not in the plan.
- Tick the step in docs/PLAN.md when done.

## Commands
- pnpm dev / pnpm build
- pnpm lint / pnpm typecheck / pnpm test
- cargo fmt --check / cargo clippy -- -D warnings / cargo test (in src-tauri)

## Rules
- No comments unless truly needed.
- TS strict, no any. Tailwind tokens only, no raw hex.
- Rust: no unwrap outside tests; logic in domain/, commands stay thin.
- Times in UTC. Keys only in the OS keychain. Never log secrets or captured text.
- Overlay must be idle when nothing moves.
- All Day.js imports via src/shared/lib/dayjs.ts.

See docs/PLAN.md sections 11 (code rules) and 6 (architecture).
```

### Skills
- Use `find-skills` or browse skills.sh, search for: Tauri, Rust, React, TypeScript, Tailwind, shadcn, accessibility, testing, git commits.
- **Propose a shortlist to the owner with what each skill does and its source repo before installing anything.**
- Install with pnpm: `pnpm dlx skills add <owner/repo>`, into the project so every session gets them.
- Prefer well-maintained skills from known publishers. Read each skill's instructions before adding it.
- Project-specific skills to write ourselves later: "add a Tauri command end to end", "add a migration", "add an agent".

### CodeGraph
After the app skeleton exists (step 0.5), run `codegraph init -i` so later sessions can explore the code efficiently.

### MR template (`.gitlab/merge_request_templates/Default.md`)
- What and why (link to the plan step)
- How to test it locally
- Screenshots or a short recording for UI changes
- Checklist: lint, typecheck, tests, fmt, clippy pass; plan updated; no new dependencies without approval; no secrets

---

## 13. Phase 0 steps (each is one MR)

- [x] **0.1 Project docs.** Add `docs/PLAN.md` (this file), `CLAUDE.md`, `README.md` (short vision and status), `.gitignore`, `.editorconfig`, `LICENSE` (after the owner chooses, see section 14).
- [x] **0.2 Scaffold.** Tauri 2 + React + TS + Vite with pnpm (`pnpm create tauri-app`). If the CLI refuses a non-empty folder, scaffold in a temp folder and move the files in. App name Itqan, identifier agreed with the owner. App runs with `pnpm tauri dev`.
- [x] **0.3 Tooling.** TS strict settings, ESLint, Prettier, rustfmt, clippy config, lefthook, commitlint, `.nvmrc`, `packageManager`, `rust-toolchain.toml`, package scripts.
- [x] **0.4 Design system.** Tailwind v4, then shadcn with `pnpm dlx shadcn@latest apply --preset b1FSRLMOG` exactly as given (if the CLI reports an unknown command, stop and ask). Add brand tokens from section 5, bundle the three fonts, light and dark themes, a small token preview page in the main window.
- [x] **0.5 Window structure.** Separate Vite entries for overlay and main; Tauri window config for both; capabilities with least privilege; tray icon with Quit; single-instance plugin. Then run `codegraph init -i`.
- [x] **0.6 Rust core skeleton.** Module layout from section 10, error type, tracing, SQLite connection with first migration (settings table), tauri-specta bindings generated, one example command used by the main window through TanStack Query. Record the rusqlite vs sqlx choice.
- [x] **0.7 Tests and CI.** Vitest + Testing Library with one example test, `cargo test` example, `.gitlab-ci.yml` running every check.
- [x] **0.8 Contributor docs and skills.** `CONTRIBUTING.md`, `PRIVACY.md`, MR template, skills shortlist approved and installed.

### Phase 1 order (after Phase 0)
- [x] **1.0 Overlay spike (go/no-go).** Go on macOS (owner tested follow, click-through and full screen; second display check pending). Transparent click-through overlay, cursor following, hit areas, over full-screen apps, two displays. macOS first, Windows check if cross-platform from day one.
- [x] 1.1 Orb character, states and bubbles (docking while typing and hiding during screen share or full-screen video moved to 1.13)
- [x] 1.2 Tasks data model and Rust commands
- [x] 1.3 Compact panel and quick add (level, XP and streak row arrive with 1.10, Health tab with 1.9, focus row with 1.5, mic button with voice)
- [x] 1.4 Scheduler, reminders, notification fallback
- [x] 1.5 Modes, work hours, prayer times (includes focus sessions; prayer times ported from adhan, see docs/decisions/0002-prayer-times.md)
- [ ] 1.6 Motivation profile and onboarding
- [ ] 1.7 Goals, skills and the Planner
- [ ] 1.8 Coach, nudge budget, outcome logging, standup and check-in
- [ ] 1.9 Health agent (rules)
- [ ] 1.10 Rewards: XP, levels, streaks, sound, confetti
- [ ] 1.11 AI provider settings, keychain, routing
- [ ] 1.12 Tray, main window screens, settings, autostart, hotkey
- [ ] 1.13 Performance and polish

---

## 14. Open decisions (ask the owner)

1. ~~**Mac-first or cross-platform from day one?**~~ **Decided:** cross-platform by design, macOS first. Build and use on macOS first, keep platform code behind traits in `platform/`, check Windows from the overlay spike (1.0) onward.
2. ~~**Licence.**~~ **Decided:** MIT. The repo is public.
3. **Apple Developer account** for signing and notarisation (affects permissions and Gatekeeper warnings for other users).
4. ~~**App identifier**~~ **Decided:** `dev.itqan.desktop`.
5. **Repo host for the public release:** GitLab (current) or a GitHub mirror for discoverability.
6. **Default coach style** for new users.

---

## 15. Glossary

- **Orb:** the floating character. Itqan's face.
- **Bubble:** a speech bubble next to the orb.
- **Compact panel:** the small window that opens from the orb.
- **Nudge:** any interruption the Coach decides to show.
- **Nudge budget:** the cap on interruptions per hour.
- **Drift:** time spent on distracting apps during focus or work hours.
- **Output task:** shipping, publishing, applying. Rewarded most for builders.
- **Motivation profile:** each user's weighted motivators and context.
- **Memory:** opt-in local capture of screen text (Phase 3).
- **BYOK:** bring your own key.
