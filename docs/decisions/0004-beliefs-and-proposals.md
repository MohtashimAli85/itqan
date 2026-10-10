# 0004: Beliefs and proposals

Status: accepted (step 2.1.1)

## Context
Phase 1's motivation profile is a set of sliders (motivator weights) and a coach style, stored in the `profile` row. Phase 2 replaces fixed settings with **learned beliefs**, and every learned change becomes a **proposal** that applies only after the user accepts it (PLAN.md section 17, D.1 to D.4).

The code that reads the profile today:
- XP multipliers use motivator weights (`progress::rewards::task_xp`).
- The standup and check-in texts use the coach style and the top motivator (`rhythm`).
- The Coach's tone uses the coach style; quiet family time uses `family_start_minute` and `family_end_minute`.
- The planner reads free hours per week; health reads the age band.

The move must keep all of that working and lose nobody's setup.

## Decisions

### 1. Beliefs
Beliefs are core data (`beliefs` table, core migration).

| Column | Meaning |
|---|---|
| `id` | |
| `statement` | in the user's words where possible: "Getting good at Rust matters a lot" |
| `kind` | `motivator`, `preference`, `pattern`, `constraint`, `goal` |
| `subject` | a stable machine key code can read, e.g. `motivator.building`, `coach.style`, `focus.minutes`; free-form beliefs from the chat use `note` |
| `value` | JSON the subject's reader understands, e.g. `{"weight": 40}`, `{"style": "mentor"}`, `{"minutes": 30}`; `null` for notes |
| `strength` | `low`, `medium`, `high` |
| `confidence` | 0 to 1 |
| `source` | `said` (onboarding, settings or chat), `observed` (behaviour), `confirmed` (the user accepted a proposal) |
| `evidence` | JSON list of short references (`"12 of 14 learning tasks done"`), at most 10, newest first |
| `status` | `active`, `proposed`, `rejected`, `archived` |
| `created_at`, `confirmed_at`, `updated_at` | UTC |

- At most one **active** belief per subject, except `note`. Saying or confirming a new value for a subject archives the old one.
- Code reads beliefs only through typed accessors in core (`beliefs::motivator_weights`, `beliefs::coach_style`, `beliefs::focus_minutes`, …), never raw JSON.
- `Profile` stays as the read model: `profile::get` fills `motivators` and `coach_style` from active beliefs, so the rewards, rhythm and Coach code does not change. `profile::save` writes those two parts as beliefs (`source = said`) and the rest (name, situation, free hours, age, family time) to the `profile` row as today. Those stay plain settings: they are facts the user states, not things Itqan learns.

### 2. Migrating the sliders
A core migration copies the current profile into beliefs, once:
- each motivator weight > 0 becomes `kind = motivator`, `subject = motivator.<name>`, `value = {"weight": w}`, `strength` high for w ≥ 30, medium for w ≥ 15, otherwise low, `source = said`, `confidence = 0.8`, `status = active`, `statement` "<Motivator> matters to me";
- the coach style becomes `kind = preference`, `subject = coach.style`, `value = {"style": "<style>"}`, `source = said`, `confidence = 0.8`.

The `profile.motivators` and `profile.coach_style` columns stay (no destructive change) but are no longer read. The migration test checks that `profile::get` returns the same motivators and coach style before and after.

### 3. Confidence
- Said by the user: 0.8. Confirmed through a proposal: at least 0.9.
- Observed: starts at 0.3. Each new supporting observation adds 0.1, capped at 0.85 (only the user can push it past that, by confirming).
- Contradicting evidence subtracts 0.15. An observed belief below 0.2 is archived.
- A rejected proposal subtracts 0.2 from its belief and suppresses the same proposal (section 4).

### 4. Proposals
Core data (`proposals` table, core migration).

