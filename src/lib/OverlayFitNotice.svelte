<script lang="ts">
	import { t } from './i18n';
	import type { OverlayFit } from './types';

	/** Beside the caption size in the live rail: what the overlay reported when its rows could
	 *  not each hold a line at that size (`fitRows`), so the control never reads 38 px while the
	 *  room sees 20 px with nothing said. */
	let { fit }: { fit: OverlayFit | null } = $props();

	const shown = $derived(fit !== null && (fit.squeezed || fit.originalHidden || !fit.room));
	// Said once per state, without the size: a resize moves the size without anything worth
	// interrupting for (`docs/accessibility.md`). The visible notice carries the size.
	const announcement = $derived(
		fit && shown
			? [
					!fit.room
						? $t.overlayControls.fit.noRoom(fit.rows)
						: fit.squeezed
							? $t.overlayControls.fit.smallerAnnounce(fit.rows)
							: '',
					fit.originalHidden ? $t.overlayControls.fit.originalHidden : ''
				]
					.filter(Boolean)
					.join(' ')
			: ''
	);
</script>

<!-- Mounted before it has anything to say: a status inserted together with its text is often
     missed. -->
<p class="sr-only" role="status">{announcement}</p>
{#if fit && shown}
	<p class="fit-notice">
		{#if !fit.room}{$t.overlayControls.fit.noRoom(
				fit.rows
			)}{:else if fit.squeezed}{$t.overlayControls.fit.smaller(fit.fontSize, fit.rows)}{/if}
		{#if fit.originalHidden}{$t.overlayControls.fit.originalHidden}{/if}
		{$t.overlayControls.fit.action($t.overlayControls.move, $t.overlay.snapToBottom)}
	</p>
{/if}

<style>
	.fit-notice {
		margin: 0;
		font-size: var(--type-caption);
		line-height: var(--leading-snug);
		color: var(--warn-soft);
	}
</style>
