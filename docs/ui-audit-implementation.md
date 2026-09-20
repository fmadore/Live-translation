# Operator UI audit implementation — 20 September 2026

Implements the supplied **Live Translation UI Audit** following the 1.4.0 feature release.
The existing dark surfaces, mint selection state, numbered setup and live-session
identity are preserved. These changes are included in version 1.4.1; final Store
assets and native acceptance remain pending.

| Finding | Implementation |
| --- | --- |
| F1 | Shared Field, Select, ToolButton, Preference and Stepper controls; rebuilt profiles, reading, history and keyboard help; removed native details disclosures. |
| F2 | Compact profile picker above setup; shortcuts and history in Settings; Gemini explanation in Engine and Reading; Start directly follows the checklist. |
| F3 | Keyboard-navigable Captions, Reading, Transcript history and App tabs. |
| F4 | Removed the primary Close footer; immediate-application note, close icon and Escape dismissal. |
| F5 | Profile summaries and Load actions, per-row management and deletion confirmation, explicit save/rename forms, desktop-only explanation. |
| F6 | Session list and detail pane, equal filter columns, Clear filters, title and export actions, narrow-window stacking. Unreadable records remain explicitly deletable. |
| F7 | Presets first, selection reflects current settings, bright/dark previews always expanded next to the appearance controls. |
| F8 | Shared 8 px control and 11 px card radius tokens across operator surfaces. Status pills and small graphic details retain their geometry. |
| F9 | Consistent Move overlay / Done wording and descriptive show/hide actions across EN/FR/DE. |
| F10 | Store product name in the app heading and document title; product-free Open window / Quit tray actions. |
| F11 | Localized tray labels cross the Tauri boundary with session state; disabled status item includes elapsed time while running. |
| F12 | System capture uses shared chevron selects; one rail action refreshes devices and applications. |
| F13 | Operator type tokens have an 11 px minimum at default Windows text size. |
| F14 | Caption size, compact width and hold controls display px, ch and s. |
| F15 | Five curated swatches for each colour role, selected-state outlines, labelled Custom controls and existing contrast feedback. |
| F16 | One keyboard-help surface under App; shortcut hint alongside Start. |
| F17 | Persistent Settings access to appearance, reading, history and app preferences; profiles remain with setup. |
| F18 | Removed duplicated elapsed figure from cost; clock remains in the status pill. Merged refresh actions. |

## Validation

- Svelte/TypeScript check: no errors or warnings.
- Final frontend suite: 337 tests passed with two workers after a clean locked install.
  The default-worker attempt hit worker startup timeouts during concurrent native compilation.
- Production frontend build passed.
- Rust formatting and all-target/all-feature Clippy passed with warnings denied.
- Native tests: 73 passed; the one billable provider probe remains ignored.
- Frontend formatting passed; dependency audit found no moderate-or-higher issues.
- Browser review covered French caption settings, German narrow-window app
  preferences, English reading controls, history empty state and keyboard tab
  navigation. No horizontal document overflow at the checked widths.
- Corrected Windows-encoded accented additions to UTF-8 and added a catalog
  regression check for accented labels/replacement characters.

The browser cannot exercise native audio, overlay-window placement or tray-menu
interaction. Those still require a Windows application smoke test. Final Store
screenshots must come from the packaged MSIX; the existing historical captures
were not replaced with browser screenshots. See the [1.4.1 release handoff](store-updates.md#release-141-handoff)
for the local installer build and outstanding submission checks.
