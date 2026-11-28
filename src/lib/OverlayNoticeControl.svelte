<script lang="ts">
	import { tick } from 'svelte';
	import { de, en, fr, locale, t } from './i18n';
	import { NOTICE_MAX_CHARS, NOTICE_PRESETS, noticeLocale, normalizeNotice } from './overlayNotice';
	import { noticeUntilCaptions, options } from './stores';
	import { captionLanguageOf } from './types';
	import { overlayContext } from './operatorContext';
	import ToolButton from './ui/ToolButton.svelte';
	import Preference from './ui/Preference.svelte';

	/** The live rail's notice for the overlay: one line the room reads instead of the captions
	 *  during a break. A suggestion shows at once; anything else is typed and shown. */
	const overlay = overlayContext();

	let open = $state(false);
	let text = $state('');
	let field = $state<HTMLInputElement | null>(null);
	let opener = $state<HTMLButtonElement | null>(null);
	const id = $props.id();

	// The suggestions are for the room, so they are in the caption language when the app has
	// it, and in the interface language otherwise.
	const presetMessages = $derived(
		{ en, fr, de }[noticeLocale(captionLanguageOf($options), $locale)].overlayNotice.presets
	);

	const shown = $derived(overlay.notice);
	// What the status region says: what the room now reads, or that captions are back — by the
	// Clear button, or by the first caption after a pause. Empty until a notice has been shown,
	// so the region is quiet at the start.
	let announcement = $state('');
	let wasShown = '';
	$effect(() => {
		if (wasShown && !shown) announcement = $t.overlayNotice.cleared;
		wasShown = shown;
	});

	async function openPanel() {
		open = true;
		await tick();
		field?.focus();
	}

	function closePanel() {
		open = false;
		void tick().then(() => opener?.focus());
	}

	function show(value: string) {
		const notice = normalizeNotice(value);
		if (!notice) return;
		overlay.setNotice(notice);
		announcement = $t.overlayNotice.showing(notice);
		text = '';
		open = false;
	}

	function clear() {
		overlay.clearNotice();
	}
</script>

<p class="sr-only" role="status">{announcement}</p>

{#if shown}
	<div class="showing">
		<p aria-hidden="true">{$t.overlayNotice.showing(shown)}</p>
		<ToolButton variant="warn" wide onclick={clear}>{$t.overlayNotice.clear}</ToolButton>
	</div>
{:else if open}
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions (Escape anywhere in the panel puts it away, as it does in a dialog.) -->
	<form
		class="notice-form"
		aria-labelledby="{id}-heading"
		onsubmit={(event) => {
			event.preventDefault();
			show(text);
		}}
		onkeydown={(event) => {
			if (event.key === 'Escape') {
				event.preventDefault();
				event.stopPropagation();
				closePanel();
			}
		}}
	>
		<h3 class="heading" id="{id}-heading">{$t.overlayNotice.heading}</h3>
		<div class="presets" role="group" aria-label={$t.overlayNotice.suggestions}>
			{#each NOTICE_PRESETS as preset (preset)}
				<ToolButton size="sm" onclick={() => show(presetMessages[preset])}>
					{presetMessages[preset]}
				</ToolButton>
			{/each}
		</div>
		<label class="field">
			<span>{$t.overlayNotice.field}</span>
			<input
				bind:this={field}
				bind:value={text}
				type="text"
				maxlength={NOTICE_MAX_CHARS}
				autocomplete="off"
				aria-describedby="{id}-hint"
			/>
		</label>
		<Preference
			label={$t.overlayNotice.untilCaptions}
			checked={$noticeUntilCaptions}
			onchange={(value) => noticeUntilCaptions.set(value)}
		/>
		<p class="hint" id="{id}-hint">{$t.overlayNotice.hint}</p>
		<div class="actions">
			<ToolButton type="submit" variant="primary" disabled={!normalizeNotice(text)}>
				{$t.overlayNotice.show}
			</ToolButton>
			<ToolButton variant="ghost" onclick={closePanel}>{$t.marker.cancel}</ToolButton>
		</div>
	</form>
{:else}
	<ToolButton wide bind:element={opener} onclick={openPanel}>
		{$t.overlayNotice.openLabel}
	</ToolButton>
{/if}

<style>
	.showing,
	.notice-form {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
	}
	.notice-form {
		padding: var(--space-3);
		border: 1px solid var(--line);
		border-radius: var(--radius-card);
		background: var(--surface-1);
	}
	.showing p {
		margin: 0;
		font-size: var(--type-small);
		line-height: var(--leading-body);
		color: var(--warn-soft);
		overflow-wrap: anywhere;
	}
	.heading {
		margin: 0;
		font-size: var(--type-small);
		font-weight: 600;
		color: var(--text-secondary);
	}
	.presets {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
	.field {
		display: flex;
		flex-direction: column;
		gap: var(--space-1);
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	.field input {
		font: inherit;
		min-height: var(--control-sm);
		color: var(--text-body);
		background: var(--surface-0);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		padding: var(--space-1) var(--space-2);
	}
	.hint {
		margin: 0;
		font-size: var(--type-caption);
		line-height: var(--leading-snug);
		color: var(--text-muted);
	}
	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
	}
</style>
