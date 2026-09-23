<script lang="ts">
	import { t } from './i18n';
	import {
		options,
		overlayHoldSeconds,
		overlayPace,
		overlayCaptionLayout,
		overlayCleanSpeech,
		overlayCaptionWidth
	} from './stores';
	import type { OverlayController } from './overlayController.svelte';
	import type { CaptionLayout } from './captionLayout';
	import Field from './ui/Field.svelte';
	import Select from './ui/Select.svelte';
	import Stepper from './ui/Stepper.svelte';
	import Preference from './ui/Preference.svelte';
	let { overlay }: { overlay: OverlayController } = $props();
</script>

<div class="reading">
	<Field label={$t.overlayControls.captionLayout}>
		<Select
			value={$overlayCaptionLayout}
			onchange={(e) => overlay.setCaptionLayout(e.currentTarget.value as CaptionLayout)}
		>
			<option value="fit">{$t.overlayControls.fitWindow}</option><option value="stable"
				>{$t.overlayControls.stable}</option
			><option value="compact">{$t.overlayControls.compact}</option>
		</Select>
	</Field>
	{#if $overlayCaptionLayout === 'compact'}<Stepper
			label={$t.overlayControls.captionWidth}
			value={$overlayCaptionWidth}
			min={20}
			max={60}
			step={2}
			unit="ch"
			onchange={overlay.setCaptionWidth}
		/>{/if}
	<Stepper
		label={$t.usability.hold}
		value={$overlayHoldSeconds}
		min={2}
		max={30}
		unit="s"
		disabled={$overlayCaptionLayout === 'stable'}
		onchange={overlay.setHoldSeconds}
	/>
	<p>{$t.usability.holdHint}</p>
	<Field label={$t.usability.pace}>
		<Select
			value={$overlayPace}
			onchange={(e) => overlay.setPace(e.currentTarget.value === 'steady' ? 'steady' : 'immediate')}
			><option value="immediate">{$t.usability.immediate}</option><option value="steady"
				>{$t.usability.steady}</option
			></Select
		>
	</Field>
	<p>{$t.usability.paceHint}</p>
	<Preference
		label={$t.overlayControls.cleanSpeech}
		note={$t.overlayControls.cleanSpeechHint}
		checked={$overlayCleanSpeech}
		onchange={overlay.setCleanSpeech}
	/>
	{#if $options.provider === 'gemini-transcribe'}<p>{$t.usability.geminiSmart}</p>{/if}
</div>

<style>
	.reading {
		display: grid;
		gap: 0.75rem;
	}
	p {
		margin: 0 0 0.5rem;
		font-size: var(--type-small);
		line-height: 1.5;
		color: var(--muted);
	}
</style>
