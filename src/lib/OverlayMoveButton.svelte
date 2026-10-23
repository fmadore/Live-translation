<script lang="ts">
	import ToolButton from './ui/ToolButton.svelte';
	import { t } from './i18n';
	import { overlayPlaced } from './stores';
	import { overlayContext } from './operatorContext';

	/** The one button that puts the overlay into move mode and takes it out again, wherever it
	 *  is offered: the running rail, the pre-flight check and the settings panel.
	 *
	 *  Its name changes with the state, so it is not also a pressed toggle (see
	 *  `docs/accessibility.md`); while moving, "Done" is the primary action instead. */
	let {
		wording,
		idleVariant = 'default',
		size = 'md',
		wide = false,
		icon = true,
		disabled = false
	}: {
		/** Whose words it uses: `move` is the running rail's (moving, then finishing moving),
		 *  `place` the pre-flight row's and Settings' (placing or adjusting, then finishing
		 *  placing), each from its own part of the catalog. */
		wording: 'move' | 'place';
		/** How the button is drawn while not moving. */
		idleVariant?: 'default' | 'ghost' | 'warn';
		size?: 'sm' | 'md';
		wide?: boolean;
		icon?: boolean;
		disabled?: boolean;
	} = $props();

	const overlay = overlayContext();

	const label = $derived(
		wording === 'move'
			? overlay.moveOverlay
				? $t.overlayControls.moveDoneLabel
				: $t.overlayControls.moveLabel
			: overlay.moveOverlay
				? $t.preflight.overlay.doneLabel
				: $overlayPlaced
					? $t.preflight.overlay.adjustLabel
					: $t.preflight.overlay.placeLabel
	);
	const text = $derived(
		wording === 'move'
			? overlay.moveOverlay
				? $t.overlayControls.done
				: $t.overlayControls.move
			: overlay.moveOverlay
				? $t.preflight.overlay.done
				: $overlayPlaced
					? $t.preflight.overlay.adjust
					: $t.preflight.overlay.place
	);
</script>

<ToolButton
	{wide}
	{size}
	{disabled}
	variant={overlay.moveOverlay ? 'primary' : idleVariant}
	aria-label={label}
	onclick={overlay.toggleMoveOverlay}
>
	{#if icon}
		<svg
			width="14"
			height="14"
			viewBox="0 0 24 24"
			fill="none"
			stroke="currentColor"
			stroke-width="1.75"
			stroke-linecap="round"
			stroke-linejoin="round"
			aria-hidden="true"
			><path
				d="M12 3.5v17M3.5 12h17M12 3.5l-3 3M12 3.5l3 3M12 20.5l-3-3M12 20.5l3-3M3.5 12l3-3M3.5 12l3 3M20.5 12l-3-3M20.5 12l-3 3"
			/></svg
		>
	{/if}
	{text}
</ToolButton>
