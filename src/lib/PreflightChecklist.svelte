<script lang="ts">
	import ApiKeyPanel from './ApiKeyPanel.svelte';
	import OverlayMoveButton from './OverlayMoveButton.svelte';
	import ChecklistRow from './ui/ChecklistRow.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import { t } from './i18n';
	import { tick } from 'svelte';
	import { options, overlayPlaced, statusMessage, whisperModels } from './stores';
	import { PROVIDER_META, isLocalWhisper, providerRequiresKey, rateText } from './providers';
	import { describeReadiness, streamCount } from './types';
	import { preflightContext } from './operatorContext';
	import {
		cancelWhisperDownload,
		downloadWhisperModel,
		watchWhisperModels,
		whisperDownloading,
		whisperModelBusy
	} from './whisperDownload';

	let {
		browserMode,
		locked,
		usesMic,
		usesSystem
	}: {
		browserMode: boolean;
		/** Setup is locked: a session, a start or a profile load is under way. */
		locked: boolean;
		usesMic: boolean;
		usesSystem: boolean;
	} = $props();
	const preflight = preflightContext();

	const needsKey = $derived(providerRequiresKey($options.provider));
	const localWhisper = $derived(isLocalWhisper($options.provider));
	const whisperTitle = $derived(
		$options.provider === 'whisper-translate' ? $t.whisper.translateTitle : $t.whisper.title
	);

	// A first run opens on Whisper with no model, and the model step is below the fold: the row
	// that says a model is missing offers the download itself, through the same controller as
	// the model step (`whisperDownload.ts`), so a download started here shows there too.
	const whisperModelId = $derived($options.whisperModel ?? 'base');
	const whisperModel = $derived($whisperModels.find((m) => m.id === whisperModelId));
	const whisperInstalled = $derived(!!whisperModel?.installed);
	const whisperBusy = $derived($whisperModelBusy || $whisperDownloading);
	const whisperDescId = $props.id();
	let whisperAction = $state<HTMLElement | null>(null);
	$effect(() => (localWhisper && !browserMode ? watchWhisperModels() : undefined));

	/** Where the keyboard goes when the row's button is replaced under it: the first enabled
	 *  button left in the row, or else the row's sentence. Cancel is disabled until the core
	 *  reports the download, so a start lands on the sentence, which then says it is
	 *  downloading; a cancel or a failure brings Download back. Only when focus was on the
	 *  row's own controls, and only when it has fallen to the page. */
	async function handOff(had: boolean) {
		if (!had) return;
		await tick();
		const lost = !document.activeElement || document.activeElement === document.body;
		if (!lost) return;
		const target =
			whisperAction?.querySelector<HTMLElement>('button:not(:disabled)') ??
			document.getElementById(whisperDescId);
		if (target && !target.matches('button')) target.tabIndex = -1;
		target?.focus();
	}

	async function downloadFromRow() {
		if (locked || browserMode || whisperBusy) return;
		const had = !!whisperAction?.contains(document.activeElement);
		const done = downloadWhisperModel(whisperModelId);
		await handOff(had);
		await done;
		await handOff(had);
	}

	// The demo row's sentence. The core names the readiness state and the catalog words it, so
	// this re-words itself when the interface language changes rather than freezing whatever
	// was true at check time.
	const demoRowText = $derived(describeReadiness(preflight.localReadiness, $t));
	const costTags = $derived(streamCount(usesMic, usesSystem, $options).tags);

	// The tick means "this source has been heard", not "we are listening now". The old check
	// asked the second question from a screen where the answer was always no, because levels
	// only flow once capture is running.
	const audioVerified = $derived(
		$options.provider === 'ondevice' ||
			((!usesMic || preflight.micVerified) && (!usesSystem || preflight.systemVerified))
	);
	// Live view for the duration of a test.
	const audioHearing = $derived(
		(!usesMic || preflight.micSignal) && (!usesSystem || preflight.systemSignal)
	);

	// Which of the four things is under test: the room mic, system loopback, both, or the
	// bundled sample. One key, used for the row title and both tenses of the description.
	const audioSubject = $derived<'microphone' | 'system' | 'both' | 'demo'>(
		$options.provider === 'ondevice'
			? 'demo'
			: $options.source === 'system'
				? 'system'
				: $options.source === 'microphone'
					? 'microphone'
					: 'both'
	);
	// System capture set to one application with none usable blocks Start and the audio test
	// alike. The row is where that reads, since a disabled button cannot say why: never chosen
	// asks for one, chosen but since closed asks for it again. Not before the first list of
	// applications has arrived, so a stored choice does not flash as missing at launch.
	const applicationMissing = $derived(
		preflight.applicationCaptureSupported !== null && !preflight.applicationReady($options)
	);
	const audioCheckDesc = $derived(
		$options.provider === 'ondevice'
			? $t.preflight.audio.heard.demo
			: applicationMissing
				? $options.systemCapture?.kind === 'application' && $options.systemCapture.process
					? $t.applications.missing
					: $t.preflight.audio.chooseApplication
				: preflight.audioTesting
					? audioHearing
						? $t.preflight.audio.hearing[audioSubject]
						: preflight.systemSilenced
							? $t.preflight.audio.silenced
							: $t.preflight.audio.listening
					: audioVerified
						? $t.preflight.audio.heard[audioSubject]
						: $t.preflight.audio.unchecked
	);
