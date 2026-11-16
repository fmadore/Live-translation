// @vitest-environment jsdom
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get, type Readable } from 'svelte/store';
import storage114 from './__fixtures__/storage-1.14.0.json';

// What an operator's stored settings look like after an update. Every preference and profile
// is validated as it is read, and a value that fails falls back to its default without a word:
// right for corrupt storage, a silent reset if a release tightens a check or renames a value
// that an older release wrote. Each fixture is a whole release's storage, every key holding a
// valid value that is not its default, and the stores must read every one of them back.
//
// Add one fixture per release (`__fixtures__/storage-<version>.json`, every key that release
// reads) and list it below. Never edit an old one: it is what that release wrote, and is on
// operators' machines now. A key added later is simply absent from older fixtures, which reads
// as its default, as it does on a real update.

interface Fixture {
	release: string;
	storage: Record<string, unknown>;
}

const FIXTURES: Fixture[] = [storage114];

/** What localStorage holds for a fixture value: a string as it is, anything else as JSON. */
const stored = (value: unknown) => (typeof value === 'string' ? value : JSON.stringify(value));

/** The value a reader should return for a fixture value. */
type Decode = 'flag' | 'number' | 'string' | 'json';

function decode(how: Decode, value: unknown): unknown {
	switch (how) {
		case 'flag':
			return value === 'true';
		case 'number':
			return Number(value);
		case 'string':
			return value;
		case 'json':
			return value;
	}
}

/** Every module that reads storage, imported afresh after the fixture is in place: most of
 *  them read it as they load. */
async function loadModules() {
	const [
		preferences,
		theme,
		exportFormat,
		history,
		aiConsent,
		i18n,
		overlayGeometry,
		profiles,
		persisted
	] = await Promise.all([
		import('./preferences'),
		import('./theme'),
		import('./exportFormat'),
		import('./history'),
		import('./aiConsent'),
		import('./i18n'),
		import('./overlayGeometry'),
		import('./profiles'),
		import('./persisted')
	]);
	const store = (s: Readable<unknown>) => () => get(s);
	const p = preferences;
	// Each key 1.14.0 stores, with how the app reads it back and what it should read.
	const readers: Record<string, [read: () => unknown, decode: Decode]> = {
		'session.options': [store(p.options), 'json'],
		'overlay.fontSize': [store(p.overlayFontSize), 'number'],
		'overlay.captionWidth': [store(p.overlayCaptionWidth), 'number'],
		'overlay.captionLayout': [store(p.overlayCaptionLayout), 'string'],
		'overlay.captionFace': [store(p.overlayCaptionFace), 'string'],
		'overlay.captionColour': [() => get(p.overlayPalette).text, 'string'],
		'overlay.scrimColour': [() => get(p.overlayPalette).scrim, 'string'],
		'overlay.scrimOpacity': [() => get(p.overlayPalette).scrimOpacity, 'number'],
		'overlay.placed': [store(p.overlayPlaced), 'flag'],
		'overlay.cleanSpeech': [store(p.overlayCleanSpeech), 'flag'],
		'overlay.fillerWords': [store(p.overlayFillerWords), 'json'],
		'overlay.holdSeconds': [store(p.overlayHoldSeconds), 'number'],
		'overlay.pace': [store(p.overlayPace), 'string'],
		'overlay.showOriginal': [store(p.overlayShowOriginal), 'flag'],
		'overlay.geometry': [overlayGeometry.loadGeometry, 'json'],
		'recovery.enabled': [store(p.recoveryEnabled), 'flag'],
		'transcript.includeOriginal': [store(p.exportOriginal), 'flag'],
		'transcript.exportFormat': [store(exportFormat.exportFormat), 'string'],
		'transcript.historyEnabled': [store(history.historyEnabled), 'flag'],
		'stage.view': [store(p.stageView), 'string'],
		'window.closeToTray': [store(p.closeToTray), 'flag'],
		'window.trayHideExplained': [store(p.trayHideExplained), 'flag'],
		'language.favourites': [store(p.languageFavourites), 'json'],
		'appearance.theme': [store(theme.theme), 'string'],
		'privacy.aiConsent': [store(aiConsent.aiConsent), 'json'],
		'ui.locale': [store(i18n.locale), 'string'],
		// Read by MeetingProfiles.svelte through this decoder.
		'meeting.profiles': [
			() => profiles.decodeProfiles(persisted.readStored(profiles.PROFILES_KEY)),
			'json'
		]
	};
	return readers;
}

beforeEach(() => {
	vi.resetModules();
	localStorage.clear();
});

afterEach(() => {
	localStorage.clear();
	vi.resetModules();
});

describe.each(FIXTURES)('stored state of $release', ({ storage }) => {
	beforeEach(() => {
		for (const [key, value] of Object.entries(storage)) localStorage.setItem(key, stored(value));
	});

	it('reads back every stored value, none reset to its default', async () => {
		const readers = await loadModules();
		for (const [key, value] of Object.entries(storage)) {
			const reader = readers[key];
			// A key this release stored that nothing reads any more: deliberate removals add
			// their own migration test, not a quiet drop.
			expect(reader, `no reader for "${key}"`).toBeDefined();
			const [read, how] = reader;
			expect(read(), key).toEqual(decode(how, value));
		}
	});

	it('leaves what is stored as it was', async () => {
		// The stores write their value back as they load; a value read as something else would
		// overwrite the operator's choice in storage, too.
		await loadModules();
		for (const [key, value] of Object.entries(storage)) {
			const now = localStorage.getItem(key);
			if (typeof value === 'string') expect(now, key).toBe(value);
			else expect(JSON.parse(now ?? 'null'), key).toEqual(value);
		}
	});
});
