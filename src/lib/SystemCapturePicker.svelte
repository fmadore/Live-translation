<script lang="ts">
	import { onMount } from 'svelte';
	import { options } from './stores';
	import { t } from './i18n';
	import type { AudioDevice, CaptureApplication } from './types';
	let {
		locked,
		outputs,
		applications,
		supported,
		refreshing,
		refresh,
		changed
	}: {
		locked: boolean;
		outputs: AudioDevice[];
		applications: CaptureApplication[];
		supported: boolean | null;
		refreshing: boolean;
		refresh: () => Promise<void>;
		changed: () => void;
	} = $props();
	const applicationMode = $derived($options.systemCapture?.kind === 'application');
	const selected = $derived(
		$options.systemCapture?.kind === 'application' ? $options.systemCapture.process : null
	);
	const selectionKey = $derived(selected ? `${selected.pid}:${selected.createdAt}` : '');
	onMount(() => {
		void refresh();
	});
</script>

<div class="capture-picker">
	<label>
		<span>{$t.applications.mode}</span>
		<select
			disabled={locked}
			value={applicationMode ? 'application' : 'output'}
			onchange={(e) => {
				changed();
				$options = {
					...$options,
					systemCapture:
						e.currentTarget.value === 'application'
							? { kind: 'application', process: null }
							: { kind: 'output' }
				};
			}}
		>
			<option value="output">{$t.applications.output}</option>
			<option value="application">{$t.applications.application}</option>
		</select>
	</label>
	{#if applicationMode}
		<label>
			<span>{$t.applications.choose}</span>
			<select
				disabled={locked || supported === false}
				value={selectionKey}
				onchange={(e) => {
					const app = applications.find(
						(a) => `${a.process.pid}:${a.process.createdAt}` === e.currentTarget.value
					);
					changed();
					$options = {
						...$options,
						systemCapture: { kind: 'application', process: app?.process ?? null }
					};
				}}
			>
				<option value="">{$t.applications.choose}</option>
				{#if selected && !applications.some((a) => a.process.pid === selected.pid && a.process.createdAt === selected.createdAt)}
					<option value={selectionKey}>{$t.applications.missing}</option>
				{/if}
				{#each applications as app (app.process.pid)}
					<option value={`${app.process.pid}:${app.process.createdAt}`}
						>{app.name} (PID {app.process.pid})</option
					>
				{/each}
			</select>
		</label>
		<button class="tool" disabled={refreshing} aria-busy={refreshing} onclick={refresh}
			>{$t.applications.refresh}</button
		>
		{#if supported === false}<p class="hint" role="status">{$t.applications.unsupported}</p>
		{:else if supported && !applications.length}<p class="hint" role="status">
				{$t.applications.empty}
			</p>{/if}
	{:else}
		<label>
			<span>{$t.devices.output}</span>
			<select
				disabled={locked}
				value={$options.systemDeviceId ?? ''}
				onchange={(e) => {
					changed();
					$options = { ...$options, systemDeviceId: e.currentTarget.value || null };
				}}
			>
				<option value="">{$t.rail.systemDefault}</option>
				{#if $options.systemDeviceId && !outputs.some((d) => d.id === $options.systemDeviceId)}<option
						value={$options.systemDeviceId}>{$t.devices.missing}</option
					>{/if}
				{#each outputs as dev (dev.id)}<option value={dev.id}
						>{dev.isDefault ? $t.rail.isDefault(dev.name) : dev.name}</option
					>{/each}
			</select>
		</label>
	{/if}
</div>

<style>
	.capture-picker {
		display: grid;
		gap: 0.6em;
		min-width: 0;
	}
	label {
		display: grid;
		gap: 0.4em;
		color: var(--muted-2);
		font-size: var(--type-12);
		min-width: 0;
	}
	select {
		width: 100%;
		min-width: 0;
		padding: 0.65em;
		border: 1px solid var(--border);
		border-radius: 7px;
		background: var(--bg);
		color: var(--text);
		font: inherit;
	}
	button {
		justify-self: start;
		padding: 0.5em;
		border: 1px solid var(--border);
		border-radius: 7px;
		background: transparent;
		color: var(--text);
		font-size: var(--type-12);
	}
	.hint {
		margin: 0;
		color: var(--muted-2);
		font-size: var(--type-12);
		line-height: 1.5;
	}
</style>
