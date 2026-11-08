// Builds the Vulkan helper (src-tauri/whisper-vulkan) for one Windows target and puts it where
// a release build looks for it: src-tauri/binaries/whisper-vulkan-<target>.exe, the sidecar
// tauri.vulkan.conf.json declares (see docs/local-whisper.md#graphics-processor).
//
//   node scripts/build-whisper-vulkan.mjs x86_64-pc-windows-msvc|aarch64-pc-windows-msvc
//
// Needs LunarG's Vulkan SDK (VULKAN_SDK set, its Bin on PATH for the shader compiler) and the
// same Clang setup as the app's own Whisper build.

import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';

const target = process.argv[2];
const targets = ['x86_64-pc-windows-msvc', 'aarch64-pc-windows-msvc'];
if (!targets.includes(target)) {
	console.error(`Usage: node scripts/build-whisper-vulkan.mjs ${targets.join('|')}`);
	process.exit(1);
}
if (!process.env.VULKAN_SDK) {
	console.error('VULKAN_SDK is not set: install LunarG’s Vulkan SDK first.');
	process.exit(1);
}

execFileSync(
	'cargo',
	['build', '--release', '--locked', '-p', 'whisper-vulkan', '--target', target],
	{ cwd: 'src-tauri', stdio: 'inherit' }
);
mkdirSync('src-tauri/binaries', { recursive: true });
copyFileSync(
	`src-tauri/target/${target}/release/whisper-vulkan.exe`,
	`src-tauri/binaries/whisper-vulkan-${target}.exe`
);
console.log(`src-tauri/binaries/whisper-vulkan-${target}.exe`);
