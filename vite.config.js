import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';
import { sveltekit } from '@sveltejs/kit/vite';
// vitest's re-export, not vite's: only this one accepts the `test` block below.
import { defineConfig } from 'vitest/config';

// Tauri expects a fixed dev port and looks for VITE_/TAURI_ env vars.
const host = process.env.TAURI_DEV_HOST;

// Fontsource lists a `.woff` fallback after every `.woff2`. WebView2 always takes the `.woff2`,
// so the fallbacks were never loaded, yet each one was still bundled into the app — about
// 350 KB. Dropping them from the CSS before Vite resolves its URLs keeps them out of the build.
/** @returns {import('vite').Plugin} */
function woff2Only() {
	return {
		name: 'woff2-only',
		enforce: 'pre',
		transform(code, id) {
			if (!/[\\/]@fontsource[\\/].+\.css$/.test(id)) return;
			return code.replace(/,\s*url\([^)]+\.woff\)\s*format\('woff'\)/g, '');
		}
	};
}

export default defineConfig({
	plugins: [
		woff2Only(),
		sveltekit({
			preprocess: vitePreprocess(),
			// Tauri serves a static bundle; no Node server at runtime.
			adapter: adapter({ fallback: 'index.html' }),
			// A desktop app is updated by its installer, not redeployed under a running window, so
			// there is nothing for SvelteKit 3's hourly `version.json` poll to find. Focus checks stay:
			// they read the same bundled file over the app's own origin.
			version: { pollInterval: 0 }
		})
	],

	// Vite options tailored for Tauri development.
	clearScreen: false,
	server: {
		port: 5173,
		strictPort: true,
		host: host || false,
		hmr: host ? { protocol: 'ws', host, port: 5183 } : undefined,
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
		// `npm run test:coverage`; CI prints the summary with `scripts/coverage-summary.mjs`.
		coverage: {
			provider: 'v8',
			// Every source file, so a module no test imports shows as uncovered, not absent.
			include: ['src/**/*.{ts,svelte}'],
			exclude: ['src/**/*.test.ts', 'src/**/*.d.ts', 'src/lib/testing/**'],
			reporter: ['text-summary', 'json-summary', 'html'],
			// Per file only for the pure modules: no Tauri, no DOM, every line reachable from a
			// plain test, and each fully covered today. A floor there catches a deleted test or a
			// new untested branch; elsewhere a per-file number would only invite tests written for
			// it.
			thresholds: {
				'src/lib/{appearance,captionColour,captionLayout,cleanSpeech,document,errors,glossary,historySearch,languages,reading,shortcuts,transcript}.ts':
					{ perFile: true, lines: 95, statements: 95, functions: 95, branches: 85 },
				// The whole of `src`, a point or so under the totals of 22 October 2026 (90.97% of
				// lines, 90.32% of statements, 87.76% of functions, 84.21% of branches). A ratchet
				// against a total that falls, not a target: raise it when the totals rise, and never
				// write a test just to clear it.
				lines: 90,
				statements: 89,
				functions: 87,
				branches: 83
			}
		},
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
					// One jsdom per worker instead of one per file, with each file still in its own
					// VM context. It halves this project's time. The logic project stays isolated the
					// ordinary way: sharing a context there lets `vi.mock` leak between files.
					pool: 'vmThreads',
					clearMocks: true,
					// The first test in a file pays for compiling the page it mounts, about half a
					// second alone but past Vitest's 5 s default on a machine busy with a Rust
					// build. A hang still fails, just later.
					testTimeout: 15_000,
					include: ['src/**/*.svelte.test.ts'],
					setupFiles: ['./vitest-setup-client.ts']
				}
			}
		]
	}
});
