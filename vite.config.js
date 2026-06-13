import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Tauri expects a fixed dev port and looks for VITE_/TAURI_ env vars.
const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
	plugins: [sveltekit()],

	// Vite options tailored for Tauri development.
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
		host: host || false,
		hmr: host
			? { protocol: 'ws', host, port: 5183 }
			: undefined,
		watch: {
			// Don't watch the Rust side from Vite.
			ignored: ['**/src-tauri/**']
		}
	},
	// Surface Tauri's target so we can tune the build if needed.
	envPrefix: ['VITE_', 'TAURI_ENV_*']
});