| Column | Meaning |
|---|---|
| `id` | |
| `kind` | `<owner>:<name>`, e.g. `core:belief`, `core:focus_minutes`, `tasks:area` (2.3) |
| `key` | dedupe key, e.g. `core:focus_minutes:30`; at most one pending proposal per key |
| `title` | what would change: "Make 30 minutes the default focus length?" |
| `reason` | why, with the evidence in plain words |
| `effect` | what accepting does, in one sentence |
| `payload` | JSON for the handler |
| `belief_id` | the belief this proposal would confirm, if any |
| `status` | `pending`, `accepted`, `rejected`, `expired` |
| `created_at`, `decided_at`, `expires_at` | UTC; pending proposals expire after 14 days |

- **Who applies it:** each owner registers a `ProposalHandler` per kind with core's `ProposalRegistry` (the same pattern as the `ActionRouter`). Core stores and shows proposals; the handler applies the change. Core never writes a module's tables (ADR 0003, section 5).
- **Accept:** the handler's `apply` runs first. Only if it succeeds is the proposal marked `accepted`, and its belief becomes `active`, `source = confirmed`, confidence at least 0.9, `confirmed_at = now`.
- **Reject:** marked `rejected`; the belief loses 0.2 confidence; the same `key` is not proposed again for 30 days.
- **Owner disabled:** a pending proposal whose kind has no registered handler is hidden, not deleted, and returns when the module is turned back on.
- **Never auto-applied** (owner decision 5, section 14: "auto-approve small tweaks: never, for now").

### 5. Observations (the D.3 "learning signals")
The word "signal" already means a module's suggestion to the Coach (ADR 0003). Behaviour that feeds learning is called an **observation**: core table `observations(id, kind, subject, value, at)`. Examples: `task.done` with the area and kind, `task.postponed`, `focus.ended` with planned minutes and whether it completed, `nudge.outcome` with the nudge kind and outcome. A core subscriber writes them from bus events and nudge outcomes; nothing is sent anywhere. Rows older than 90 days are deleted on startup.

Rule-based learners (2.1.5) read observations, update observed beliefs (section 3) and open proposals when a belief crosses 0.6 confidence and differs from what is active.

### 6. AI and the no-key fallback (2.1.4)
- **With a key:** the five onboarding questions (PLAN D.2) are answered in a short chat. The smart model turns the answers into a JSON list of `{statement, kind, subject, value, strength}`. It is parsed the way the planner parses its plan: extract the JSON, validate every field against the known kinds and subjects, drop anything invalid, and treat unknown subjects as `note`. The user sees the list and can edit or remove each item before saving. Saved items are `source = said`, `confidence = 0.8`. Answers are redacted before sending, as all AI input is.
- **Without a key:** the same screen offers simple choices: pick what matters from the motivator list (each pick becomes a medium-strength motivator belief) and choose a coach style. The chat is offered later from Settings once a key exists.
- AI never creates an active belief on its own: chat output is shown for review first, and anything learned later goes through a proposal.

### 7. Where proposals appear
- A **Proposals** page in the main window (core route `/proposals`) with accept and reject, plus a count in the sidebar.
- A **panel row** when at least one proposal is pending: "1 suggestion to review" opens the page.
- The weekly review (2.8) and the Evening and Night shifts (2.2) surface them too.

## Consequences
- Old profile columns linger, unused. A later migration can drop them once beliefs have shipped for a while.
- Every learned behaviour needs a handler and a typed accessor, which is a little ceremony, but it keeps "learned" from ever meaning "changed silently".
- Observations are a new local log of behaviour. They stay on the device like everything else, and PRIVACY.md gets a line about them in 2.1.5.

## Step plan
| Step | Delivers |
|---|---|
| 2.1.1 | this ADR |
| 2.1.2 | `beliefs` table, slider migration with test, typed accessors, `profile::get` and `save` on beliefs |
| 2.1.3 | `proposals` table, `ProposalRegistry`, accept and reject commands, Proposals page and panel row, `core:belief` handler |
| 2.1.4 | conversational onboarding with review list; no-key choices |
| 2.1.5 | `observations` table and subscriber, confidence updates, first rule-based learners (focus length, motivators from finished work, nudges that are always dismissed) |
