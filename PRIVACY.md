# Privacy

Itqan is local-first. This page describes what the app does with your data today and what it commits to as it grows.

## What stays on your machine

- Everything you enter (tasks, goals, settings, progress) is stored in a local SQLite database in your operating system's app data folder.
- There are no accounts, no backend servers and no telemetry. Itqan does not send usage data anywhere.

## AI providers

- Itqan uses your own AI provider key (bring your own key). The app never ships a key.
- Keys are stored in the OS keychain (macOS Keychain, Windows Credential Manager), never in config files or the database.
- Only the minimum text needed for a request is sent to the provider you configure, after redaction of sensitive values such as phone numbers, card numbers, emails, one-time codes and API keys.
- You can run a local model instead, in which case nothing leaves your machine.
- Anything the AI proposes needs your confirmation before it changes your data.

## Activity tracking (planned)

Tracking the front app and window title to notice drift is a planned feature. When it ships it will be opt-in, based on a list you control, and the data will stay local.

## Itqan Memory (planned)

Capturing on-screen text is a planned, optional feature. When it ships:

- Every app is off until you opt in, per app.
- Captured text is encrypted at rest, deleted automatically (by default at the end of the day), and can be paused or deleted with one click.
- A visible indicator shows when Itqan is reading.
- Password managers, banking apps and private browsing windows are never captured by default.
- Capturing messages from colleagues may conflict with your workplace policy. Check before enabling it.

## Contact

Open an issue on the repository for privacy questions or concerns.
