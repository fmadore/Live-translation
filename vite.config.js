import { sveltekit } from '@sveltejs/kit/vite';
// vitest's re-export, not vite's: only this one accepts the `test` block below.
import { defineConfig } from 'vitest/config';

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
	// Surface Tauri's target so we can tune the build if needed. (Vite prefixes are
	// literal `startsWith` matches — no globs.)
	envPrefix: ['VITE_', 'TAURI_ENV_'],

	// Two projects rather than one jsdom environment for everything. Pure logic stays in node,
	// which is faster and means a test cannot start depending on the DOM by accident; anything
	// that renders a component opts in by being named `*.svelte.test.ts`.
	test: {
		projects: [
			{
				extends: true,
				test: {
					name: 'logic',
					environment: 'node',
					include: ['src/**/*.test.ts'],
					exclude: ['src/**/*.svelte.test.ts']
				}
			},
			{
				extends: true,
				// Without the browser condition Svelte resolves to its server build and
				// `mount()` throws — components can only be rendered against the client entry.
				resolve: { conditions: ['browser'] },
				test: {
					name: 'component',
					environment: 'jsdom',
					clearMocks: true,
					include: ['src/**/*.svelte.test.ts'],
					setupFiles: ['./vitest-setup-client.ts']
				}
			}
		]
	}
});
