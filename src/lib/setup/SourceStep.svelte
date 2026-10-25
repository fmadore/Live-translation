<script lang="ts">
	import LevelMeter from '../LevelMeter.svelte';
	import SystemCapturePicker from '../SystemCapturePicker.svelte';
	import ToolButton from '../ui/ToolButton.svelte';
	import Field from '../ui/Field.svelte';
	import Select from '../ui/Select.svelte';
	import { t } from '../i18n';
	import { micLevel, options, systemLevel } from '../stores';
	import { preflightContext } from '../operatorContext';
	import type { SetupActions } from '../setupActions';

	/** Step 02: where the audio comes from — the room, the computer's sound, or both — with the
	 *  devices for each and their meters. */
	let {
		actions,
		locked,
		browserMode,
		usesMic,
		usesSystem
	}: {
		actions: SetupActions;
		locked: boolean;
		browserMode: boolean;
		usesMic: boolean;
		usesSystem: boolean;
	} = $props();
	const preflight = preflightContext();
	// Only then are applications listed: see `SystemCapturePicker`.
	const applicationCapture = $derived(usesSystem && $options.systemCapture?.kind === 'application');
</script>

<section class="rail-section">
	<div class="step-head">
		<span class="step-no">02</span>
		<h2 class="kicker">{$t.rail.step.whereFrom}</h2>
	</div>
	<div class="tiles">
		<button
			class="tile ui-card"
			disabled={locked}
			aria-pressed={$options.source === 'microphone'}
			onclick={() => actions.setSource('microphone')}
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"
				aria-hidden="true"
				><rect x="9" y="2.5" width="6" height="11" rx="3" /><path
					d="M5.5 11.5a6.5 6.5 0 0 0 13 0"
				/><path d="M12 18v3.5" /></svg
			>
			<span>{$options.provider === 'ondevice' ? $t.source.demo : $t.source.microphone}</span>
		</button>
		<button
			class="tile ui-card"
			disabled={locked || $options.provider === 'ondevice'}
			aria-pressed={$options.source === 'system'}
			onclick={() => actions.setSource('system')}
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"
				stroke-linejoin="round"
				aria-hidden="true"
				><path d="M4 9.5h3.5L13 5v14L7.5 14.5H4z" /><path d="M16.5 9.2a4.2 4.2 0 0 1 0 5.6" /></svg
			>
			<span>{$t.source.system}</span>
		</button>
		<button
			class="tile ui-card"
			disabled={locked || $options.provider === 'ondevice'}
			aria-pressed={$options.source === 'both'}
			onclick={() => actions.setSource('both')}
		>
			<svg
				width="18"
				height="18"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="1.75"
				stroke-linecap="round"
				aria-hidden="true"><path d="M3.5 6.5h4.5L12 12l4 5.5h4.5M3.5 17.5h4.5L12 12" /></svg
			>
			<span>{$t.source.both}</span>
		</button>
	</div>
	<p class="hint">
		{$options.provider === 'ondevice'
			? $t.rail.demoSourceHint
			: applicationCapture
				? $t.applications.hint
				: $t.rail.sourceHint}
	</p>

	{#if usesMic && $options.provider !== 'ondevice'}
		<Field label={$t.rail.micDevice}>
			<Select
				disabled={locked}
				value={$options.micDeviceId ?? ''}
				onchange={(e) => {
					preflight.invalidateAudioTest();
					$options = {
						...$options,
						micDeviceId: e.currentTarget.value || null,
						micDeviceName: null
					};
				}}
			>
				<option value="">{$t.rail.systemDefault}</option>
				{#if $options.micDeviceId && !preflight.microphones.some((d) => d.id === $options.micDeviceId)}
					<option value={$options.micDeviceId}>{$t.devices.missing}</option>
				{/if}
				{#each preflight.microphones as dev (dev.id)}
					<option value={dev.id}>
						{dev.isDefault ? $t.rail.isDefault(dev.name) : dev.name}
					</option>
				{/each}
			</Select>
		</Field>
	{/if}

	{#if usesSystem && $options.provider !== 'ondevice'}
		<SystemCapturePicker
			{locked}
			outputs={preflight.outputs}
			applications={preflight.applications}
			supported={preflight.applicationCaptureSupported}
			refresh={preflight.refreshApplications}
			changed={preflight.invalidateAudioTest}
		/>
	{/if}
	{#if $options.provider !== 'ondevice'}
		<ToolButton
			disabled={browserMode || preflight.refreshing || preflight.refreshingApplications}
			aria-busy={preflight.refreshing || preflight.refreshingApplications}
			onclick={() =>
				Promise.all([
					preflight.refresh(),
					applicationCapture ? preflight.refreshApplications() : undefined
				])}
		>
			{preflight.refreshing ? $t.devices.refreshing : $t.devices.refresh}
		</ToolButton>
	{/if}
	<div class="meters">
		{#if usesMic}
			<LevelMeter
				level={$micLevel}
				label={$options.provider === 'ondevice' ? $t.source.demo : $t.source.microphone}
			/>
		{/if}
		{#if usesSystem}
			<LevelMeter level={$systemLevel} label={$t.stage.origin.system} />
		{/if}
	</div>
</section>

<style>
	/* Direct children of the rail's scrolling column keep their height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.kicker {
		flex: 0 0 auto;
	}

	/* ---- Selection cards ----------------------------------------------------
	   The border, the fill and the selected wash of the source tiles and the engine rows
	   are the shared .ui-card's; what is set here is their layout and how their text
	   follows the selection. */

	.tiles {
		display: grid;
		grid-template-columns: repeat(3, 1fr);
		gap: var(--space-2);
	}
	.tile {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3) var(--space-1) var(--space-2);
		color: var(--text-muted);
		font-size: var(--type-small);
		font-weight: 500;
		line-height: 1;
	}
	.tile[aria-pressed='true'] {
		color: var(--accent-soft);
		font-weight: 600;
	}

	.meters {
		display: flex;
		flex-direction: column;
		gap: var(--space-2);
		margin-top: var(--space-2);
	}
</style>
