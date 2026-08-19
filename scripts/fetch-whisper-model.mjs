#!/usr/bin/env node
// Downloads the ggml speech model that backs the keyless on-device subtitle engine.
//
// The model is not committed — `tauri build` bundles it as an app resource (see
// `bundle.resources` in tauri.conf.json), so this must run before any release build; CI does
// so in release.yml. It is a no-op once the file is present.
//
// The default is the 5-bit quantized `base` model: 57 MiB rather than the 141 MiB of full
// `base`, for a small accuracy cost that is well within the margin of an engine already
// documented as less accurate than Voxtral. Installer size is a real constraint for Store
// distribution, so the quantized model is the better default here.
//
// Set WHISPER_MODEL to pick another from https://huggingface.co/ggerganov/whisper.cpp:
//   base-q5_1  57 MiB  default
//   base      141 MiB  unquantized, marginally better
//   small-q5_1 ~181 MiB  noticeably better, materially slower on CPU
// The app also honours WHISPER_MODEL_PATH at runtime to override whatever shipped.

import { createHash } from 'node:crypto';
import { createWriteStream } from 'node:fs';
import { mkdir, rename, stat, unlink } from 'node:fs/promises';
import { dirname, join } from 'node:path';
import { Readable } from 'node:stream';
import { pipeline } from 'node:stream/promises';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const repoRoot = join(here, '..');

const model = process.env.WHISPER_MODEL ?? 'base-q5_1';
const fileName = `ggml-${model}.bin`;
// tauri.conf.json bundles `resources/models/*.bin` into `models/`, and the app scans that
// directory, so the real model name is kept here rather than flattened to a fixed name.
const destination = join(repoRoot, 'src-tauri', 'resources', 'models', fileName);
const url = `https://huggingface.co/ggerganov/whisper.cpp/resolve/main/${fileName}?download=true`;

/** Human-readable byte count for progress output. */
function mib(bytes) {
	return `${(bytes / 1024 / 1024).toFixed(1)} MiB`;
}

async function alreadyPresent() {
	try {
		const info = await stat(destination);
		// A truncated download from an interrupted run would otherwise be treated as valid
		// and produce a confusing "failed to load the speech model" at runtime.
		if (info.size > 1024 * 1024) {
			console.log(`${destination} already present (${mib(info.size)}) — nothing to do.`);
			return true;
		}
		console.warn(`${destination} looks truncated (${mib(info.size)}); re-downloading.`);
		await unlink(destination);
	} catch {
		// Missing: fall through and download.
	}
	return false;
}

async function main() {
	if (await alreadyPresent()) return;

	await mkdir(dirname(destination), { recursive: true });
	console.log(`Downloading ${fileName} …`);

	const response = await fetch(url, { redirect: 'follow' });
	if (!response.ok || !response.body) {
		throw new Error(`download failed: HTTP ${response.status} ${response.statusText}`);
	}

	// Write to a temporary name and rename on success, so an interrupted run never leaves a
	// half-written file that looks like a valid model.
	const partial = `${destination}.partial`;
	const hash = createHash('sha256');
	const source = Readable.fromWeb(response.body);
	source.on('data', (chunk) => hash.update(chunk));
	await pipeline(source, createWriteStream(partial));
	await rename(partial, destination);

	const { size } = await stat(destination);
	console.log(`Wrote ${destination} (${mib(size)})`);
	console.log(`sha256 ${hash.digest('hex')}`);
}

main().catch(async (error) => {
	await unlink(`${destination}.partial`).catch(() => {});
	console.error(`\nCould not fetch the speech model: ${error.message}`);
	console.error('On-device subtitles will be unavailable until this succeeds.');
	process.exit(1);
});
