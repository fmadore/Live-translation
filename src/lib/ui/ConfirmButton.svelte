<script lang="ts">
	import ToolButton from './ToolButton.svelte';

	/** A destructive action that asks once before it acts: the first press turns the button into
	 *  its confirmation and shows Cancel beside it, the second press acts. The same button
	 *  element changes its name, so the keyboard stays where it was and a screen reader reads
	 *  the new name in place.
	 *
	 *  Controlled: the caller owns `confirming`, because what withdraws the question (another
	 *  row chosen, the selection changed) is the caller's business. */
	let {
		confirming,
		label,
		confirmLabel,
		cancelLabel,
		disabled = false,
		cancelDisabled = disabled,
		variant = 'default',
		class: className = '',
		onask,
		onconfirm,
		oncancel
	}: {
		/** The question is showing: the next press acts. */
		confirming: boolean;
		label: string;
		confirmLabel: string;
		cancelLabel: string;
		disabled?: boolean;
		/** Cancel's own state; by default the same as the button's. */
		cancelDisabled?: boolean;
		variant?: 'default' | 'danger';
		class?: string;
		/** The first press: ask. */
		onask: () => void;
		/** The second press: act. */
		onconfirm: () => void;
		oncancel: () => void;
	} = $props();
</script>

<ToolButton
	{variant}
	class={className}
	{disabled}
	onclick={() => (confirming ? onconfirm() : onask())}
	>{confirming ? confirmLabel : label}</ToolButton
>{#if confirming}<ToolButton disabled={cancelDisabled} onclick={oncancel}>{cancelLabel}</ToolButton
	>{/if}
