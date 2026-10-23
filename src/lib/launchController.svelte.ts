import { fromStore, type Readable } from 'svelte/store';
import { locale, t } from './i18n';
import { captionLanguageError, type DemoLanguage } from './languages';
import { isRunning, options, sessionStartedAt, statusMessage } from './stores';
import type { StartOptions } from './types';

/** What a launch reads from the rest of the page: the session it starts, and the readiness
 *  answers other controllers own. */
export interface LaunchPort {
	/** A browser preview has no core to start. */
	browserMode: boolean;
	/** The session controller's start and its busy flag. */
	start: (options: StartOptions) => Promise<boolean>;
	sessionBusy: Readable<boolean>;
	/** A meeting profile is being loaded into the setup. */
	profileBusy: () => boolean;
	/** The chosen engine could start: a key, an installed model, or the demo's readiness. */
	engineReady: () => boolean;
	/** The chosen application to capture is running, when the setup names one. */
	applicationReady: (options: StartOptions) => boolean;
	/** Called as a launch goes ahead, before the start: the capture-failure banner goes. */
	launching: () => void;
}

/** Start, Rehearse and the start shortcut: what keeps a session from starting, and the one
 *  launch path they share.
 *
 *  Everything it answers is a getter over stores and the port rather than a `$derived`, so it
 *  is always current when read — from the page's template, where reading it tracks the stores
 *  behind it, and from a plain test, where nothing is tracked.
 *
 *  Every public method is an arrow-function field, so a call site can hand one on bare
 *  (`onStart={launch.start}`) without losing `this`. */
export class LaunchController {
	readonly #port: LaunchPort;
	readonly #options = fromStore(options);
	readonly #running = fromStore(isRunning);
	readonly #startedAt = fromStore(sessionStartedAt);
	readonly #messages = fromStore(t);
	readonly #locale = fromStore(locale);
	readonly #sessionBusy;

	constructor(port: LaunchPort) {
		this.#port = port;
		this.#sessionBusy = fromStore(port.sessionBusy);
	}

	/** Whether the last launch was a rehearsal — a session fed by the bundled sample recording
	 *  instead of live audio. It only ever reaches the backend as one extra field on the start
	 *  call: writing it into the options store would persist it, and the next launch would
	 *  silently rehearse instead of captioning the room.
	 *
	 *  Set by every launch and never reset, so it is read only together with a running
	 *  session (`rehearsalRunning`): nothing about how a run ends, or about the Idle a start
	 *  receives from the session it replaces, can take the rehearsal's rail from it while it
	 *  runs (review U1). */
	rehearsing = $state(false);

	/** A run has begun but no source has reported yet (review U7). `start_session` can answer
	 *  before the first status, and until one arrives the run is open but not yet running: a
	 *  second press, or the shortcut, would restart it. Start stays busy for that moment, as it
	 *  is while the start call is in flight, so it also keeps the focus it holds until Stop
	 *  takes its place. */
	get opening(): boolean {
		return this.#startedAt.current !== null && !this.#running.current;
	}

	get rehearsalRunning(): boolean {
		return this.rehearsing && this.#running.current;
	}

	/** Which language the sample recording should be spoken in. Translation rehearses on the
	 *  language the room is *not* reading, so the operator sees real translation rather than a
	 *  passthrough; the subtitle engines auto-detect and rehearse with the English fixture. The
	 *  built-in demonstration already owns a sample timeline, so it has no rehearsal. */
	get fixtureLanguage(): DemoLanguage {
		const setup = this.#options.current;
		if (setup.mode === 'translate') return setup.targetLanguage === 'en' ? 'fr' : 'en';
		return setup.provider === 'ondevice' && setup.targetLanguage === 'fr' ? 'fr' : 'en';
	}

	/** Why a caption language of the setup blocks Start, or empty. Either caption language can
	 *  be one the engine does not offer — the second one too, after an engine change. */
	get languageError(): string {
		return captionLanguageError(
			this.#options.current,
			this.#messages.current,
			this.#locale.current
		);
	}

	/** Everything that keeps a session from starting, in one place. Start, Rehearse and the
	 *  Ctrl+Shift+Space shortcut all read it, so they cannot drift apart again — Rehearse once
	 *  stayed enabled while a profile was loading. */
	get blockers() {
		return {
			busy:
				this.#port.browserMode ||
				this.#port.profileBusy() ||
				this.#sessionBusy.current ||
				this.#running.current ||
				this.opening,
			language: this.languageError,
			engine: !this.#port.engineReady(),
			// Only a live start opens the chosen application; a rehearsal plays the bundled
			// sample.
			application: !this.#port.applicationReady(this.#options.current)
		};
	}

	get canStart(): boolean {
		const b = this.blockers;
		return !b.busy && !b.language && !b.engine && !b.application;
	}

	get canRehearse(): boolean {
		const b = this.blockers;
		return !b.busy && !b.language && !b.engine && this.#options.current.provider !== 'ondevice';
	}

	/** Start and Rehearse share one launch path; a rehearsal differs only by the extra field.
	 *  The buttons are already disabled for every blocker. The shortcut is not, so the two
	 *  blockers an operator can act on from here say why rather than doing nothing — worded
	 *  where the status line shows them, so a change of interface language re-words them too. */
	launch = async (rehearsal?: DemoLanguage): Promise<void> => {
		const blockers = this.blockers;
		if (rehearsal === undefined ? !this.canStart : !this.canRehearse) {
			if (blockers.busy || blockers.engine) return;
			const setup = this.#options.current;
			if (blockers.language) statusMessage.set((m, tag) => captionLanguageError(setup, m, tag));
			else if (rehearsal === undefined) statusMessage.set((m) => m.applications.missing);
			return;
		}
		this.rehearsing = rehearsal !== undefined;
		this.#port.launching();
		const setup = this.#options.current;
		await this.#port.start(rehearsal === undefined ? setup : { ...setup, rehearsal });
	};

	start = (): Promise<void> => this.launch();
	rehearse = (): Promise<void> => this.launch(this.fixtureLanguage);
}

export function createLaunchController(port: LaunchPort): LaunchController {
	return new LaunchController(port);
}
