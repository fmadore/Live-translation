<script lang="ts">
	import WhisperActivity from '#lib/WhisperActivity.svelte';
	import { whisperModels } from '#lib/stores.js';
	import { whisperPendingSeconds } from '#lib/whisperProgress.js';
	import { isLocalWhisper } from '#lib/providers.js';
	import { onDestroy, onMount } from 'svelte';
	import MeetingProfiles from '#lib/MeetingProfiles.svelte';
	import OperatorToolbar from '#lib/OperatorToolbar.svelte';
	import SessionControls from '#lib/SessionControls.svelte';
	import DeviceRecoveryBanner from '#lib/DeviceRecoveryBanner.svelte';
	import RunningStage from '#lib/RunningStage.svelte';
	import IdleStage from '#lib/IdleStage.svelte';
	import LaunchNotes from '#lib/LaunchNotes.svelte';
	import AppCredit from '#lib/AppCredit.svelte';
	import LiveRail from '#lib/LiveRail.svelte';
	import SetupSheet from '#lib/SetupSheet.svelte';
	import TranscriptMonitor from '#lib/TranscriptMonitor.svelte';
	import SettingsDialog, { type SettingsTab } from '#lib/SettingsDialog.svelte';
	import ActiveSessionPrompt from '#lib/ActiveSessionPrompt.svelte';
	import RecoveryPrompt from '#lib/RecoveryPrompt.svelte';
	import TrayHidePrompt from '#lib/TrayHidePrompt.svelte';
	import UnsavedPrompt from '#lib/UnsavedPrompt.svelte';
	import ZoomCaptionsDialog from '#lib/ZoomCaptionsDialog.svelte';
	import { createPreflightController } from '#lib/preflightController.svelte.js';
	import { createQuitController } from '#lib/quitController.svelte.js';
	import { createOverlayController } from '#lib/overlayController.svelte.js';
	import { createSessionClock } from '#lib/sessionClock.svelte.js';
	import { createSessionController } from '#lib/sessionController.js';
	import { createSetupActions } from '#lib/setupActions.js';
	import { createDeviceFailure } from '#lib/deviceFailure.svelte.js';
	import { createRecoveryOffer } from '#lib/recoveryOffer.svelte.js';
	import { createZoomCaptions } from '#lib/zoomCaptions.svelte.js';
	import { syncNative } from '#lib/nativeSync.svelte.js';
	import { provideOperatorContext } from '#lib/operatorContext.js';
	import { createLaunchController } from '#lib/launchController.svelte.js';
	import { shortcut } from '#lib/shortcuts.js';
	import { on, isTauri } from '#lib/tauri.js';
	import { describeError } from '#lib/errors.js';
	import {
		sessionState,
		isRunning,
		statusMessage,
		applyStatus,
		options,
		transcript,
		overlayFontSize,
		noteActivity,
		describeStatus,
		pauseRequested,
		pushCaption,
		stageView
	} from '#lib/stores.js';
	import Tabs from '#lib/ui/Tabs.svelte';
	import { followTextScale } from '#lib/textScale.js';
	import { followTheme } from '#lib/theme.js';
	import type { SessionState } from '#lib/types.js';
	import { formatDateTime, localeTag, locale, t } from '#lib/i18n/index.js';

	// Resolved at component init, not in `onMount`. `isTauri()` is a synchronous property
	// check and this app never server-renders (`ssr = false` in +layout.ts), so the answer is
	// available on the very first render — which is what keeps a child like ApiKeyPanel from
	// mounting and firing a Tauri-only command during `npm run dev`.
	const browserMode = !isTauri();
	// Light, dark or Windows' app mode. `app.html` sets it before the first frame; this keeps it
	// in step with the setting and with Windows from here on. The overlay keeps the dark tokens.
	onDestroy(followTheme());
	const session = createSessionController();
	const overlay = createOverlayController();
	const quit = createQuitController({
		stop: () => session.stop(),
		toggleOverlayVisible: () => overlay.toggleOverlayVisible()
	});
	const sessionBusy = session.busy;
	const sessionStarting = session.starting;
	let profileBusy = $state(false);
	const controlsLocked = $derived($isRunning || $sessionBusy || profileBusy);
	// Test audio follows `controlsLocked`, as its button does. A capture failure holds only the
	// automatic device fallback, which would otherwise turn the banner's Retry into a fallback.
	const preflight = createPreflightController(!browserMode, {
		locked: () => controlsLocked,
		holdSelection: () => device.failed !== null || device.retrying
	});
	const device = createDeviceFailure({
		stop: () => session.stop(),
		start: (selected) => session.start(selected),
		refreshApplications: () => preflight.refreshApplications()
	});
	// The components under the page read these two from context rather than through props.
	provideOperatorContext({ overlay, preflight });
	$effect(() => {
		if (!controlsLocked && !preflight.audioTesting && !preflight.audioTestBusy)
			preflight.validateSelection();
	});

	// Whether the selected engine could start: a key the panel has confirmed for this engine,
	// an installed Whisper model on a processor that runs it, or the demonstration's readiness.
	// Derived rather than mirrored into a store, so nothing can leave it holding the previous
	// engine's answer.
	const engineReady = $derived(preflight.engineReady($options, $whisperModels));

	// Ticks only while a session is open.
	const clock = createSessionClock();

	// Captions in Zoom: independent of the session, so it lives here rather than under it, and
	// the caption link it holds survives the dialog closing (but not the window reloading).
	const zoom = createZoomCaptions();
	let zoomOpen = $state(false);

	// Start, Rehearse and the start shortcut: what blocks them, and the launch they share.
	// Created before the meter gates below, which depend on whether a rehearsal is running.
	const launch = createLaunchController({
		browserMode,
		start: (selected) => session.start(selected),
		sessionBusy,
		profileBusy: () => profileBusy,
		engineReady: () => engineReady,
		applicationReady: (selected) => preflight.applicationReady(selected),
		launching: () => device.clear()
	});
	const rehearsalRunning = $derived(launch.rehearsalRunning);

	// A rehearsal is a single System-origin stream no matter what source is saved, so the
	// meters follow the fixture rather than showing a dead Room meter. Idle is unaffected
	// (`rehearsalRunning` is false there, so the pre-flight audio check keeps its semantics).
	const usesMic = $derived(!rehearsalRunning && $options.source !== 'system');
	const usesSystem = $derived(rehearsalRunning || $options.source !== 'microphone');

	onMount(() => {
		if (browserMode) return;

		void preflight.refresh();
		void preflight.refreshLocalReadiness();
		void preflight.refreshWhisperCpu();
		void recoveryOffer.load();
		overlay.initialize();
		// Where the overlay was last placed on these displays. It paints nothing until there are
		// captions or move mode, so moving it now shows no jump from where it was created.
		void overlay.restorePlacement();

		// Sharing with Zoom may have outlived the last load of this window. Asked once the
		// listener is in place, so nothing the core reports in between is missed.
		const zoomListening = on.zoomCaptions(zoom.apply);
		void zoomListening.then(() => zoom.resync());

		const unlisteners: Array<Promise<() => void>> = [
			zoomListening,
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
				applyStatus(s, { starting: $sessionStarting });
				device.noteStatus(s);
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
			on.overlayState(overlay.applyState),
			// A projector unplugged or plugged back in mid-event: put the overlay where this
			// layout last had it, once the core has made sure it is on a display at all.
			on.displaysChanged((layout) => void overlay.handleDisplaysChanged(layout))
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
	const recoveryOffer = createRecoveryOffer();

	// The close guard, tray state, recovery spool and the overlay's languages follow this
	// window's state.
	syncNative({ desktop: !browserMode, overlay, clock });

	const stop = () => session.stop();
	const togglePause = () => void session.pause(!$pauseRequested);

	const actions = createSetupActions({
		locked: () => controlsLocked,
		invalidateAudioTest: () => preflight.invalidateAudioTest(),
		refreshDevices: () => void preflight.refresh(),
		whisperRefused: () => preflight.whisperRefused
	});

	// Whisper is the first-launch engine, but a processor without the instruction sets it was
	// built for cannot run it. Whichever way it got selected — a fresh install, saved options, a
	// profile — the keyless demo takes over, so the app always opens on something that starts.
	$effect(() => {
		if (preflight.whisperRefused && $options.provider === 'whisper' && !controlsLocked)
			actions.avoidUnsupportedWhisper();
	});

	let settingsOpen = $state(false);
	let settingsTab = $state<SettingsTab>('captions');

	// The status line: plain text as it stands, a core failure as the sentence for its id plus
	// the technical detail, a message of the window's own as the catalog words it. Derived
	// rather than stored, so switching language re-words a message that is already on screen.
	const statusText = $derived($statusMessage ? describeStatus($statusMessage, $t, $locale) : '');
	// The quit prompt's failed save, likewise.
	const closeErrorText = $derived(quit.closeError ? describeError(quit.closeError, $t) : '');

	// ---- Display labels ---------------------------------------------------------
	//
	// Every word below comes from the catalog, so a change of interface language re-renders
	// the rail and the stage without touching the session.

	// What a screen reader hears when the session changes state. Deliberately separate from the
	// pill: the pill carries a clock that reprints every second, and a live region wrapped
	// around a ticking clock announces the whole session state every second with it.
	// The running stage is reading the whole transcript rather than the turns on screen.
	const reading = $derived($isRunning && $stageView === 'transcript');

	const stateAnnouncement = $derived<Record<SessionState, string>>($t.announce);
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
		if (command === 'togglePause' && !browserMode && $isRunning && !$sessionBusy) togglePause();
		if (command === 'toggleSession' && !browserMode && !profileBusy && !$sessionBusy) {
			// Start answers to the same blockers as its button; see `LaunchController.launch`.
			if ($isRunning) void stop();
			else void launch.start();
		}
	}}
/>

<div class="app" class:device-error={device.failed !== null}>
	<OperatorToolbar
		elapsed={clock.elapsed}
		{settingsOpen}
		onOpenSettings={() => (settingsOpen = true)}
		zoomState={zoom.status.state}
		{zoomOpen}
		onOpenZoom={() => (zoomOpen = true)}
	>
		{#snippet actions()}
			<SessionControls
				busy={$sessionBusy || launch.opening}
				startDisabled={!launch.canStart}
				rehearseDisabled={!launch.canRehearse}
				paused={$pauseRequested}
				onStart={launch.start}
				onRehearse={launch.rehearse}
				onStop={stop}
				onPause={togglePause}
			/>
		{/snippet}
	</OperatorToolbar>

	<div class="rule" class:live={$isRunning}>
		{#if $isRunning}<span class="sweep"></span>{/if}
	</div>

	<!-- The two regions that speak for the session. Neither holds anything that changes on a
	     timer, so they announce on real changes only, and both are in the DOM from the first
	     render — a live region inserted at the same moment as its text is often missed. The
	     visible copies of this text below are `aria-hidden`, so nothing is announced twice. -->
	<p class="sr-only" role="status">{stateAnnouncement[$sessionState]}</p>
	<p class="sr-only" role="status">{statusText}</p>
	{#if device.failed}
		<DeviceRecoveryBanner
			failed={device.failed}
			busy={$sessionBusy || device.retrying}
			onRetry={() => device.retry(false)}
			onFallback={() => device.retry(true)}
			onReselect={device.reselectApplication}
		/>
	{/if}

	<div class="body">
		<aside class="rail">
			{#if !$isRunning}
				<MeetingProfiles
					locked={controlsLocked || preflight.audioTesting || preflight.audioTestBusy}
					onBusy={(value) => (profileBusy = value)}
					onLoaded={async () => {
						preflight.invalidateAudioTest();
						await preflight.refresh();
						await preflight.refreshLocalReadiness();
					}}
				/>
			{/if}
			{#if $isRunning}
				<LiveRail {clock} rehearsing={rehearsalRunning} {usesMic} {usesSystem} />
			{:else}
				<SetupSheet
					{actions}
					locked={controlsLocked}
					{browserMode}
					{usesMic}
					{usesSystem}
					languageError={launch.languageError}
				/>
			{/if}
		</aside>

		<main class="stage">
			{#if isLocalWhisper($options.provider)}<WhisperActivity />{/if}
			{#if browserMode}
				<div class="banner">
					{$t.stage.browserBanner.before}
					<code>{$t.stage.browserBanner.command}</code>
					{$t.stage.browserBanner.after}
				</div>
			{/if}

			{#if $isRunning}
				<!-- Issue #124: the turns on screen, or the whole transcript to read back and copy
				     from while the meeting goes on. -->
				<Tabs
					tabs={[
						{ id: 'live', label: $t.stage.view.live },
						{ id: 'transcript', label: $t.stage.view.transcript }
					]}
					bind:selected={$stageView}
					label={$t.stage.view.label}
					idPrefix="stage-view"
					panelId="stage-panel"
				/>
			{/if}
			<!-- One column for both states, so the transcript below is mounted once and kept through
			     Start and Stop (review S5): its copy window, whether it is following, and a save
			     under way all carry over. While a session runs it is the view tabs' panel. -->
			<div
				class="stage-panel"
				class:running={$isRunning}
				class:reading
				id="stage-panel"
				role={$isRunning ? 'tabpanel' : undefined}
				aria-labelledby={$isRunning ? `stage-view-${$stageView}` : undefined}
			>
				{#if $isRunning}
					<RunningStage {statusText} />
				{:else}
					<IdleStage {browserMode} locked={controlsLocked} {usesMic} {usesSystem} />
				{/if}

				<!-- Idle, the saved transcript follows the pre-flight check, once there is one. -->
				{#if $isRunning || $transcript.length > 0}
					<TranscriptMonitor
						mode={$options.mode}
						transcript={$transcript}
						onError={(message) => statusMessage.set(message)}
						live={$isRunning}
						reader={reading}
					/>
				{/if}

				{#if !$isRunning}
					<LaunchNotes {statusText} />
					<AppCredit />
				{/if}
			</div>
		</main>
	</div>
</div>

{#if settingsOpen}
	<SettingsDialog
		{browserMode}
		bind:tab={settingsTab}
		onHideWindow={quit.hideWindow}
		onClose={() => (settingsOpen = false)}
	/>
{/if}

{#if zoomOpen}
	<ZoomCaptionsDialog
		{zoom}
		onClose={() => (zoomOpen = false)}
		showOnScreen={overlay.overlayVisible}
		onShowOnScreen={(show) => {
			if (show !== overlay.overlayVisible) void overlay.toggleOverlayVisible();
		}}
	/>
{/if}

<!-- Both are modal on purpose: each is the last moment at which an event's record can still
     be kept, and each has to be answered before the log underneath it changes again. -->
{#if recoveryOffer.offer}
	<RecoveryPrompt
		lines={recoveryOffer.offer.snapshot.lines.length}
		savedAt={formatDateTime(recoveryOffer.offer.snapshot.savedAt, $localeTag)}
		path={recoveryOffer.offer.path}
		onRestore={() => void recoveryOffer.answer(true)}
		onDelete={() => void recoveryOffer.answer(false)}
	/>
{/if}

{#if quit.hidePrompt}
	<TrayHidePrompt running={$isRunning} onChoice={(choice) => void quit.onHideChoice(choice)} />
{/if}

{#if quit.sessionPrompt}
	<ActiveSessionPrompt
		elapsed={clock.elapsed}
		fromTray={quit.sessionPromptFromTray}
		pendingSeconds={isLocalWhisper($options.provider) ? $whisperPendingSeconds : 0}
		onChoice={(stopIt) => void quit.onSessionChoice(stopIt)}
	/>
{/if}

{#if quit.closePrompt}
	<UnsavedPrompt
		lines={$transcript.length}
		endedSession={quit.closeEndedSession}
		answering={quit.closeAnswering}
		error={closeErrorText}
		onChoice={(choice) => void quit.onCloseChoice(choice)}
	/>
{/if}

<style>
	.app {
		height: 100vh;
		display: grid;
		/* The bar sizes to its own content rather than to a slot: at 225% its buttons are twice
		   as tall, and a narrow window wraps it onto a second line. */
		grid-template-rows: auto 2px minmax(0, 1fr);
		background: var(--surface-0);
		/* The query container for the column rule below. Its `em` is the scaled root, which
		   is what lets a text-size change move the breakpoint. */
		container: window / inline-size;
	}
	.app.device-error {
		grid-template-rows: auto 2px auto minmax(0, 1fr);
	}

	/* ---- Header ------------------------------------------------------------- */

	.rule {
		background: var(--line);
	}
	.rule.live {
		background: var(--accent-chip);
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
		padding: var(--space-5) var(--space-5) var(--space-5);
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		background: var(--surface-1);
		border-bottom: 1px solid var(--line);
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
		padding: var(--space-6) var(--space-6) var(--space-6);
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
			border-right: 1px solid var(--line);
			overflow-y: auto;
		}
		.stage {
			overflow-y: auto;
		}
	}
	/* Both columns scroll rather than compress: a flex column shrinks its children before the
	   scrollbar appears, which would clip text on a short window. */
	.banner {
		flex: 0 0 auto;
	}

	/* ---- Stage -------------------------------------------------------------- */

	/* The stage's column, idle or running; running, it sits under its tabs. Idle or live, it is
	   the scrolling column it always was: its blocks keep their height and the transcript sits
	   at the foot. Reading, it gives the transcript the stage's height, which the reader passes
	   on to its log. */
	.stage-panel {
		display: flex;
		flex-direction: column;
		flex: 1 0 auto;
	}
	.stage-panel.running {
		margin-top: var(--space-4);
	}
	.stage-panel.reading {
		flex: 1 1 auto;
		min-height: 0;
	}

	.banner {
		background: var(--surface-1);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-card);
		padding: var(--space-3);
		margin-bottom: var(--space-5);
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--text-muted);
	}
	.banner code {
		font-family: var(--font-mono);
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	/* Near the window's minimum height, tighten the vertical rhythm so the pre-flight checklist
	   and the Start button still land above the fold. */
	@media (max-height: 740px) {
		.rail {
			padding: var(--space-4) var(--space-5) var(--space-5);
			gap: var(--space-4);
		}
		.stage {
			padding-top: var(--space-5);
			padding-bottom: var(--space-5);
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
