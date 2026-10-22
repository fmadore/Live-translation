<script lang="ts">
	// The last thing between a live session's transcript and nothing at all. Shown when the
	// operator closes the window with finalized captions that never reached disk (issue #25).

	import ModalPrompt from './ModalPrompt.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import { t } from './i18n';
	import type { CloseChoice } from './document';

	interface Props {
		/** Finalized lines that would be lost. */
		lines: number;
		/** True when the close arrived mid-session and capture has just been drained, so the
		 *  count can be explained rather than appearing from nowhere. */
		endedSession: boolean;
		/** The answer being carried out: Save while the transcript is written, Discard while the
		 *  spool is cleared and the app quits. Every answer is locked meanwhile, Escape too, so
		 *  nothing can be pressed twice or taken back halfway through. */
		answering: Exclude<CloseChoice, 'cancel'> | null;
		/** Last save failure — kept on screen rather than quitting on a write that did not land. */
		error: string;
		onChoice: (choice: CloseChoice) => void;
	}

	let { lines, endedSession, answering, error, onChoice }: Props = $props();
	const busy = $derived(answering !== null);

	let saveEl = $state<HTMLButtonElement | null>(null);

	// Save is the recoverable answer, so it takes focus: Enter keeps the transcript, and the
	// destructive button has to be aimed at.
	$effect(() => {
		saveEl?.focus();
	});
</script>

<ModalPrompt
	title={$t.prompt.unsaved.title}
	onDismiss={busy ? undefined : () => onChoice('cancel')}
>
	<p>
		{#if endedSession}{$t.prompt.unsaved.sessionEnded}{/if}
		{$t.prompt.unsaved.body(lines)}
	</p>

	{#if error}
		<p class="error" role="alert">{$t.prompt.unsaved.failed(error)}</p>
	{/if}

	<div class="actions">
		<ToolButton
			variant="primary"
			size="lg"
			bind:element={saveEl}
			disabled={busy}
			aria-busy={answering === 'save'}
			onclick={() => onChoice('save')}
		>
			{answering === 'save' ? $t.prompt.unsaved.saving : $t.prompt.unsaved.save}
		</ToolButton>
		<ToolButton
			variant="danger"
			size="lg"
			disabled={busy}
			aria-busy={answering === 'discard'}
			onclick={() => onChoice('discard')}
		>
			{$t.prompt.unsaved.discard}
		</ToolButton>
		<ToolButton variant="ghost" size="lg" disabled={busy} onclick={() => onChoice('cancel')}
			>{$t.prompt.unsaved.cancel}</ToolButton
		>
	</div>
	<p class="note">{$t.prompt.unsaved.note}</p>
</ModalPrompt>
