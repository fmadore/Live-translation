import { afterEach, describe, expect, it } from 'vitest';
import {
	captionBudget,
	captionLanguageOf,
	canFlipDirection,
	clampOverlayFont,
	clampOverlayWidth,
	DEFAULT_OVERLAY_FONT,
	DEFAULT_OVERLAY_WIDTH,
	loadOverlayFont,
	loadOverlayWidth,
	loadStartOptions,
	OVERLAY_WIDTH_MAX,
	OVERLAY_WIDTH_MIN,
	providerCanTranslate,
	providerDetectsLanguage,
	providerKeyName,
	providerRequiresKey,
	SESSION_OPTIONS_KEY
} from './types';
import { PROVIDER_META } from './providers';

// The operator UI used to offer "Flip mid-session with F2" over a handler that refused for
// the whole of a session (issue #21). Keeping the rule in one tested predicate is what stops
// the shortcut and the copy drifting apart again.
describe('canFlipDirection', () => {
	it('allows a swap while the operator is still setting a translation up', () => {
		expect(canFlipDirection('translate', false)).toBe(true);
	});

	it('refuses once controls are locked, which is the whole of a running session', () => {
		expect(canFlipDirection('translate', true)).toBe(false);
	});

	it('refuses in subtitle mode, which has no direction to flip', () => {
		expect(canFlipDirection('transcribe', false)).toBe(false);
		expect(canFlipDirection('transcribe', true)).toBe(false);
	});
});

// Gemini appears twice under two different model ids — Live Translate for captions,
// Transcribe Live for subtitles. These predicates are what keep the two apart everywhere:
// the rail that offers a backend, the guard that starts a session, and the credential row.
describe('the two Gemini backends', () => {
	it('routes each to exactly one mode', () => {
		expect(providerCanTranslate('gemini')).toBe(true);
		expect(providerCanTranslate('gemini-transcribe')).toBe(false);
	});

	it('shares one AI Studio credential, so switching mode never re-prompts', () => {
		expect(providerKeyName('gemini-transcribe')).toBe(providerKeyName('gemini'));
		expect(PROVIDER_META['gemini-transcribe'].keyUrl).toBe(PROVIDER_META.gemini.keyUrl);
		expect(providerRequiresKey('gemini-transcribe')).toBe(true);
	});

	it('bills the subtitle model separately from the translation model', () => {
		expect(PROVIDER_META['gemini-transcribe'].modelId).toBe('gemini-3.5-transcribe-live');
		expect(PROVIDER_META['gemini-transcribe'].hourlyEstimate).toBeLessThan(
			PROVIDER_META.gemini.hourlyEstimate
		);
	});
});

// Step 03 asks for a language only when the operator has one to give. Both subtitle engines
// identify the spoken language themselves; the built-in demo instead picks which script to play.
describe('providerDetectsLanguage', () => {
	it('is true for the backends that identify the spoken language themselves', () => {
		expect(providerDetectsLanguage('mistral')).toBe(true);
		expect(providerDetectsLanguage('gemini-transcribe')).toBe(true);
	});

	it('is false where the operator still chooses one', () => {
		expect(providerDetectsLanguage('gemini')).toBe(false);
		expect(providerDetectsLanguage('openai')).toBe(false);
		expect(providerDetectsLanguage('ondevice')).toBe(false);
	});
});

// This project runs in node, so the persisted-setup path needs a stand-in for the one
// storage method `loadStartOptions` reads.
function storing(record: unknown): void {
	Object.defineProperty(globalThis, 'localStorage', {
		configurable: true,
		value: { getItem: () => JSON.stringify(record) }
	});
}

describe('loadStartOptions with the Gemini subtitle backend', () => {
	afterEach(() => {
		Reflect.deleteProperty(globalThis, 'localStorage');
	});

	it('restores it, because it is a valid pairing with subtitle mode', () => {
		storing({
			source: 'system',
			mode: 'transcribe',
			targetLanguage: 'en',
			provider: 'gemini-transcribe',
			micDeviceName: null
		});
		const loaded = loadStartOptions();
		expect(loaded.provider).toBe('gemini-transcribe');
		expect(loaded.mode).toBe('transcribe');
		expect(loaded.source).toBe('system');
	});

	// The rail offers no such pair, so repairing one field would only guess which of the two
	// the operator meant — the whole record goes back to the keyless first-run defaults.
	it('discards a stored record that pairs it with translation', () => {
		storing({
			source: 'microphone',
			mode: 'translate',
			targetLanguage: 'fr',
			provider: 'gemini-transcribe'
		});
		expect(loadStartOptions().provider).toBe('ondevice');
	});
});

