<script lang="ts">
	import { get } from 'svelte/store';
	import { t } from './i18n';
	import { api, isTauri } from './tauri';
	import { describeError } from './errors';
	import { validateDevices } from './audioDevices';
	import { normalizeStartOptions } from './types';
	import { PROFILES_KEY, decodeProfiles, type MeetingProfile } from './profiles';
	import {
		options,
		overlayFontSize,
		overlayCaptionWidth,
		overlayCaptionLayout,
		overlayCaptionFace,
		overlayPalette,
		overlayCleanSpeech,
		overlayHoldSeconds,
		overlayPace
	} from './stores';
	import type { OverlayController } from './overlayController.svelte';
	let {
		locked,
		overlay,
		onLoaded,
		onBusy
	}: {
		locked: boolean;
		overlay: OverlayController;
		onLoaded: () => Promise<void>;
		onBusy: (busy: boolean) => void;
	} = $props();
	let profiles = $state(
		decodeProfiles(typeof localStorage === 'undefined' ? null : localStorage.getItem(PROFILES_KEY))
	);
	let selected = $state('');
	let name = $state('');
	let busy = $state(false);
	let notice = $state('');
	let error = $state('');
	let confirming = $state(false);
	function persist(next: MeetingProfile[]) {
		localStorage.setItem(PROFILES_KEY, JSON.stringify(next));
		profiles = next;
	}
	async function perform(action: () => Promise<void>) {
		if (locked || busy) return;
		busy = true;
		onBusy(true);
		error = '';
		notice = '';
		try {
			await action();
		} catch (e) {
			error = describeError(e, get(t));
		} finally {
			busy = false;
			onBusy(false);
		}
	}
	function save() {
		return perform(async () => {
			if (!name.trim()) return;
			const profile: MeetingProfile = {
				id: crypto.randomUUID(),
				name: name.trim().slice(0, 80),
				options: normalizeStartOptions(get(options)),
				appearance: {
					fontSize: get(overlayFontSize),
					width: get(overlayCaptionWidth),
					layout: get(overlayCaptionLayout),
					face: get(overlayCaptionFace),
					palette: { ...get(overlayPalette) },
					cleanSpeech: get(overlayCleanSpeech),
					hold: get(overlayHoldSeconds),
					pace: get(overlayPace)
				},
				placement: isTauri() ? await api.getOverlayPlacement() : null
			};
			persist([...profiles, profile]);
			selected = profile.id;
			name = '';
			notice = get(t).usability.profileSaved;
		});
	}
	function load() {
		return perform(async () => {
			const profile = profiles.find((p) => p.id === selected);
			if (!profile) return;
			const [mics, outputs] = await Promise.all([api.listMicrophones(), api.listOutputs()]);
			const saved = normalizeStartOptions(profile.options);
			const checked = validateDevices(saved, mics, outputs);
			if (profile.placement) await api.setOverlayPlacement(profile.placement);
			const a = profile.appearance;
			options.set(checked);
			overlayFontSize.set(a.fontSize);
			overlayCaptionWidth.set(a.width);
			overlayCaptionLayout.set(a.layout);
			overlayCaptionFace.set(a.face);
			overlayPalette.set({ ...a.palette });
			overlayCleanSpeech.set(a.cleanSpeech);
			overlayHoldSeconds.set(a.hold);
			overlayPace.set(a.pace);
			overlay.initialize();
			overlay.pushOverlayConfig({ interactive: overlay.moveOverlay });
			await onLoaded();
			notice =
				(saved.micDeviceId && !checked.micDeviceId) ||
				(saved.systemDeviceId && !checked.systemDeviceId)
					? get(t).usability.profileMissing
					: get(t).usability.profileLoaded;
		});
	}
	function remove() {
		if (!confirming) {
			confirming = true;
			return;
		}
		return perform(async () => {
			persist(profiles.filter((p) => p.id !== selected));
			selected = '';
			confirming = false;
		});
	}
</script>

<details class="profiles">
	<summary>{$t.usability.profiles}</summary>
	<p>{$t.usability.profileHint}</p>
	<label
		>{$t.usability.profileName}<input
			maxlength="80"
			bind:value={name}
			disabled={locked || busy}
		/></label
	>
	<button disabled={locked || busy || !name.trim()} onclick={save}
		>{$t.usability.saveProfile}</button
	>
	<label
		>{$t.usability.chooseProfile}<select
			bind:value={selected}
			disabled={locked || busy}
			onchange={() => (confirming = false)}
			><option value="">{$t.usability.chooseProfile}</option>{#each profiles as p (p.id)}<option
					value={p.id}>{p.name}</option
				>{/each}</select
		></label
	>
	<div class="actions">
		<button disabled={locked || busy || !selected || !isTauri()} onclick={load}
			>{$t.usability.loadProfile}</button
		><button disabled={locked || busy || !selected} onclick={remove}
			>{confirming ? $t.usability.confirmDelete : $t.usability.deleteProfile}</button
		>{#if confirming}<button onclick={() => (confirming = false)} disabled={busy}
				>{$t.history.cancel}</button
			>{/if}
	</div>
	<p role="status">{notice}</p>
	{#if error}<p role="alert">{error}</p>{/if}
</details>

<style>
	.profiles {
		margin: 0.75rem 0;
		padding: 0.75rem;
		border-block: 1px solid var(--border);
		font-size: var(--type-12);
		color: var(--text-soft);
	}
	summary {
		cursor: pointer;
		font-weight: 600;
	}
	p {
		color: var(--muted);
		line-height: 1.5;
		overflow-wrap: anywhere;
	}
	label {
		display: grid;
		gap: 0.4rem;
		margin: 0.75rem 0;
	}
	input,
	select,
	button {
		min-width: 0;
		max-width: 100%;
		padding: 0.5rem;
		font: inherit;
		color: var(--text);
		background: var(--panel-2);
		border: 1px solid var(--border);
		border-radius: 6px;
	}
	.actions {
		display: flex;
		gap: 0.5rem;
		flex-wrap: wrap;
	}
	button:disabled {
		opacity: 0.5;
	}
</style>
