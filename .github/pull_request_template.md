## What and why

<!-- What this changes, and why. Link the issue if there is one. -->

## Not tried on a Mac

<!-- Every change must keep the macOS port compiling (CI checks it). Say what has not been run
on a real Mac, or "nothing Mac-specific". -->

## Checklist

From `AGENTS.md`; strike out what does not apply.

- [ ] A new command has its three edits: the handler in `lib.rs`, `src-tauri/src/command_names.rs`, and the capability of the window that calls it (never widening the overlay's).
- [ ] Changed types are changed on both sides: `src-tauri/src/types.rs` and `src/lib/types.ts`.
- [ ] Every new visible string, error sentences included, is in `en.ts`, `fr.ts` and `de.ts`.
- [ ] If what the app sends, to whom, or what it keeps changed: `docs/privacy.md` is updated, with a new effective date.
- [ ] If what the app advertises changed: `docs/store-listing-next.md` is updated.
- [ ] No test or CI step calls a billable service.
- [ ] `npm run verify` and `npm run verify:rust` pass (the commands are listed in the README).
