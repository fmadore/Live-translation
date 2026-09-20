<script lang="ts">
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
	<button
		type="button"
		class="ui-tool"
		disabled={disabled || value <= min}
		aria-label={`${label} −`}
		onclick={() => onchange(Math.max(min, value - step))}>−</button
	>
	<output>{value} {unit}</output>
	<button
		type="button"
		class="ui-tool"
		disabled={disabled || value >= max}
		aria-label={`${label} +`}
		onclick={() => onchange(Math.min(max, value + step))}>+</button
	>
</div>
