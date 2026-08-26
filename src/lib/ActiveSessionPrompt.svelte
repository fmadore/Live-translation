<script lang="ts">
	// Asked before anything is stopped, when leaving would cut a live session short.
	//
	// The whole point of issue #22 is that an accidental click on the window's X must not end
	// an event's captions. So this comes first — ahead of the stop, ahead of the drain, ahead
	// of any question about the transcript — and its safe answer is the one that changes
	// nothing.

	import ModalPrompt from './ModalPrompt.svelte';

	interface Props {
		/** Elapsed session time, already formatted, so the operator can see what they would
		 *  be ending rather than being asked about an abstraction. */
		elapsed: string;
		/** True when the request came from the tray rather than the window's close button. */
		fromTray: boolean;
		onChoice: (stop: boolean) => void;
	}

	let { elapsed, fromTray, onChoice }: Props = $props();

	let keepEl = $state<HTMLButtonElement | null>(null);

	// Focus stays on the answer that keeps the session: Enter is safe, and ending an event's
	// captions has to be aimed at.
	$effect(() => {
		keepEl?.focus();
	});
</script>

<ModalPrompt title="A caption session is running" onDismiss={() => onChoice(false)}>
	<p>
		Captions have been live for {elapsed}. Closing stops the session, waits for the last
		captions to arrive, and then quits.
	</p>
	{#if !fromTray}
		<p class="note">
			To put the window away without stopping anything, use Minimize to tray — or turn on
			<em>Keep running in the tray when I close this window</em>.
		</p>
	{/if}

	<div class="actions">
		<button class="primary" bind:this={keepEl} onclick={() => onChoice(false)}>
			Keep captioning
		</button>
		<button class="danger" onclick={() => onChoice(true)}>Stop and close</button>
	</div>
</ModalPrompt>

<style>
	em {
		font-style: normal;
		color: var(--text-soft);
	}
</style>