describe('overlay caption size and measure', () => {
	afterEach(() => {
		Reflect.deleteProperty(globalThis, 'localStorage');
	});

	function stored(value: unknown): void {
		Object.defineProperty(globalThis, 'localStorage', {
			configurable: true,
			value: { getItem: () => value }
		});
	}

	it('holds a requested measure inside the range, on a whole number of ch', () => {
		expect(clampOverlayWidth(30)).toBe(30);
		expect(clampOverlayWidth(2)).toBe(OVERLAY_WIDTH_MIN);
		expect(clampOverlayWidth(500)).toBe(OVERLAY_WIDTH_MAX);
		expect(clampOverlayWidth(31.6)).toBe(32);
	});

	// The stored value is a string from localStorage that the operator can edit and a crash
	// can truncate. Anything that is not a usable measure falls back rather than throwing —
	// the same contract the font size already had.
	it('falls back to the default for anything that is not a usable stored value', () => {
		for (const junk of [null, '', 'wide', 'NaN', '0', '-12', undefined]) {
			stored(junk);
			expect(loadOverlayWidth(), String(junk)).toBe(DEFAULT_OVERLAY_WIDTH);
		}
		stored('44');
		expect(loadOverlayWidth()).toBe(44);
		// Out of range on the way in, not just on the way out.
		stored('900');
		expect(loadOverlayWidth()).toBe(OVERLAY_WIDTH_MAX);
	});

	it('reports the default when there is no storage at all', () => {
		expect(loadOverlayWidth()).toBe(DEFAULT_OVERLAY_WIDTH);
		expect(loadOverlayFont()).toBe(DEFAULT_OVERLAY_FONT);
	});

	it('keeps the font size contract it was modelled on', () => {
		expect(clampOverlayFont(38)).toBe(38);
		expect(clampOverlayFont(4)).toBe(20);
		expect(clampOverlayFont(400)).toBe(96);
	});

	// The point of the whole exercise: an install that never touches the control must render
	// exactly what it rendered before the control existed.
	it('leaves the default output identical to the fixed measure it replaced', () => {
		expect(DEFAULT_OVERLAY_WIDTH).toBe(30);
		expect(captionBudget(DEFAULT_OVERLAY_WIDTH)).toBe(220);
	});

	// The budget is a vertical limit: it caps how much of the slide a streaming turn covers.
	// Holding it fixed while the measure moved would have made a wide caption cover less and
	// a narrow one cover more, which is the setting changing something nobody asked it to.
	it('spends the budget on the same number of lines at every measure', () => {
		// Exact to within the rounding, and no further: a budget is a whole number of
		// characters, so the proportion cannot survive to more decimal places than that.
		const perCh = captionBudget(DEFAULT_OVERLAY_WIDTH) / DEFAULT_OVERLAY_WIDTH;
		for (let width = OVERLAY_WIDTH_MIN; width <= OVERLAY_WIDTH_MAX; width += 1) {
			expect(Math.abs(captionBudget(width) - perCh * width), `at ${width}ch`).toBeLessThanOrEqual(
				0.5
			);
		}
	});

	it('never returns a budget from an out-of-range measure', () => {
		expect(captionBudget(1000)).toBe(captionBudget(OVERLAY_WIDTH_MAX));
		expect(captionBudget(0)).toBe(captionBudget(OVERLAY_WIDTH_MIN));
	});
});

// The overlay marks its captions up with this, so a wrong answer is a screen reader reading
// French with English phonemes and a missing answer is only a voice that stays neutral. The
// asymmetry is why "unknown" has to survive as far as the markup instead of being guessed.
describe('captionLanguageOf', () => {
	const base = { mode: 'translate', provider: 'gemini', targetLanguage: 'fr' } as const;

	it('names the target language when the app is translating into it', () => {
		expect(captionLanguageOf(base)).toBe('fr');
		expect(captionLanguageOf({ ...base, targetLanguage: 'en' })).toBe('en');
	});

	it('names it for the built-in demonstration, which plays a script in that language', () => {
		expect(
			captionLanguageOf({ mode: 'transcribe', provider: 'ondevice', targetLanguage: 'en' })
		).toBe('en');
	});

	// The stored target keeps whatever it was last set to, so it is present and plausible and
	// wrong — exactly the shape of value that gets used by accident.
	it('admits it does not know while a subtitle engine detects the spoken language', () => {
		for (const provider of ['mistral', 'gemini-transcribe'] as const) {
			expect(
				captionLanguageOf({ mode: 'transcribe', provider, targetLanguage: 'fr' }),
				provider
			).toBeUndefined();
		}
	});

	// One rule, stated twice, drifts. The app can name the caption language exactly when the
	// operator is allowed to choose it — `setTarget` refuses under the same condition.
	it('knows the language for precisely the settings that let the operator choose it', () => {
		const providers = Object.keys(PROVIDER_META) as Array<keyof typeof PROVIDER_META>;
		for (const provider of providers) {
			for (const mode of ['translate', 'transcribe'] as const) {
				const operatorMayChoose = mode === 'translate' || provider === 'ondevice';
				expect(
					captionLanguageOf({ mode, provider, targetLanguage: 'en' }) !== undefined,
					`${mode}/${provider}`
				).toBe(operatorMayChoose);
			}
		}
	});
});
