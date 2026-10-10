# 0005: Shifts, anchors and the day ledger

Status: proposed (stub, written in step 2.2.1)

## Context
Phase 2 splits the day into shifts, each with its own agent, tone and orb colour (PLAN.md section 17, D.5). The owner confirmed the five defaults and that work hours come from the existing settings.

## To decide in step 2.2.1
- How anchors (salah events, work schedule, first activity) resolve into shift boundaries each day.
- How shifts relate to the existing modes (work, evening, rest, focus).
- `ShiftStarted` and `ShiftEnded` events and how the Coach uses them.
- The day ledger: who writes notes, how commitments are followed up.
- Orb colour and tone per shift.
- Fallback anchors (clock times, work hours) when the salah module is disabled or has no location.
