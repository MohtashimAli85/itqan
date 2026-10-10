# 0002: In-house prayer time calculation, ported from adhan

Status: accepted (step 1.5)

## Context
The plan asked to evaluate a maintained Rust crate (for example `salah`) against the `adhan` reference implementation before choosing.

## Evaluation
- `salah` 0.7.6 (a Rust port of adhan) was run for Karachi (University of Islamic Sciences, Karachi; Hanafi Asr) for every day of 2026 and compared with adhan-js 4.4.6.
- It matched adhan to the minute for the first 252 days, then **panicked** on 2026-09-10: when a time rounds up to 24:00 UTC it builds an invalid hour and calls `unwrap()` (`astronomy/solar.rs`). Karachi Fajr sits near 23:59 UTC for part of the year, so this would crash the app on real dates. Release builds use `panic = "abort"`, so it cannot be caught.

## Decision
Port adhan's algorithm (Jean Meeus, *Astronomical Algorithms*) into `src-tauri/src/prayer/` (since step 2.0.4: the `itqan-salah` crate, `src-tauri/crates/itqan-salah/`), with every conversion returning `Option` instead of panicking.

- Supported methods: Muslim World League, Egyptian, Karachi, Umm al-Qura, Dubai, North America (ISNA), Kuwait, Qatar.
- Supported Asr schools: Shafi and Hanafi.
- Supported high-latitude rules: middle of the night, seventh of the night, twilight angle.
- Not ported: Moonsighting Committee seasonal adjustments, Tehran's maghrib angle, and polar circle resolution. Polar days return no times.

## Verification
`src-tauri/crates/itqan-salah/tests/fixtures/prayer/*.csv` were generated with adhan-js 4.4.6 (`TZ=UTC`, default parameters plus the madhab shown in each header): Karachi for every day of 2026, and London, New York, Makkah, Cairo and Dubai every fifth day. The Rust port matches every time exactly to the minute.

## Licence
adhan is MIT licensed (Copyright (c) 2016 Batoul Apps). The notice is kept in `src-tauri/crates/itqan-salah/LICENSE-adhan`.
