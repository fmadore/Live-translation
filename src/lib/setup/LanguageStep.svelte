<script lang="ts">
	import GlossaryList from '../GlossaryList.svelte';
	import LanguagePicker from '../LanguagePicker.svelte';
	import whisperLanguages from '../whisperLanguages.json';
	import Kbd from '../ui/Kbd.svelte';
	import Field from '../ui/Field.svelte';
	import LanguageCard from '../ui/LanguageCard.svelte';
	import Select from '../ui/Select.svelte';
	import { displayNames, languageName } from '../languages';
	import { locale, t } from '../i18n';
	import { languageFavourites, options } from '../stores';
	import { isLocalWhisper, providerDetectsLanguage } from '../providers';
	import type { TargetLanguage } from '../types';
	import type { SetupActions } from '../setupActions';

	/** Step 03: the caption language, the language spoken in the room, or the demo's script —
	 *  whichever the chosen mode and engine ask for. */
	let {
		actions,
		locked,
		languageError
	}: {
		actions: SetupActions;
		locked: boolean;
		/** Why the chosen caption language cannot start, or empty. */
		languageError: string;
	} = $props();

	// Both caption-language pickers share one list of favourites.
	function pin(code: TargetLanguage) {
		languageFavourites.update((pins) =>
			pins.includes(code) ? pins.filter((p) => p !== code) : [...pins, code]
		);
	}

	// This step asks which language to render into, which demo script to play, or nothing when
	// the backend detects the spoken language itself (Voxtral then takes names and terms).
	const spokenChoices = $derived.by(() => {
		const names = displayNames($locale, 'none');
		return whisperLanguages
			.map((l) => ({ code: l.code, name: names?.of(l.code) || l.name }))
			.sort((a, b) => a.name.localeCompare(b.name, $locale));
	});
	const languageStepTitle = $derived(
		$options.mode === 'translate'
			? $t.rail.step.roomReads
			: $options.provider === 'ondevice'
				? $t.rail.step.demoLanguage
				: $t.rail.step.spokenLanguage
	);
</script>

<section class="rail-section">
	<div class="step-head">
		<span class="step-no">03</span>
		<h2 class="kicker">{languageStepTitle}</h2>
	</div>
	{#snippet spokenLanguage(hint: string)}
		<Field label={$t.whisper.language}>
			<Select
				value={$options.spokenLanguage ?? ''}
				disabled={locked}
				onchange={(e) =>
					($options = { ...$options, spokenLanguage: e.currentTarget.value || null })}
			>
				<option value="">{$t.whisper.auto}</option>
				{#each spokenChoices as language (language.code)}
					<option value={language.code}>{language.name}</option>
				{/each}
			</Select>
		</Field>
		<p class="hint">{hint}</p>
	{/snippet}
	{#if $options.provider === 'whisper'}
		{@render spokenLanguage($t.whisper.languageHint)}
	{:else if $options.mode === 'transcribe' && providerDetectsLanguage($options.provider)}
		<p class="hint">{$t.rail.autoDetectHint($t.engine[$options.provider])}</p>
		<!-- Only Voxtral takes a list of names and terms. The list stays in the setup with any
		     other engine, so choosing Voxtral again brings it back. -->
		{#if $options.provider === 'mistral'}<GlossaryList {locked} />{/if}
	{:else if $options.mode === 'translate'}
		<LanguagePicker
			value={$options.targetLanguage}
			provider={$options.provider}
			favourites={$languageFavourites}
			disabled={locked}
			error={languageError}
			onchange={(code) => code && actions.setTarget(code)}
			onpin={pin}
		/>
		{#if isLocalWhisper($options.provider)}
			<!-- English is the only caption language, so there is nothing to flip to and no
			     second language to offer: say so rather than show an empty picker. Whisper still
			     has to hear the spoken language, which it translates from. -->
			<p class="hint">{$t.language.englishOnly}</p>
			{@render spokenLanguage($t.whisper.translateLanguageHint)}
		{:else}
			<p class="hint inline-hint">
				<span>{$t.rail.flipHint}</span><Kbd command="direction" />
			</p>
			<!-- The same searchable picker as the first language, not a <label>: it holds pin
			     buttons as well as its field, and names the field itself. -->
			<div class="ui-field">
				<span aria-hidden="true">{$t.language.second}</span>
				<LanguagePicker
					value={$options.secondTargetLanguage ?? null}
					provider={$options.provider}
					favourites={$languageFavourites}
					disabled={locked}
					label={$t.language.second}
					none={$t.language.secondNone}
					exclude={$options.targetLanguage}
					onchange={actions.setSecondTarget}
					onpin={pin}
				/>
			</div>
			<p class="hint">{$t.language.secondHint}</p>
		{/if}
	{:else}
		{#if languageError}<p class="hint" role="status">{languageError}</p>{/if}
		<div class="lang-cards">
			<LanguageCard
				code="EN"
				name={languageName('en', $locale)}
				selected={$options.targetLanguage === 'en'}
				disabled={locked}
				onclick={() => actions.setTarget('en')}
			/>
			<LanguageCard
				code="FR"
				name={languageName('fr', $locale)}
				selected={$options.targetLanguage === 'fr'}
				disabled={locked}
				onclick={() => actions.setTarget('fr')}
			/>
		</div>
		<p class="hint">{$t.rail.demoLanguageHint}</p>
	{/if}
</section>

<style>
	/* Direct children of the rail's scrolling column keep their height rather than
	   compress, so a short window scrolls instead of clipping text. */
	.kicker {
		flex: 0 0 auto;
	}

	.lang-cards {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: var(--space-2);
	}
</style>
