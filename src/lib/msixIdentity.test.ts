import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';

// The Microsoft Store package's identity. Partner Center assigned the identity name, the display
// name and the publisher, and ingestion refuses a package unless they match byte for byte (see
// docs/microsoft-store.md, "Store identity (assigned)"); the microphone is the one capability the
// app declares beyond the bundler's runFullTrust. Changing any of these is a Store decision,
// taken in Partner Center first: update the pins here only then.

const read = (file: string) =>
	readFileSync(new URL(`../../src-tauri/gen/windows/${file}`, import.meta.url), 'utf8');

describe('MSIX identity', () => {
	const manifest = read('AppxManifest.xml.template');
	const bundle = JSON.parse(read('bundle.config.json')) as {
		publisher?: string;
		publisherDisplayName?: string;
		capabilities?: { device?: string[] };
	};

	it('keeps the identity name Partner Center assigned', () => {
		expect(manifest).toMatch(/<Identity\s+Name="49346FMadore\.LiveTranslationSubtitles"\s/);
	});

	it('keeps the display name Partner Center reserved', () => {
		expect(manifest).toContain('<DisplayName>Live Translation &amp; Subtitles</DisplayName>');
		expect(manifest).toContain('DisplayName="Live Translation &amp; Subtitles"');
	});

	it('takes the publisher from the bundle configuration', () => {
		expect(manifest).toContain('Publisher="{{PUBLISHER}}"');
		expect(bundle.publisher).toBe('CN=5D0ECC96-3998-452E-B7E9-29BE9B576F86');
		expect(bundle.publisherDisplayName).toBe('FMadore');
	});

	it('declares the microphone and no other device capability', () => {
		expect(bundle.capabilities?.device).toEqual(['microphone']);
	});
});
