<script lang="ts">
	import { t, localeTag } from '../i18n';
	let { label, value = $bindable('') }: { label: string; value?: string } = $props();
	const id = $props.id();
	let picker: HTMLInputElement;
	let draft = $state(value);
	let invalid = $state(false);
	$effect(() => {
		draft = value;
		invalid = false;
	});
	function edit(raw: string) {
		draft = raw;
		const date = new Date(`${raw}T00:00:00Z`);
		const valid =
			/^\d{4}-\d{2}-\d{2}$/.test(raw) &&
			!Number.isNaN(date.getTime()) &&
			date.toISOString().slice(0, 10) === raw;
		invalid = !!raw && !valid;
		if (!invalid) value = raw;
	}
</script>

<div class="ui-field">
	<label for={id}>{label}</label>
	<div class="date-control">
		<input
			{id}
			type="text"
			value={draft}
			placeholder={$t.usability.dateFormat}
			maxlength="10"
			aria-invalid={invalid}
			aria-describedby={invalid ? `${id}-error` : undefined}
			oninput={(event) => edit(event.currentTarget.value)}
		/>
		<button
			type="button"
			aria-label={`${label} — ${$t.usability.chooseDate}`}
			onclick={() => picker.showPicker()}
		>
			<svg
				width="16"
				height="16"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.7"
				aria-hidden="true"
				><rect x="3" y="5" width="18" height="16" rx="2" /><path
					d="M16 3v4M8 3v4M3 11h18M7 15h2M11 15h2M15 15h2M7 18h2M11 18h2"
				/></svg
			>
		</button>
		<input
			class="native-picker"
			type="date"
			lang={$localeTag}
			bind:this={picker}
			{value}
			tabindex="-1"
			aria-hidden="true"
			onchange={(event) => {
				value = event.currentTarget.value;
			}}
		/>
	</div>
	{#if invalid}<span id={`${id}-error`} class="date-error">{$t.usability.invalidDate}</span>{/if}
</div>

<style>
	.date-control {
		position: relative;
	}
	.date-control > input[type='text'] {
		width: 100%;
		padding-right: 2.75rem;
	}
	button {
		position: absolute;
		right: 1px;
		top: 1px;
		bottom: 1px;
		width: 2.5rem;
		display: grid;
		place-items: center;
		padding: 0;
		border: 0;
		border-radius: var(--radius-control);
		background: transparent;
		color: var(--text-soft);
	}
	button:hover {
		background: var(--surface-3);
	}
	.native-picker {
		position: absolute;
		bottom: 0;
		right: 0;
		width: 1px;
		height: 1px;
		opacity: 0;
		pointer-events: none;
	}
	.date-error {
		color: var(--danger-soft);
		font-size: var(--type-caption);
	}
</style>
