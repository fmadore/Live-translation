<script lang="ts">
	import { languageName } from './languages';
	import { locale, t } from './i18n';
	import ToolButton from './ui/ToolButton.svelte';
	import type { FallbackOffer } from './sourceFallback.svelte';
	import type { Origin, Provider } from './types';

	let {
		offer,
		origins,
		from,
		reconnecting,
		switching,
		onContinue,
		onDismiss
	}: {
		/** What continuing would do: Whisper's task, and what it changes for the room. */
		offer: FallbackOffer;
		/** The sources waiting, in the order they are continued. */
		origins: readonly Origin[];
		/** The engine each source was on. */
		from: (origin: Origin) => Provider;
		/** Whether a source is waiting because it is still reconnecting. */
		reconnecting: (origin: Origin) => boolean;
		switching: boolean;
		onContinue: () => void;
		/** Not now: leave the sources as they are, ending a run nothing else keeps going. */
		onDismiss: () => void;
	} = $props();

	const noteId = $props.id();
	// The consequence is on the button, before the click: the audience's overlay changes
	// language (decision 4).
	const spoken = $derived(offer.note !== null && offer.note.kind !== 'keepsEnglish');
	const note = $derived.by(() => {
		const n = offer.note;
		if (!n) return '';
		if (n.kind === 'keepsEnglish') return $t.fallback.keepsEnglish;
		if (n.kind === 'turboCannotTranslate') return $t.fallback.turboCannotTranslate;
		return $t.fallback.cannotTranslate(languageName(n.language, $locale));
	});
</script>

<section class="source-fallback" aria-label={$t.fallback.region}>
	{#each origins as origin (origin)}
		<p class="hint">
			{(reconnecting(origin) ? $t.fallback.reconnecting : $t.fallback.ended)(
				$t.source[origin],
				$t.provider.vendor[from(origin)]
			)}
		</p>
	{/each}
	{#if note || origins.length > 1}
		<p class="hint" id={noteId}>
			{note}{#if note && origins.length > 1}{' '}{/if}{#if origins.length > 1}{$t.fallback
					.bothSources}{/if}
		</p>
	{/if}
	<ToolButton
		variant="primary"
		disabled={switching}
		aria-describedby={note || origins.length > 1 ? noteId : undefined}
		onclick={onContinue}
		>{switching
			? $t.fallback.switching
			: spoken
				? $t.fallback.spoken
				: $t.fallback.whisper}</ToolButton
	>
	<ToolButton disabled={switching} onclick={onDismiss}>{$t.fallback.notNow}</ToolButton>
</section>

<style>
	.source-fallback {
		padding: var(--space-3) var(--space-5);
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		border-bottom: 1px solid var(--line-strong);
	}
	.source-fallback p {
		flex-basis: 100%;
		margin: 0;
	}
	.source-fallback > :global(.ui-tool) {
		flex: 1;
	}
</style>
