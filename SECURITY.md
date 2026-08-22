# Security policy

## Supported versions

Only the latest tagged release receives fixes. Installers are published per tag on the
[releases page](https://github.com/fmadore/Live-translation/releases).

## Reporting a vulnerability

Report privately through
[GitHub's private vulnerability reporting](https://github.com/fmadore/Live-translation/security/advisories/new)
rather than opening a public issue. Expect an acknowledgement within a week.

## How this app handles credentials

- Provider API keys (Gemini, OpenAI, Mistral) are stored in the OS credential store —
  Windows Credential Manager — and are read only by the Rust core. They are never written to the frontend, to disk, or to logs.
- The webview runs under a restrictive CSP (`connect-src 'self'`); all provider WebSocket
  traffic originates from Rust, not the frontend.
- `.env` is git-ignored and intended for development only. `.env.example` contains variable
  names, never values.

## What leaves the machine

Captured audio is streamed to whichever provider you select, under that provider's terms.
Nothing is sent when no session is running. Transcripts are written only when you choose
**Save text** or **Save Markdown**, under `Documents/Live-translation/`. A native Save As
workflow is planned in [issue #26](https://github.com/fmadore/Live-translation/issues/26).

## Installer signing

The NSIS installers on GitHub Releases are currently **unsigned**, so Windows SmartScreen may
warn on first launch ("Windows protected your PC" — choose **More info → Run anyway**). Verify
downloads come from the official releases page above. Version 1.0.0 has also been submitted as
an MSIX to the Microsoft Store and is awaiting certification; Store-distributed packages are
signed by Microsoft. See [`docs/microsoft-store.md`](docs/microsoft-store.md).
