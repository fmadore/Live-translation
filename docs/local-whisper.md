# Local multilingual Whisper transcription

Included in **1.6.0**, merged in [PR #99](https://github.com/fmadore/Live-translation/pull/99).
Implements [issue #98](https://github.com/fmadore/Live-translation/issues/98). This is an
additional **Subtitles** provider; local translation is outside this feature’s scope.

Suggested by [@valentinrabot](https://github.com/valentinrabot) in
[issue #98](https://github.com/fmadore/Live-translation/issues/98): local transcription for
same-language meetings without ongoing API costs, with Whisper offered as a possible engine.

New installations open on Whisper with Base and automatic language detection selected.
Downloading a model and starting capture both require the user's action. Existing saved
engine, model, language and audio choices remain in place on upgrade.

## Using it

1. Choose Subtitles, the microphone/system/application source, and **Whisper** as the engine.
2. Select a spoken language, or leave **Detect automatically** selected. Choosing the known
   language helps with short utterances and avoids repeated language guessing.
3. Download a model. The download needs an internet connection; subsequent transcription is offline.
4. Start captioning. Captions feed the existing overlay, history, recovery and export workflows.
5. Stop when the meeting ends, then let **Finishing the transcript** reach completion before
   exporting the complete transcript. The pending duration reports unprocessed audio, not a
   prediction of processing time. Discarding pending audio requires confirmation and leaves
   an explicit incomplete-transcript notice.

| Model | Download | Intended choice |
| --- | ---: | --- |
| Tiny Q5_1 | 31 MiB | Lowest CPU and memory demand; try first on slower machines |
| Base Q5_1 (default) | 57 MiB | A balance between speed and recognition quality |
| Small Q5_1 | 182 MiB | More capacity, with higher CPU and memory demand |

All three are multilingual models, not the English-only `.en` variants. The selector exposes
all 99 language tokens they support, including French, English, German, Arabic, Japanese,
Spanish and Swahili. Support is not a claim of equal accuracy across languages. The separate
Cantonese token introduced with large-v3 is not advertised for these models. Interface languages
remain English, French and German, independently of the language being transcribed.

Models are pinned to Hugging Face `ggerganov/whisper.cpp` revision
`5359861c739e955e79d9a303bcbc70fb988958b1`. Exact sizes and SHA-256 digests are in
`src-tauri/src/whisper/models.rs`. Downloads use HTTPS, cancellation, timeouts and a temporary
file; only a completely verified download is installed. Installed bytes are verified again
before passing them to the native loader. Files in use cannot be deleted or replaced.

## Audio handling and limits

Capture produces approximately 100 ms of 16 kHz mono PCM16 per frame. A bounded queue gives
the disk writer 30 seconds of scheduling slack. The inference worker reads an anonymous local
temporary file rather than the cloud clients’ low-latency queues, so delayed recognition does
not trigger their stale-audio discard behavior. Buffering is bounded to 2 GiB per source
(roughly 18 hours of audio without the reader catching up and resetting the file); disk-full, I/O failure and writer overflow stop the
source and report that its transcript may be incomplete. Memory does not grow with the audio
backlog. The existing text transcript remains in memory and can use optional history/recovery.

Recognition uses windows of up to roughly 10 seconds, one second of overlap, conservative
silence endpointing and Whisper’s no-speech probability. It produces finalized captions, not
speculative word-by-word interim captions. A final short window is padded for inference and
its timestamps are clamped to the actual audio duration. Pauses retain elapsed meeting timing.
Exact overlap text is reconciled only for overlapping timestamps; repetitions later in the
meeting are retained. Accuracy at window boundaries, background noise and code-switching still
need real meeting acceptance tests. Whisper can misrecognize speech or hallucinate text.

Pause stops new audio entering transcription or its temporary file, while the existing backlog
continues processing. Normal Stop drains input and the final partial window without the cloud
pipeline’s five-second limit. An explicit discard token interrupts inference. Temporary audio
is delete-on-close and cannot be recovered after the process exits; see [privacy](privacy.md).

## Building

`whisper-rs` 0.16.0 vendors whisper.cpp through `whisper-rs-sys` 0.15.0. The CPU backend is
statically linked. No Python, external Whisper executable, GPU, CUDA or model is bundled in
the installer. Model loading uses Rust’s filesystem API so non-ASCII Windows usernames work.

Install LLVM/Clang with libclang, CMake and Ninja alongside the normal Rust/Tauri Windows
prerequisites. Run in a Visual Studio Developer PowerShell for the target architecture:

```powershell
$env:CC = 'clang-cl'
$env:CXX = 'clang-cl'
$env:CXXFLAGS = '/EHsc'
$env:CMAKE_GENERATOR = 'Ninja'
$env:LIBCLANG_PATH = 'C:\Program Files\LLVM\bin'
npm ci
npm run tauri dev
```

The native Windows ARM64 backend requires Clang; plain MSVC cannot compile this version of
GGML’s ARM CPU code. CI and release packaging use native Windows x64 and ARM64 runners, with
the same compiler setup. `.cargo/config.toml` disables build-host-specific instruction sets,
including AVX-only x64 builds, and targets baseline ARMv8. CPU portability takes precedence
over optional acceleration in this first implementation. Real-time throughput is not guaranteed;
Both sources runs two inference states and increases CPU/memory use.

Whisper.cpp and the OpenAI model weights use the MIT license; whisper-rs uses the Unlicense.
Their notices are included under `resources/licenses` and bundled with the app.

## Validation

CI builds/lints/tests the application on Linux, Windows x64 and Windows ARM64. The opt-in
`local_whisper_smoke` test downloads the pinned Tiny model and runs the production buffering,
segmentation and inference path over bundled English (auto-detect) and French (explicit-language)
recordings. It checks recognisable transcript content, monotonic timestamps and complete
EOF draining. Run it with:

```sh
cd src-tauri
cargo test --locked --all-features local_whisper_smoke -- --ignored --nocapture
```

Unit tests cover language vocabulary, corrupt model rejection, active-model protection,
writer overflow, pause filtering, final-tail handling, timestamp gaps and producer failure
ordering. UI tests cover persisted multilingual settings and model/progress controls.

Before release, test packaged x64 and ARM64 applications on real devices: download/cancel/remove,
relaunch offline, microphone and Teams/application loopback, Both sources, pause/resume, long
backlogs, Stop/discard, exports, non-Latin speech and clean uninstall. CI audio fixtures do not
establish microphone permissions, actual conference-call capture quality or battery performance.

Recorded on 4 October 2026 for merged PR #99: 538 frontend tests and 120 Rust tests passed
(3 opt-in Rust tests ignored in the ordinary suite). The opt-in Whisper smoke test passed on
ARM64 for English/French, including non-ASCII model paths. All seven PR CI jobs passed on
[run 37196818518](https://github.com/fmadore/Live-translation/actions/runs/37196818518).
A separate ARM64 Whisper Test install was verified as native ARM64; this is not final MSIX
acceptance. The [1.6.0 handoff](store-updates.md#release-160-handoff) records new package evidence.
