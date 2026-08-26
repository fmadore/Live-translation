<script lang="ts">
	// The last thing between a live session's transcript and nothing at all. Shown when the
	// operator closes the window with finalized captions that never reached disk (issue #25).

	import ModalPrompt from './ModalPrompt.svelte';
	import type { CloseChoice } from './document';

	interface Props {
		/** Finalized lines that would be lost. */
		lines: number;
		/** True when the close arrived mid-session and capture has just been drained, so the
		 *  count can be explained rather than appearing from nowhere. */
		endedSession: boolean;
		/** Set while the Markdown file is being written, so Save cannot be pressed twice. */
		saving: boolean;
		/** Last save failure — kept on screen rather than quitting on a write that did not land. */
		error: string;
		onChoice: (choice: CloseChoice) => void;
	}

	let { lines, endedSession, saving, error, onChoice }: Props = $props();

	let saveEl = $state<HTMLButtonElement | null>(null);

	// Save is the recoverable answer, so it takes focus: Enter keeps the transcript, and the
	// destructive button has to be aimed at.
	$effect(() => {
		saveEl?.focus();
	});
</script>

<ModalPrompt title="Save this transcript before closing?" onDismiss={() => onChoice('cancel')}>
	<p>
		{#if endedSession}The session has been stopped and the last captions collected.{/if}
		{lines === 1 ? '1 line' : `${lines} lines`}
		{lines === 1 ? 'has' : 'have'} not been saved. Closing without saving discards
		{lines === 1 ? 'it' : 'them'}.
	</p>

	{#if error}
		<p class="error" role="alert">Could not save: {error}</p>
	{/if}

	<div class="actions">
		<button class="primary" bind:this={saveEl} disabled={saving} onclick={() => onChoice('save')}>
			{saving ? 'Saving…' : 'Save and close'}
		</button>
		<button class="danger" disabled={saving} onclick={() => onChoice('discard')}>
			Discard and close
		</button>
		<button disabled={saving} onclick={() => onChoice('cancel')}>Cancel</button>
	</div>
	<p class="note">Saves Markdown to your Documents folder. Cancel keeps the app open.</p>
</ModalPrompt>
