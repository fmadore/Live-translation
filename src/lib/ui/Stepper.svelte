<script lang="ts">
	import ToolButton from './ToolButton.svelte';
	let {
		label,
		value,
		min,
		max,
		step = 1,
		unit = '',
		disabled = false,
		onchange
	}: {
		label: string;
		value: number;
		min: number;
		max: number;
		step?: number;
		unit?: string;
		disabled?: boolean;
		onchange: (value: number) => void;
	} = $props();
</script>

<div class="ui-stepper" role="group" aria-label={label}>
	<span>{label}</span>
	<ToolButton
		size="sm"
		disabled={disabled || value <= min}
		aria-label={`${label} −`}
		onclick={() => onchange(Math.max(min, value - step))}>−</ToolButton
	>
	<output>{value} {unit}</output>
	<ToolButton
		size="sm"
		disabled={disabled || value >= max}
		aria-label={`${label} +`}
		onclick={() => onchange(Math.min(max, value + step))}>+</ToolButton
	>
</div>
