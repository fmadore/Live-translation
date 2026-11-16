<script lang="ts">
	import { localeTag, t } from './i18n';
	import { formatUsd } from './providers';
	import {
		COST_WARNING_CHOICES,
		costWarningUsd,
		DURATION_WARNING_CHOICES,
		durationWarningHours,
		IDLE_PAUSE_CHOICES,
		idlePauseMinutes
	} from './spendingGuard';
	import Field from './ui/Field.svelte';
	import Select from './ui/Select.svelte';

	/** Settings › App: the spending guard's three choices, each Never until picked. */
	const idBase = $props.id();
</script>

<div class="rail-section">
	<h2 class="kicker">{$t.guard.heading}</h2>
	<Field label={$t.guard.idlePause}>
		<Select
			value={String($idlePauseMinutes)}
			aria-describedby="{idBase}-note"
			onchange={(e) => idlePauseMinutes.set(Number(e.currentTarget.value))}
		>
			{#each IDLE_PAUSE_CHOICES as minutes (minutes)}
				<option value={String(minutes)}
					>{minutes ? $t.guard.minutes(minutes) : $t.guard.never}</option
				>
			{/each}
		</Select>
	</Field>
	<Field label={$t.guard.costWarning}>
		<Select
			value={String($costWarningUsd)}
			onchange={(e) => costWarningUsd.set(Number(e.currentTarget.value))}
		>
			{#each COST_WARNING_CHOICES as amount (amount)}
				<option value={String(amount)}
					>{amount ? formatUsd(amount, $localeTag) : $t.guard.never}</option
				>
			{/each}
		</Select>
	</Field>
	<Field label={$t.guard.durationWarning}>
		<Select
			value={String($durationWarningHours)}
			onchange={(e) => durationWarningHours.set(Number(e.currentTarget.value))}
		>
			{#each DURATION_WARNING_CHOICES as hours (hours)}
				<option value={String(hours)}>{hours ? $t.guard.hours(hours) : $t.guard.never}</option>
			{/each}
		</Select>
	</Field>
	<p class="hint" id="{idBase}-note">{$t.guard.note}</p>
</div>
