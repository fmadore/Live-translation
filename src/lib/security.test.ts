import { readdirSync, readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// The windows' Content Security Policy, which only `src-tauri/tauri.conf.json` holds. Above all
// `connect-src 'self'` (AGENTS.md): providers are called from Rust only, so the interface never
// opens a connection of its own, and a key could not leave from the renderer even if script
// there were compromised. Tauri adds its own IPC sources to the policy at run time; nothing
// written here needs them. A change to this policy is a security decision, made here on
// purpose and explained in SECURITY.md, never a side effect.

const TAURI = new URL('../../src-tauri/', import.meta.url);

interface TauriConfig {
	app?: { security?: { csp?: unknown } & Record<string, unknown> };
}
const config = (file: string) =>
	JSON.parse(readFileSync(new URL(file, TAURI), 'utf8')) as TauriConfig;

const CSP =
	"default-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; " +
	"connect-src 'self'; object-src 'none'; base-uri 'none'; form-action 'none'; frame-src 'none'";

/** The policy as directive name → sources. */
function directives(csp: string): Map<string, string[]> {
	const map = new Map<string, string[]>();
	for (const part of csp.split(';')) {
		const [name, ...sources] = part.trim().split(/\s+/);
		if (name) map.set(name, sources);
	}
	return map;
}

describe('Content Security Policy', () => {
	const csp = config('tauri.conf.json').app?.security?.csp;

	it('is exactly the reviewed policy', () => {
		expect(csp).toBe(CSP);
	});

	it("lets the interface connect to its own origin only (connect-src 'self')", () => {
		expect(typeof csp).toBe('string');
		expect(directives(csp as string).get('connect-src')).toEqual(["'self'"]);
	});

	it('is not loosened by a platform or build configuration', () => {
		// tauri.macos.conf.json, tauri.appstore.conf.json, tauri.vulkan.conf.json…: each is
		// merged over tauri.conf.json, so a `security` block there would replace the policy.
		const overlays = readdirSync(TAURI).filter(
			(file) => /^tauri\..+\.conf\.json$/.test(file) && file !== 'tauri.conf.json'
		);
		expect(overlays.length).toBeGreaterThan(0);
		for (const file of overlays) {
			const security = config(file).app?.security;
			if (security === undefined) continue;
			expect(security.csp, file).toBe(CSP);
			expect(Object.keys(security), file).not.toContain('dangerousDisableAssetCspModification');
		}
	});
});
