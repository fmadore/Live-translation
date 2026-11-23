# Live Translation: instructions for coding agents

Read by Claude Code (through `CLAUDE.md`) and Codex. The [README](README.md) is the public
introduction and lists the commands; [docs/README.md](docs/README.md) indexes the guides;
[docs/architecture.md](docs/architecture.md) says how the app works and why; the
[roadmap](ROADMAP.md) says what is shipped and open. This file says how to change it.

When the `tauri-dev` skill is installed (it lives in `fmadore/agent-skills`), load it for
framework detail: Tauri's current versions and pitfalls, the ACL, CSP, signing and Store
packaging. This file and the docs win where they differ, and every rule here holds without it.

## Windows and macOS

- Windows (x64, ARM64) is the release target, through installers and the Microsoft Store. The
  macOS port is built, signed and launched in CI and has had one first run on a real Mac
  ([docs/macos.md](docs/macos.md), #195): every change must keep it compiling, and a pull
  request says what has not been tried on a Mac.
- CI runs Rust on Linux, Windows x64, Windows ARM64 and macOS. Code a Windows PC cannot compile
  (the `macos_impl` modules) is checked there before merging.
- What differs between systems is written per feature, not in a `platform/` layer: one public
  function with the same signature behind `#[cfg(windows)]`, `#[cfg(target_os = "macos")]` and
  a stub for other systems, each delegating to a `windows_impl` or `macos_impl` module
  ([docs/macos.md](docs/macos.md)). The interface asks `src/lib/platform.ts` and takes each
  system's wording from the `src/lib/i18n/mac.ts` overrides; components never test the system.
- Paths go through `Path`/`PathBuf`; nothing writes a `\` or `/` into a path.

## Rules

- **The two windows are not equally trusted**
  ([architecture](docs/architecture.md#the-two-windows-are-not-equally-trusted)). Adding a
  command is three edits: the handler in `lib.rs`, `src-tauri/src/command_names.rs`, and the
  capability of the window that calls it. The overlay never gets `core:event:default` (it
  includes `emit`) or core window permissions (they take any window's label); it gets
  overlay-only commands that act on the overlay alone.
- **Providers are called from Rust only.** Keys stay in Credential Manager or the Keychain and
  never reach the renderer, logs or the transcript; the CSP keeps `connect-src 'self'`.
- **Errors cross as `{ id, detail? }`.** The core names the failure in `errors.rs`; the
  interface words it in the catalogues ([docs/localization.md](docs/localization.md)).
- **Types are written on both sides** (`src-tauri/src/types.rs` and `src/lib/types.ts`).
  Change both; `src/lib/contract.test.ts` fails when commands, event names or serde strings
  drift apart.
- **No hard-coded interface text.** Every visible string, error sentences included, is in
  `en.ts`, `fr.ts` and `de.ts`. The interface language is not the caption language.
- **Real-time audio callbacks write only to bounded channels**; start, pause and stop go through
  the session's cancellation tokens and lifecycle lock
  ([architecture](docs/architecture.md#concurrency-and-shutdown)).
- **Never call a billable service in tests or CI**: providers are contract-tested against
  recorded messages and loopback servers.
- A change to what the app sends, to whom, or what it keeps updates
  [docs/privacy.md](docs/privacy.md) with a new effective date; a change to what it advertises
  updates [docs/store-listing-next.md](docs/store-listing-next.md).
- **Keep the public docs site current.** GitHub Pages publishes `docs/` at
  <https://fmadore.github.io/Live-translation/>. A change users can see, and every new feature,
  updates it in the same pull request: the user guide that covers it (or a new one, linked from
  the home page), and [docs/index.html](docs/index.html) when it changes what the home page
  says (features, engines, rates with their date, platforms, guides). Relative links stay
  inside `docs/`; `src/lib/docsLinks.test.ts` checks them.
- Check the latest version of a dependency on its registry before adding or bumping it.

## Before every commit

Run `npm run verify` and `npm run verify:rust`, as the README lists. The first runs
`npm run format:check`, `check`, `lint`, `knip`, `check:languages`, `npm test`, `npm run build`
and `npm run test:style`; the second runs `cargo fmt --all --check`,
`cargo clippy --locked --all-targets --all-features -- -D warnings` and
`cargo test --locked --all-features`, each with `--manifest-path src-tauri/Cargo.toml`, as CI
does. A change to the Whisper Vulkan helper also runs `npm run verify:vulkan` where the Vulkan
SDK is installed.
