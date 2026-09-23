<script lang="ts">
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import MeetingProfiles from '$lib/MeetingProfiles.svelte';
	import OperatorTitlebar from '$lib/OperatorTitlebar.svelte';
	import SessionControls from '$lib/SessionControls.svelte';
	import DeviceRecoveryBanner from '$lib/DeviceRecoveryBanner.svelte';
	import LiveTurns from '$lib/LiveTurns.svelte';
	import LiveRail from '$lib/LiveRail.svelte';
	import SetupSheet from '$lib/SetupSheet.svelte';
	import PreflightChecklist from '$lib/PreflightChecklist.svelte';
	import TranscriptMonitor from '$lib/TranscriptMonitor.svelte';
	import SettingsDialog, { type SettingsTab } from '$lib/SettingsDialog.svelte';
	import ActiveSessionPrompt from '$lib/ActiveSessionPrompt.svelte';
	import RecoveryPrompt from '$lib/RecoveryPrompt.svelte';
	import TrayHidePrompt from '$lib/TrayHidePrompt.svelte';
	import UnsavedPrompt from '$lib/UnsavedPrompt.svelte';
	import { createPreflightController } from '$lib/preflightController.svelte';
	import { createQuitController } from '$lib/quitController.svelte';
	import { createOverlayController } from '$lib/overlayController.svelte';
	import { createSessionClock } from '$lib/sessionClock.svelte';
	import { createSessionController } from '$lib/sessionController';
	import { createSetupActions } from '$lib/setupActions';
	import { languageName, supportsLanguage, type DemoLanguage } from '$lib/languages';
	import { shortcut } from '$lib/shortcuts';
	import { api, on, isTauri } from '$lib/tauri';
	import { asStatus, describeError, isAppError } from '$lib/errors';
	import {
		sessionState,
		isRunning,
		statusMessage,
		applyStatus,
		hasKey,
		options,
		transcript,
		transcriptDirty,
		recoveryEnabled,
		restoreTranscript,
		closeToTray,
		overlayFontSize,
		noteActivity,
		sessionStartedAt,
		pushCaption
	} from '$lib/stores';
	import { decodeRecovery, shouldGuardClose, type RecoverySnapshot } from '$lib/document';
	import { recovery, startRecoverySpool } from '$lib/recovery';
	import { followTextScale } from '$lib/textScale';
	import { historyEnabled } from '$lib/history';
	import { captionLanguageOf, providerRequiresKey } from '$lib/types';
	import type { Origin, Provider, SessionState } from '$lib/types';
	import { formatDateTime, localeTag, locale, t } from '$lib/i18n';

	// Resolved at component init, not in `onMount`. `isTauri()` is a synchronous property
	// check and this app never server-renders (`ssr = false` in +layout.ts), so the answer is
	// available on the very first render — which is what keeps a child like ApiKeyPanel from
	// mounting and firing a Tauri-only command during `npm run dev`.
	const browserMode = !isTauri();
	const session = createSessionController();
	const overlay = createOverlayController();
	const quit = createQuitController({
		stop: () => session.stop(),
		toggleOverlayVisible: () => overlay.toggleOverlayVisible()
	});
	const sessionBusy = session.busy;
	let profileBusy = $state(false);
	const controlsLocked = $derived($isRunning || $sessionBusy || profileBusy);
	const preflight = createPreflightController(
		!browserMode,
		() => controlsLocked || failedDevice !== null || retryingDevice
	);
	let failedDevice = $state<Origin | null>(null);
	let retryingDevice = $state(false);
	$effect(() => {
		if (!controlsLocked && !preflight.audioTesting && !preflight.audioTestBusy)
			preflight.validateSelection();
	});
	async function reselectApplication() {
		await session.stop();
		if ($isRunning) return;
		failedDevice = null;
		$options = { ...$options, systemCapture: { kind: 'application', process: null } };
		await preflight.refreshApplications();
	}
	async function retryDevice(fallback: boolean) {
		if (!failedDevice || retryingDevice) return;
		retryingDevice = true;
		const affected = failedDevice;
		// Snapshot before stopping: idle validation must not change an explicit Retry
		// into an implicit fallback if the chosen endpoint is still absent.
		const selected = { ...$options };
		try {
			await session.stop();
			if ($isRunning) return;
			if (fallback) {
				if (affected === 'microphone') {
					selected.micDeviceId = null;
					selected.micDeviceName = null;
				} else selected.systemDeviceId = null;
			}
			$options = selected;
			failedDevice = null;
			await session.start(selected);
		} finally {
			retryingDevice = false;
		}
	}

	// The keyless demonstration is always bundled and ready. A commercial
	// provider starts NOT ready: clearing the flag on the switch itself closes the
	// tick where the previous provider's `true` would leave Start enabled before the
	// remounted ApiKeyPanel has re-checked the keychain.
	const needsKey = $derived(providerRequiresKey($options.provider));
	$effect(() => {
		hasKey.set(needsKey ? false : (preflight.localReadiness?.ready ?? false));
	});

	// Ticks only while a session is open.
	const clock = createSessionClock();

	// True for the duration of a rehearsal run — a session fed by the bundled sample recording
	// instead of live audio. It only ever reaches the backend as one extra field on the start
	// call: writing it into the options store would persist it, and the next launch would
	// silently rehearse instead of captioning the room. Declared here because the meter gates
	// below depend on it.
	let rehearsing = $state(false);

	// A rehearsal is a single System-origin stream no matter what source is saved, so the
	// meters follow the fixture rather than showing a dead Room meter. Idle is unaffected
	// (`rehearsing` is false there, so the pre-flight audio check keeps its semantics).
	const usesMic = $derived(!rehearsing && $options.source !== 'system');
	const usesSystem = $derived(rehearsing || $options.source !== 'microphone');

	onMount(() => {
		if (browserMode) return;

		void preflight.refresh();
		void preflight.refreshLocalReadiness();
		void loadRecovery();
		overlay.initialize();

		const unlisteners: Array<Promise<() => void>> = [
			// Windows' accessibility text size, which WebView2 does not pass on by itself.
			// This window honours it; the overlay deliberately does not — see
			// `docs/accessibility.md`.
			followTextScale(),
			on.caption((c) => pushCaption(c)),
			on.level((l) => {
				preflight.noteLevel(l);
				if (l.rms > 0.02 && $isRunning) noteActivity(l.source, 'audio');
			}),
			on.status((s) => {
				applyStatus(s);
				if (
					s.state === 'error' &&
					isAppError(s.message) &&
					['error.micCapture', 'error.micStream', 'error.systemCapture'].includes(s.message.id)
				) {
					failedDevice =
						s.origin ?? (s.message.id === 'error.systemCapture' ? 'system' : 'microphone');
				}
			}),
			on.devicesChanged(() => void preflight.refresh()),
			// A test is not a session, so it reports on its own channel and never touches the
			// session state machine. Rust is authoritative: it also ends the test when a
			// session starts, and says so here.
			// Closing with unsaved captions, or mid-session, is held by the core until this
			// answers it — see `lifecycle::CloseGuard`.
			on.closeRequested(() => void quit.onCloseRequested()),
			// Tray entries that need session or transcript state to carry out.
			on.trayCommand((command) => void quit.onTrayCommand(command)),
			// Named `audioTest` rather than `t`, which is now the message catalog.
			on.audioTest(preflight.applyAudioTest),
			// The overlay can be locked, placed and resized from its own toolbar; mirror that
			// back so the rail and the pre-flight check don't drift out of sync.
			on.overlayState(overlay.applyState)
		];

		return () => {
			preflight.dispose();
			void Promise.all(unlisteners).then((fns) => fns.forEach((f) => f()));
		};
	});

	// ---- Quit and crash safety (issue #25) --------------------------------------
	// The transcript is a document with a saved state, not a scrolling side effect: it is
	// never truncated, closing the window with unsaved lines asks first, and — only if the
	// operator opts in — a local spool covers the crash the prompt cannot.

	/** How often the opt-in spool is refreshed while captions are arriving. Long enough that a
	 *  busy session is not writing constantly, short enough that a crash costs a sentence. */

	let recovered = $state<{ snapshot: RecoverySnapshot; path: string } | null>(null);

	// Keep the core's guard current. While it and the tray preference are both false a close
	// is not intercepted at all, so a wedged renderer cannot produce an unclosable window.
	$effect(() => {
		if (browserMode) return;
		void api.setCloseGuard(shouldGuardClose($transcriptDirty, $isRunning)).catch(() => {});
	});

	// The tray preference is a standing setting rather than something session state decides,
	// so the core tracks it separately.
	$effect(() => {
		if (browserMode) return;
		void api.setCloseToTray($closeToTray).catch(() => {});
	});

	// The tray menu must never describe a state the app has left: pushed on every change, and
	// once on mount.
	$effect(() => {
		if (browserMode) return;
		void api
			.setTrayState($isRunning, overlay.overlayVisible, {
				open: $t.design.trayOpen,
				quit: $t.design.trayQuit,
				stop: $t.design.trayStop,
				show: $t.design.trayShow,
				hide: $t.design.trayHide,
				status: `${stateLabel[$sessionState]}${$isRunning ? ' · ' + clock.elapsed : ''}`
			})
			.catch(() => {});
	});

	// ---- Recovery spool ---------------------------------------------------------

	$effect(() => {
		if (browserMode || !$recoveryEnabled) return;
		return startRecoverySpool(
			() => (get(transcriptDirty) ? get(transcript) : null),
			(error) => statusMessage.set(get(t).error.recoveryWrite(String(error)))
		);
	});

	async function loadRecovery() {
		try {
			const stored = await recovery.read();
			if (!stored) return;
			const snapshot = decodeRecovery(stored.contents);
			if (!snapshot) {
				// Truncated mid-write, or hand-edited. There is nothing to offer, and leaving it
				// would strand caption text on disk that no prompt will ever clear.
				await recovery.clear();
				return;
			}
			recovered = { snapshot, path: stored.path };
		} catch (error) {
			statusMessage.set(asStatus(error));
		}
	}

	/** Answer the recovery offer. Either answer retires the spool — the operator has now
	 *  decided, and a file nobody chose to keep must not survive the decision. */
	async function answerRecovery(restore: boolean) {
		const found = recovered;
		recovered = null;
		if (!found) return;
		if (restore) restoreTranscript(found.snapshot.lines);
		try {
			await recovery.clear();
		} catch (error) {
			statusMessage.set(asStatus(error));
		}
	}

	// ---- Launching --------------------------------------------------------------
	// The session's own clock is what settles back after a run: `beginSession()` sets it, and
	// the whole-session idle status clears it (stores.ts), so it is null exactly when the app is
	// back at idle — including after a stop, an error, or a backend-side end of session.
	$effect(() => {
		if ($sessionStartedAt === null) rehearsing = false;
	});

	/** Which language the sample recording should be spoken in. Translation rehearses on the
	 *  language the room is *not* reading, so the operator sees real translation rather than a
	 *  passthrough; the subtitle engines auto-detect and rehearse with the English fixture. The built-in
	 *  demonstration already owns a sample timeline, so its separate rehearsal control is disabled. */
	const fixtureLanguage = $derived<DemoLanguage>(
		$options.mode === 'translate'
			? $options.targetLanguage === 'en'
				? 'fr'
				: 'en'
			: $options.provider === 'ondevice'
				? $options.targetLanguage === 'fr'
					? 'fr'
					: 'en'
				: 'en'
	);

	// Start and Rehearse share one launch path; a rehearsal differs only by the extra field.
	async function launch(rehearsal?: DemoLanguage) {
		if ($sessionBusy || $isRunning || profileBusy) return;
		if (languageError) {
			statusMessage.set(languageError);
			return;
		}
		if (!rehearsal && !preflight.applicationReady($options)) {
			statusMessage.set($t.applications.missing);
			return;
		}
		rehearsing = rehearsal !== undefined;
		failedDevice = null;
		const started = await session.start(
			rehearsal === undefined ? $options : { ...$options, rehearsal }
		);
		if (!started) rehearsing = false;
	}

	const start = () => launch();
	const rehearse = () => launch(fixtureLanguage);

	const stop = () => session.stop();

	const actions = createSetupActions({
		locked: () => controlsLocked,
		invalidateAudioTest: () => preflight.invalidateAudioTest(),
		refreshDevices: () => void preflight.refresh()
	});

	let settingsOpen = $state(false);
	let settingsTab = $state<SettingsTab>('captions');

	// The overlay is a separate webview, so the operator's interface language and the audience's
	// caption language are pushed to it the same way the caption size is — on load, which also
	// syncs the rest of the appearance, and on every change. One effect, so load sends one
	// config rather than one per language. Skipped in a browser preview, which has no second
	// window.
	const captionLanguage = $derived(captionLanguageOf($options));
	$effect(() => {
		const config = { locale: $locale, captionLanguage };
		if (!browserMode) overlay.pushOverlayConfig(config);
	});

	// The status line: plain text as it stands, a core failure as the sentence for its id plus
	// the technical detail. Derived rather than stored, so switching language re-words a
	// message that is already on screen.
	const statusText = $derived($statusMessage ? describeError($statusMessage, $t) : '');

	// ---- Display labels ---------------------------------------------------------
	//
	// Every word below comes from the catalog, so a change of interface language re-renders
	// the rail and the stage without touching the session.

	const stateLabel = $derived<Record<SessionState, string>>($t.state);

	// What a screen reader hears when the session changes state. Deliberately separate from the
	// pill: the pill carries a clock that reprints every second, and a live region wrapped
	// around a ticking clock announces the whole session state every second with it.
	const stateAnnouncement = $derived<Record<SessionState, string>>($t.announce);

	const languageError = $derived(
		supportsLanguage($options.provider, $options.targetLanguage)
			? ''
			: $t.language.unsupported(
					$t.engine[$options.provider],
					languageName($options.targetLanguage, $locale)
				)
	);
