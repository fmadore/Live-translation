<script lang="ts">
	import { onMount } from 'svelte';
	import { t } from './i18n';
	import { isRunning, statusMessage } from './stores';
	import { api, on } from './tauri';
	import { asStatus } from './errors';
	import {
		noteWhisperProgress,
		whisperFinalizing as finalizing,
		whisperPendingSeconds as pending,
		whisperProgress
	} from './whisperProgress';
	import ToolButton from './ui/ToolButton.svelte';

	/** Whisper transcribes 10-second windows: `WINDOW` in `src-tauri/src/whisper/segment.rs`. */
	const WINDOW_SECONDS = 10;
	/** The most audio a session that keeps up still holds, in the whole seconds the strip
	 *  shows. A window counts as pending from its first sample until its transcription
	 *  completes (`Spool::complete` in `whisper/mod.rs`), and a cut can overshoot by one capture
	 *  chunk of up to a second: 11 s. While it is transcribed, the next window fills, and the
	 *  transcription has to finish within the 9 s of new audio that brings (a window keeps a
	 *  second of overlap) or the backlog grows. So up to 20 s, 21 once rounded up, is a session
	 *  keeping up; only beyond it is Whisper falling behind. */
	const KEEPING_UP_SECONDS = 2 * WINDOW_SECONDS + 1;

	// Discard asks a second time before throwing audio away, and the question goes as soon as
	// there is nothing left to discard. `$derived` is writable: the button sets it, and the
	// backlog starting or ending resets it.
	const backlogged = $derived($isRunning && $pending > 0);
	let confirmDiscard = $derived.by(() => {
		void backlogged;
		return false;
	});
	// The strip is for a backlog, not for the windows every session is filling and transcribing;
	// finishing is always shown, because Stop is waiting on it.
	const finishing = $derived($isRunning && $finalizing);
	// Once shown, the strip stays until the backlog is down to the window being filled, so a
	// backlog hovering at the threshold does not show and hide it every second, moving the
	// whole stage each time.
	const CAUGHT_UP_SECONDS = WINDOW_SECONDS + 1;
	let wasBehind = false;
	const behind = $derived.by(() => {
		wasBehind =
			$isRunning && (wasBehind ? $pending > CAUGHT_UP_SECONDS : $pending > KEEPING_UP_SECONDS);
		return wasBehind;
	});
	const shown = $derived(finishing || behind);
	// Progress arrives about once a second, so a live region around the seconds would speak for
	// the whole session. Narrator hears the changes of state instead: finishing starting, and a
	// discard being confirmed. `$derived` is writable again: Discard sets it, and finishing
	// starting or ending resets it.
	let discarding = $derived.by(() => {
		void finishing;
		return false;
	});
	const announcement = $derived(
		discarding ? $t.whisper.discarding : finishing ? $t.whisper.processing : ''
	);
	// The backlog is shared with the quit prompt, so a finished run's figure is cleared rather
	// than only hidden — again if a late report lands after the run has ended.
	$effect(() => {
		void $pending;
		if (!$isRunning) whisperProgress.set({});
	});
	onMount(() => {
		let disposed = false;
		let cleanup: (() => void) | undefined;
		void on.whisperProgress(noteWhisperProgress).then((fn) => {
			if (disposed) fn();
			else cleanup = fn;
		});
		return () => {
			disposed = true;
			cleanup?.();
		};
	});
	async function discard() {
		confirmDiscard = false;
		discarding = true;
		try {
			await api.discardWhisperPending();
		} catch (error) {
			discarding = false;
			statusMessage.set(asStatus(error));
		}
	}
</script>

{#if shown}
	<div class="local-progress">
		<span>{$finalizing ? $t.whisper.processing : $t.whisper.pending} · {$pending} s</span>
		{#if $finalizing && $pending > 0}
			<ToolButton onclick={() => (confirmDiscard = true)}>{$t.whisper.discard}</ToolButton>
		{/if}
	</div>
{/if}
<p class="sr-only" role="status">{announcement}</p>
{#if confirmDiscard}
	<div class="discard-confirm" role="alert">
		<p>{$t.whisper.discardConfirm}</p>
		<ToolButton onclick={discard}>{$t.whisper.discard}</ToolButton>
		<ToolButton onclick={() => (confirmDiscard = false)}>{$t.history.cancel}</ToolButton>
	</div>
{/if}

<style>
	.local-progress,
	.discard-confirm {
		padding: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		font-size: var(--type-small);
	}
	.local-progress {
		display: flex;
		gap: var(--space-3);
		align-items: center;
		justify-content: space-between;
	}
</style>
