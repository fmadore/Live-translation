<script lang="ts">
	// Shared chrome for the two decisions the operator must not miss: an unsaved transcript at
	// quit, and a recovery spool found at launch. Both are modal because both are the last
	// moment at which an event's record can still be kept.

	import { onMount } from 'svelte';
	import type { Snippet } from 'svelte';

	interface Props {
		/** Used as the dialog's accessible name. */
		title: string;
		/** Escape, so a dialog is never a trap. Omit for one that has no safe dismissal. */
		onDismiss?: () => void;
		children: Snippet;
	}

	let { title, onDismiss, children }: Props = $props();

	const titleId = $props.id();

	let prompt = $state<HTMLDivElement | null>(null);

	// Answering the dialog puts focus back where it was taken from. Captured before the callers
	// focus their own default button, which they do from their own `onMount`.
	onMount(() => {
		const opener = document.activeElement as HTMLElement | null;
		return () => opener?.focus?.();
	});

	// `aria-modal` tells a screen reader to ignore what is behind the dialog, but nothing stops
	// the Tab key from walking into it — a keyboard operator would be typing into a session
	// they can no longer see. The cycle stays inside until the dialog answers.
	function focusable(): HTMLElement[] {
		if (!prompt) return [];
		return Array.from(
			prompt.querySelectorAll<HTMLElement>(
				'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])'
			)
		);
	}

	function onKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape' && onDismiss) {
			event.preventDefault();
			onDismiss();
			return;
		}
		if (event.key !== 'Tab') return;
		const stops = focusable();
		if (!stops.length) return;
		const first = stops[0];
		const last = stops[stops.length - 1];
		const active = document.activeElement as HTMLElement | null;
		// Focus that has already escaped the dialog — or has not entered it yet — is pulled
		// back to the end or the start of the cycle, whichever way Tab was going.
		if (!prompt?.contains(active)) {
			event.preventDefault();
			(event.shiftKey ? last : first).focus();
		} else if (event.shiftKey && active === first) {
			event.preventDefault();
			last.focus();
		} else if (!event.shiftKey && active === last) {
			event.preventDefault();
			first.focus();
		}
	}
</script>

<svelte:window on:keydown={onKeydown} />

<div class="scrim">
	<div class="prompt" bind:this={prompt} role="dialog" aria-modal="true" aria-labelledby={titleId}>
		<h2 id={titleId}>{title}</h2>
		{@render children()}
	</div>
</div>

<style>
	.scrim {
		position: fixed;
		inset: 0;
		z-index: 50;
		display: grid;
		place-items: center;
		padding: 24px;
		background: rgba(8, 9, 11, 0.78);
	}
	.prompt {
		width: min(460px, 100%);
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 24px;
		border-radius: 14px;
		border: 1px solid var(--border);
		background: var(--panel);
		box-shadow: 0 24px 64px rgba(0, 0, 0, 0.55);
	}
	h2 {
		margin: 0;
		font-size: 17px;
		font-weight: 600;
		line-height: 1.35;
		color: var(--text-bright);
		text-wrap: balance;
	}

	/* The bodies live in the caller's snippet, so their rules have to reach into it. */
	.prompt :global(p) {
		margin: 0;
		font-size: 13px;
		line-height: 1.6;
		color: var(--text-soft);
		text-wrap: pretty;
	}
	.prompt :global(p.note) {
		font-size: 11.5px;
		color: var(--muted-3);
	}
	.prompt :global(p.error) {
		padding: 9px 11px;
		border-radius: 8px;
		border: 1px solid var(--danger-border);
		background: var(--danger-bg);
		color: var(--danger-soft);
		font-size: 12.5px;
		word-break: break-word;
	}
	.prompt :global(code) {
		font-family: var(--font-mono);
		font-size: 11px;
		word-break: break-all;
	}
	.prompt :global(.actions) {
		display: flex;
		flex-wrap: wrap;
		gap: 9px;
		margin-top: 2px;
	}
	.prompt :global(.actions button) {
		font-size: 13px;
		font-weight: 500;
		line-height: 1;
		padding: 10px 15px;
		border-radius: 8px;
		border: 1px solid var(--border);
		background: transparent;
		color: var(--text-soft);
	}
	.prompt :global(.actions button:hover:not(:disabled)) {
		border-color: var(--border-hover);
		color: var(--text);
	}
	.prompt :global(.actions button.primary) {
		flex: 1 1 auto;
		font-weight: 600;
		color: var(--on-accent);
		background: var(--accent);
		border-color: var(--accent);
	}
	.prompt :global(.actions button.primary:hover:not(:disabled)) {
		background: var(--accent-2);
		border-color: var(--accent-2);
	}
	.prompt :global(.actions button.danger) {
		color: var(--danger-soft);
		border-color: var(--danger-border);
	}
	.prompt :global(.actions button.danger:hover:not(:disabled)) {
		background: var(--danger-bg);
		border-color: var(--danger);
		color: var(--danger-soft);
	}
	.prompt :global(.actions button:focus-visible) {
		outline: 2px solid var(--accent);
		outline-offset: 2px;
	}
</style>