</script>

<svelte:window
	onkeydown={(e) => {
		const command = shortcut(
			e,
			settingsOpen ||
				quit.hidePrompt ||
				quit.sessionPrompt ||
				quit.closePrompt ||
				!!document.querySelector('[role="dialog"]')
		);
		if (!command) return;
		e.preventDefault();
		if (command === 'direction') actions.flipDirection();
		if (command === 'larger') overlay.setFont($overlayFontSize + 2);
		if (command === 'smaller') overlay.setFont($overlayFontSize - 2);
		if (command === 'toggleOverlay' && !browserMode) void overlay.toggleOverlayVisible();
		if (command === 'toggleSession' && !browserMode && !profileBusy && !$sessionBusy) {
			if ($isRunning) void stop();
			else if ($hasKey && preflight.applicationReady($options)) void start();
		}
	}}
/>

<div class="app" class:device-error={failedDevice !== null}>
	<OperatorTitlebar
		elapsed={clock.elapsed}
		{settingsOpen}
		onOpenSettings={() => (settingsOpen = true)}
	/>

	<!-- The two regions that speak for the session. Neither holds anything that changes on a
	     timer, so they announce on real changes only, and both are in the DOM from the first
	     render — a live region inserted at the same moment as its text is often missed. The
	     visible copies of this text below are `aria-hidden`, so nothing is announced twice. -->
	<p class="sr-only" role="status">{stateAnnouncement[$sessionState]}</p>
	<p class="sr-only" role="status">{statusText}</p>
	{#if failedDevice}
		<DeviceRecoveryBanner
			failed={failedDevice}
			busy={$sessionBusy || retryingDevice}
			onRetry={() => retryDevice(false)}
			onFallback={() => retryDevice(true)}
			onReselect={reselectApplication}
		/>
	{/if}

	<div class="rule" class:live={$isRunning}>
		{#if $isRunning}<span class="sweep"></span>{/if}
	</div>

	<SessionControls
		busy={$sessionBusy}
		startDisabled={!!languageError ||
			!$hasKey ||
			browserMode ||
			profileBusy ||
			$sessionBusy ||
			!preflight.applicationReady($options)}
		rehearseDisabled={!!languageError ||
			!$hasKey ||
			browserMode ||
			$sessionBusy ||
			$options.provider === 'ondevice'}
		onStart={start}
		onRehearse={rehearse}
		onStop={stop}
	/>

	<div class="body">
		<aside class="rail">
			{#if !$isRunning}
				<MeetingProfiles
					locked={controlsLocked || preflight.audioTesting || preflight.audioTestBusy}
					{overlay}
					onBusy={(value) => (profileBusy = value)}
					onLoaded={async () => {
						preflight.invalidateAudioTest();
						await preflight.refresh();
						await preflight.refreshLocalReadiness();
					}}
				/>
			{/if}
			{#if $isRunning}
				<LiveRail {overlay} {clock} {rehearsing} {usesMic} {usesSystem} />
			{:else}
				<SetupSheet
					{actions}
					{preflight}
					locked={controlsLocked}
					{browserMode}
					{usesMic}
					{usesSystem}
					{languageError}
				/>
			{/if}
		</aside>

		<main class="stage">
			{#if browserMode}
				<div class="banner">
					{$t.stage.browserBanner.before}
					<code>{$t.stage.browserBanner.command}</code>
					{$t.stage.browserBanner.after}
				</div>
			{/if}

			{#if $isRunning}
				<LiveTurns />

				{#if $statusMessage}
					<p class="status-msg" aria-hidden="true">{statusText}</p>
				{/if}

				<span class="grow"></span>

				<TranscriptMonitor
					mode={$options.mode}
					transcript={$transcript}
					onError={(message) => statusMessage.set(message)}
				/>
			{:else}
				<span class="kicker">{$t.preflight.kicker}</span>
				<h2 class="ready">{$t.preflight.heading}</h2>
				<p class="intro">{$t.preflight.intro}</p>

				<PreflightChecklist
					{preflight}
					{overlay}
					{browserMode}
					locked={controlsLocked}
					{usesMic}
					{usesSystem}
				/>

				<!-- Saved transcript follows the persistent session controls. -->
				{#if $transcript.length > 0}
					<TranscriptMonitor
						mode={$options.mode}
						transcript={$transcript}
						onError={(message) => statusMessage.set(message)}
					/>
				{/if}

				{#if $statusMessage}
					<p class="status-msg" aria-hidden="true">{statusText}</p>
				{/if}

				<div class="launch">
					<span class="rehearse-hint" id="rehearse-hint">
						{$options.provider === 'ondevice'
							? $t.preflight.rehearse.demoHint
							: $t.preflight.rehearse.hint}
					</span>
					<span class="privacy">
						{$historyEnabled
							? $t.history.stored
							: $recoveryEnabled
								? $t.preflight.privacy.spooled
								: $t.preflight.privacy.memoryOnly}
						{$options.provider === 'ondevice'
							? $t.preflight.privacy.demo
							: $t.preflight.privacy.cloud($t.provider.vendor[$options.provider])}
					</span>
				</div>
			{/if}
		</main>
	</div>
</div>

{#if settingsOpen}
	<SettingsDialog
		{overlay}
		{browserMode}
		bind:tab={settingsTab}
		onHideWindow={quit.hideWindow}
		onClose={() => (settingsOpen = false)}
	/>
{/if}

<!-- Both are modal on purpose: each is the last moment at which an event's record can still
     be kept, and each has to be answered before the log underneath it changes again. -->
{#if recovered}
	<RecoveryPrompt
		lines={recovered.snapshot.lines.length}
		savedAt={formatDateTime(recovered.snapshot.savedAt, $localeTag)}
		path={recovered.path}
		onRestore={() => void answerRecovery(true)}
		onDelete={() => void answerRecovery(false)}
	/>
{/if}

{#if quit.hidePrompt}
	<TrayHidePrompt running={$isRunning} onChoice={(choice) => void quit.onHideChoice(choice)} />
{/if}

{#if quit.sessionPrompt}
	<ActiveSessionPrompt
		elapsed={clock.elapsed}
		fromTray={quit.sessionPromptFromTray}
		onChoice={(stopIt) => void quit.onSessionChoice(stopIt)}
	/>
{/if}

{#if quit.closePrompt}
	<UnsavedPrompt
		lines={$transcript.length}
		endedSession={quit.closeEndedSession}
		saving={quit.closeSaving}
		error={quit.closeError}
		onChoice={(choice) => void quit.onCloseChoice(choice)}
	/>
{/if}

<style>
	.app {
		height: 100vh;
		display: grid;
		/* The title bar sizes to its own text rather than to a slot: at 225% its label is
		   28px tall and a fixed 40px row would crop it. */
		grid-template-rows: auto 2px auto minmax(0, 1fr);
		background: var(--surface-0);
		/* The query container for the column rule below. Its `em` is the scaled root, which
		   is what lets a text-size change move the breakpoint. */
		container: window / inline-size;
	}
	.app.device-error {
		grid-template-rows: auto auto 2px auto minmax(0, 1fr);
	}

	/* ---- Header ------------------------------------------------------------- */

	.rule {
		background: var(--hairline);
	}
	.rule.live {
		background: #163027;
		position: relative;
		overflow: hidden;
	}
	.sweep {
		position: absolute;
		inset: 0;
		background: linear-gradient(90deg, transparent, var(--accent), transparent);
		animation: sweep 2.6s linear infinite;
	}

	/* ---- Layout ------------------------------------------------------------- */

	/* One column is the safe shape, so it is the default: the rail above the stage, the whole
	   thing scrolling as a page. The two-column layout is an enhancement that only applies
	   while the window can hold 23.75em of rail and 30em of stage — measured in `em`, so the
	   threshold rises with the operator's text size instead of leaving a 225% rail squeezed
	   into 380 physical pixels. At 100% that is 860px, comfortably inside the 980px minimum
	   window, so nothing about the normal layout changes. */
	.body {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		min-height: 0;
		overflow-y: auto;
	}
	.rail {
		/* rem, not px: at 225% the type doubles, and a fixed 20px gap leaves a section
		   heading touching the paragraph above it. Same numbers at 100%. */
		padding: 1.375rem 1.375rem 1.625rem;
		display: flex;
		flex-direction: column;
		gap: 1.25rem;
		background: var(--panel);
		border-bottom: 1px solid var(--hairline);
	}
	/* The same measure the rail has as a column, so a stacked rail keeps the proportions the
	   cards were drawn at instead of stretching a two-line description across the window.
	   Never binds in the two-column layout, where the rail is exactly this wide. Global, so
	   that a child component's root element (MeetingProfiles) is capped too, and in rem, so
	   a child that sets a smaller font size is not capped narrower than its siblings. */
	.rail > :global(*) {
		max-width: 23.75rem;
	}
	.stage {
		padding: 1.875rem 2.375rem 2rem;
		display: flex;
		flex-direction: column;
	}
	@container window (min-width: 53.75em) {
		.body {
			grid-template-columns: 23.75em minmax(0, 1fr);
			overflow-y: hidden;
		}
		/* Side by side, each column carries its own scrollbar again, so the pre-flight
		   checklist and the transcript scroll independently. */
		.rail {
			border-bottom: 0;
			border-right: 1px solid var(--hairline);
			overflow-y: auto;
		}
		.stage {
			overflow-y: auto;
		}
	}
	/* Both columns scroll rather than compress: a flex column shrinks its children before the
	   scrollbar appears, which would clip text on a short window. */
	.kicker,
	.banner,
	.ready,
	.intro,
	.status-msg,
	.launch {
		flex: 0 0 auto;
	}

	/* ---- Stage -------------------------------------------------------------- */

	.banner {
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: var(--radius-card);
		padding: 12px;
		margin-bottom: 22px;
		font-size: var(--type-13);
		line-height: 1.5;
		color: var(--muted);
	}
	.banner code {
		font-family: var(--font-mono);
		font-size: var(--type-12);
		color: var(--text-dim);
	}
	.ready {
		margin: 16px 0 0;
		font-size: var(--type-27);
		font-weight: 600;
		line-height: 1.2;
		letter-spacing: -0.02em;
	}
	.intro {
		margin: 8px 0 0;
		font-size: var(--type-13-5);
		line-height: 1.55;
		color: var(--muted);
		max-width: 48ch;
		text-wrap: pretty;
	}

	/* Keep actions aligned and let supporting text use the available reading width. */
	.launch {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 12px;
		margin-top: 30px;
	}
	.rehearse-hint {
		font-size: var(--type-11-5);
		line-height: 1.45;
		color: var(--muted-3);
		max-width: 72ch;
		text-wrap: pretty;
	}

	.privacy {
		font-size: var(--type-12-5);
		line-height: 1.5;
		color: var(--muted-3);
		max-width: 72ch;
		text-wrap: pretty;
	}
	.status-msg {
		margin: 16px 0 0;
		font-size: var(--type-13);
		line-height: 1.5;
		color: var(--warn);
	}

	/* Near the window's minimum height, tighten the vertical rhythm so the pre-flight checklist
	   and the Start button still land above the fold. */
	@media (max-height: 740px) {
		.rail {
			padding: 1.125rem 1.375rem 1.25rem;
			gap: 1rem;
		}
		.stage {
			padding-top: 1.375rem;
			padding-bottom: 1.375rem;
		}
		.ready {
			margin-top: 10px;
			font-size: var(--type-24);
		}
		.launch {
			margin-top: 22px;
		}
	}

	@media (prefers-reduced-motion: reduce) {
		.sweep {
			animation: none;
		}
	}

	/* Windows contrast themes. A contrast theme replaces every colour this window chose, so
	   anything it says in colour alone has to be said again in a way the theme keeps.
	   Everything else is deliberately left to the system palette. */
</style>
