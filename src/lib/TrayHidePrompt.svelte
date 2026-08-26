<script lang="ts">
	// Shown the first time closing the window hides the app instead of quitting it.
	//
	// An app that disappears from the taskbar and keeps running reads as a crash unless it
	// says otherwise — and this one may still be holding a microphone. Said once, in the
	// window the operator is looking at, rather than as a toast: Focus Assist, notification
	// settings and a full-screen slideshow can all swallow a toast, and this is the one
	// message that has to land.

	import ModalPrompt from './ModalPrompt.svelte';

	interface Props {
		/** True when a session is still running, which is what makes this worth saying. */
		running: boolean;
		onChoice: (choice: 'hide' | 'quit') => void;
	}

	let { running, onChoice }: Props = $props();

	let hideEl = $state<HTMLButtonElement | null>(null);

	$effect(() => {
		hideEl?.focus();
	});
</script>

<ModalPrompt title="Live Translation will keep running" onDismiss={() => onChoice('quit')}>
	<p>
		You asked for closing this window to leave the app running in the tray, so it will
		disappear from the taskbar
		{#if running}
			and keep captioning.
		{:else}
			but stay ready.
		{/if}
		Its icon stays in the notification area, next to the clock — open it, show or hide the
		overlay, stop the session, or quit from there.
	</p>
	<p class="note">
		Said once. Turn it off again with <em>Keep running in the tray when I close this window</em>.
	</p>

	<div class="actions">
		<button class="primary" bind:this={hideEl} onclick={() => onChoice('hide')}>
			Got it — hide to the tray
		</button>
		<button onclick={() => onChoice('quit')}>Quit instead</button>
	</div>
</ModalPrompt>

<style>
	em {
		font-style: normal;
		color: var(--text-soft);
	}
</style>
