<script lang="ts">
	import { exportOriginal, inFlightLines } from './stores';
	import { hasOriginalSpeech } from './transcript';
	import type { AppError } from './errors';
	import type { OutputMode, TranscriptLine } from './types';
	import TranscriptActions from './TranscriptActions.svelte';
	import TranscriptLog from './TranscriptLog.svelte';
	import RecoveryPreference from './RecoveryPreference.svelte';

	/** The transcript panel: its toolbar and notes (`TranscriptActions`), the log
	 *  (`TranscriptLog`), and the crash-recovery opt-in.
	 *
	 *  The page mounts it once and keeps it through Start and Stop, so the copy window, whether
	 *  the log is following, and a save under way all carry over from one state to the other
	 *  (review S5); it used to be mounted once per state, and each swap reset them. */
	let {
		mode,
		transcript,
		onError,
		live = false,
		reader = false
	}: {
		mode: OutputMode;
		transcript: TranscriptLine[];
		onError: (message: string | AppError) => void;
		/** A session is running, so turns still being spoken belong in what Copy takes. */
		live?: boolean;
		/** The stage's transcript reader (issue #124): the log fills the stage instead of six
		 *  rows under the live turns, shows the turns still being spoken at its foot, and the
		 *  original speech under each paragraph when it is set to be exported. */
		reader?: boolean;
	} = $props();

	// Turns not yet final. Only a running session has any: after Stop they are committed.
	const pending = $derived(live ? $inFlightLines : []);
	// The reader shows what Save and Copy will write, so the original speech appears under
	// each paragraph exactly when it would be exported.
	const showOriginal = $derived(reader && hasOriginalSpeech(transcript) && $exportOriginal);
</script>

<section class="monitor" class:reader>
	<TranscriptActions {transcript} {pending} {onError} />
	<TranscriptLog {mode} {transcript} pending={reader ? pending : []} {reader} {showOriginal} />
	<RecoveryPreference {onError} />
</section>

<style>
	.monitor {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		margin-top: var(--space-5);
		padding-top: var(--space-4);
		border-top: 1px solid var(--line);
		/* The stage is a scrolling flex column; keep the block at its natural height there. */
		flex: 0 0 auto;
	}

	/* The stage's whole height, its own scrollbar for the log, and the rest of the block at
	   its natural height around it. No rule above: the tabs already divide it from the bar. */
	.monitor.reader {
		flex: 1 1 auto;
		min-height: 0;
		margin-top: var(--space-4);
		padding-top: 0;
		border-top: 0;
	}
</style>