</script>

<div class="checklist">
	{#if localWhisper && preflight.whisperRefused}
		<!-- Subtitles are normally replaced by the demo at once, and shown here only while setup
		     is locked and cannot switch. Translation stays, since the demo cannot translate:
		     the operator chooses a cloud engine instead. -->
		<ChecklistRow
			status="wait"
			title={whisperTitle}
			desc={$t.whisper.cpuUnsupported(preflight.whisperCpuMissing)}
			warn
		/>
	{:else if localWhisper}
		<!-- Not a live region: the model step's status says missing, downloading and ready
		     once each, and a second region here would say each twice. -->
		<ChecklistRow
			status={whisperInstalled ? 'ok' : 'wait'}
			title={whisperTitle}
			descId={whisperDescId}
			desc={whisperInstalled
				? $t.whisper.ready
				: whisperModel?.downloading
					? $t.whisper.downloading
					: $t.whisper.missing}
			warn={!whisperInstalled && !whisperModel?.downloading}
		>
			{#snippet action()}
				<div class="whisper-action" bind:this={whisperAction}>
					{#if whisperInstalled}
						<!-- Nothing left to do here; removing a model is the model step's. -->
					{:else if whisperBusy}
						{#if whisperModel?.downloading}
							<progress
								value={whisperModel.downloadedBytes}
								max={whisperModel.bytes}
								aria-label={$t.whisper.downloading}
								aria-valuetext="{Math.floor(
									(whisperModel.downloadedBytes / whisperModel.bytes) * 100
								)}%"
							></progress>
						{/if}
						<ToolButton
							variant="ghost"
							size="sm"
							disabled={locked || !$whisperDownloading}
							onclick={() => void cancelWhisperDownload()}
						>
							{$t.whisper.cancel}
						</ToolButton>
					{:else}
						<ToolButton
							variant="warn"
							size="sm"
							disabled={locked || browserMode}
							aria-describedby={whisperDescId}
							onclick={downloadFromRow}
						>
							{whisperModel
								? $t.whisper.downloadSized(
										$t.whisper.names[whisperModelId],
										Math.ceil(whisperModel.bytes / 1048576)
									)
								: $t.whisper.download}
						</ToolButton>
					{/if}
				</div>
			{/snippet}
		</ChecklistRow>
	{:else if !needsKey}
		<ChecklistRow
			status={preflight.localReadiness?.ready ? 'ok' : 'wait'}
			title={$t.preflight.demoRow.title}
			desc={demoRowText}
			warn={!preflight.localReadiness?.ready}
		/>
	{:else if !browserMode}
		<ApiKeyPanel
			provider={$options.provider}
			{locked}
			onAvailability={(provider, available) => {
				if ($options.provider === provider) preflight.noteKey(provider, available);
			}}
			onError={(message) => statusMessage.set(message)}
		/>
	{/if}

	<ChecklistRow
		status={audioVerified && !applicationMissing ? 'ok' : 'wait'}
		title={$t.preflight.audio.title[audioSubject]}
		desc={audioCheckDesc}
		warn={applicationMissing || (preflight.audioTesting && !audioHearing)}
	>
		{#snippet action()}
			<!-- The demo opens no device, so there is nothing to test. Otherwise the row keeps
			     the button in both states: re-checking after moving a cable or switching the
			     room mic is exactly when an operator needs it. -->
			{#if $options.provider === 'ondevice' || browserMode}
				<span></span>
			{:else if preflight.audioTesting}
				<ToolButton
					variant="ghost"
					size="sm"
					disabled={preflight.audioTestBusy}
					aria-busy={preflight.audioTestBusy}
					onclick={preflight.stopAudioTest}
				>
					{$t.preflight.audio.stopTest}
				</ToolButton>
			{:else}
				<ToolButton
					variant="warn"
					size="sm"
					disabled={preflight.audioTestBusy || locked || !preflight.applicationReady($options)}
					aria-busy={preflight.audioTestBusy}
					onclick={preflight.startAudioTest}
				>
					{audioVerified ? $t.preflight.audio.retest : $t.preflight.audio.test}
				</ToolButton>
			{/if}
		{/snippet}
	</ChecklistRow>

	<ChecklistRow
		status={$overlayPlaced ? 'ok' : 'wait'}
		title={$t.preflight.overlay.title}
		desc={$overlayPlaced ? $t.preflight.overlay.placed : $t.preflight.overlay.unplaced}
		warn={!$overlayPlaced}
	>
		{#snippet action()}
			<!-- Placement is never final: re-entering move mode is the way to adjust position and
			     caption size, so the row keeps a button in both states. -->
			<OverlayMoveButton
				wording="place"
				size="sm"
				icon={false}
				idleVariant={$overlayPlaced ? 'ghost' : 'warn'}
			/>
		{/snippet}
	</ChecklistRow>

	<ChecklistRow
		status="neutral"
		glyph="$"
		title={$t.preflight.cost.title}
		desc={localWhisper
			? $t.whisper.cost
			: !needsKey
				? $t.preflight.cost.free
				: $t.preflight.cost.billed}
	>
		{#snippet action()}
			<span class="check-rate">{rateText(PROVIDER_META[$options.provider], $t)}</span>
			<!-- The rate is per stream: a second source and a second caption language are each a
			     second stream, counted as the running rail counts them. -->
			{#each costTags as tag (tag)}<span class="check-tag">{$t.cost[tag]}</span>{/each}
		{/snippet}
	</ChecklistRow>
</div>

<style>
	.checklist {
		flex: 0 0 auto;
		margin-top: var(--space-5);
		border-top: 1px solid var(--line);
	}
	.whisper-action {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: flex-end;
		gap: var(--space-2);
	}
	.whisper-action progress {
		width: 6em;
		accent-color: var(--accent);
	}
	.check-tag {
		font-size: var(--type-small);
		color: var(--text-muted);
		white-space: nowrap;
	}
	.check-rate {
		font-family: var(--font-mono);
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1;
		color: var(--text-secondary);
		font-variant-numeric: tabular-nums;
	}
	/* Near the window's minimum height, tighten the vertical rhythm so the checklist and the
	   launch notes still land above the fold. */
	@media (max-height: 740px) {
		.checklist {
			margin-top: var(--space-4);
		}
	}
</style>
