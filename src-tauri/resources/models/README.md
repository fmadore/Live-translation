# On-device speech model

The keyless **Live subtitles → On-device** engine loads a ggml Whisper model from this
directory. The model is tens of megabytes, so it is **not committed** — run:

```bash
npm run fetch:whisper-model
```

`tauri.conf.json` bundles everything here into the app's `models/` resource directory, and
`src-tauri/src/ondevice/engine.rs` picks the first `*.bin` it finds (sorted, so the choice is
stable). Without a model the app still builds and runs; selecting the on-device engine simply
reports that the model is missing.

This file exists so the resource glob always matches something. Tauri treats a glob that
matches no files as a build error, and requiring the model at build time would mean every CI
compile downloading it.

The default is `base-q5_1` (57 MiB), 5-bit quantized to keep the installer small. Set
`WHISPER_MODEL=base` for the unquantized 141 MiB version or `WHISPER_MODEL=small-q5_1` for
better accuracy at a real CPU cost, or point `WHISPER_MODEL_PATH` at a `.bin` at runtime to
override whatever shipped.
