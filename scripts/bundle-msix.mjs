// Builds the Microsoft Store package for one architecture with @choochmeque/tauri-windows-bundle,
// with the Vulkan helper inside (see docs/packaging-msix.md).
//
//   node scripts/bundle-msix.mjs x64|arm64      (npm run bundle:msix:x64 / bundle:msix:arm64)
//
// The bundler reads tauri.conf.json and tauri.windows.conf.json only, and `tauri build` merges the
// latter on Windows by itself, so the release-only configuration (tauri.vulkan.conf.json: the
// helper as a sidecar) is laid in as tauri.windows.conf.json for the build and removed afterwards:
// an ordinary build, CI's tests or `tauri dev` must not need the helper built. Build it first:
// `node scripts/build-whisper-vulkan.mjs <target>`. The same pattern as Kalamu's script.

import { execFileSync } from 'node:child_process';
import { copyFileSync, existsSync, rmSync } from 'node:fs';

const arch = process.argv[2];
if (arch !== 'x64' && arch !== 'arm64') {
	console.error('Usage: node scripts/bundle-msix.mjs x64|arm64');
	process.exit(1);
}

const target = arch === 'x64' ? 'x86_64-pc-windows-msvc' : 'aarch64-pc-windows-msvc';
if (!existsSync(`src-tauri/binaries/whisper-vulkan-${target}.exe`)) {
	console.error(
		`The Vulkan helper is missing: run \`node scripts/build-whisper-vulkan.mjs ${target}\` first.`
	);
	process.exit(1);
}

const windows = 'src-tauri/tauri.windows.conf.json';
copyFileSync('src-tauri/tauri.vulkan.conf.json', windows);
try {
	execFileSync(
		process.execPath,
		[
			'node_modules/@choochmeque/tauri-windows-bundle/dist/cli.js',
			'build',
			'--arch',
			arch,
			'--runner',
			'npm'
		],
		{ stdio: 'inherit' }
	);
} finally {
	rmSync(windows, { force: true });
}
