<script lang="ts">
	// Shown at launch when a recovery spool from a previous run is still on disk — the app
	// went down before its transcript was saved. Answered before anything else, because
	// restoring replaces the log and a session started first would be overwritten.

	import ModalPrompt from './ModalPrompt.svelte';

	interface Props {
		lines: number;
		/** When the spool was last written, already formatted for display. */
		savedAt: string;
		/** Full path, so the operator knows exactly what "Delete" removes. */
		path: string;
		onRestore: () => void;
		onDelete: () => void;
	}

	let { lines, savedAt, path, onRestore, onDelete }: Props = $props();

	let restoreEl = $state<HTMLButtonElement | null>(null);

	$effect(() => {
		restoreEl?.focus();
	});
</script>

<!-- No Escape dismissal: leaving it unanswered would leave captions on disk that the operator
     never chose to keep, which is the one outcome this feature must not produce. -->
<ModalPrompt title="Recover the transcript from your last session?">
	<p>
		The app closed with {lines === 1 ? '1 unsaved line' : `${lines} unsaved lines`} still in the
		log. They were spooled locally at {savedAt} and have not left this PC.
	</p>

	<div class="actions">
		<button class="primary" bind:this={restoreEl} onclick={onRestore}>Restore the transcript</button>
		<button class="danger" onclick={onDelete}>Delete it</button>
	</div>
	<p class="note">Either answer removes the spool file at <code>{path}</code>. Restoring loads
		the lines back into the log, still unsaved, so you can save them where you want them.</p>
</ModalPrompt>
