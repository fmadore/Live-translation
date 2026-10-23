<script lang="ts">
	import ModeStep from './setup/ModeStep.svelte';
	import SourceStep from './setup/SourceStep.svelte';
	import LanguageStep from './setup/LanguageStep.svelte';
	import EngineStep from './setup/EngineStep.svelte';
	import type { SetupActions } from './setupActions';

	/** The idle rail: the numbered setup sheet — what to show, where from, in which language
	 *  and with which engine, one component per step under `setup/`. The source and engine
	 *  steps read the pre-flight controller from the page's context. */
	let {
		actions,
		locked,
		browserMode,
		usesMic,
		usesSystem,
		languageError
	}: {
		actions: SetupActions;
		/** A session, a start or a profile load is under way. */
		locked: boolean;
		browserMode: boolean;
		usesMic: boolean;
		usesSystem: boolean;
		/** Why the chosen caption language cannot start, or empty. */
		languageError: string;
	} = $props();
</script>

<ModeStep {actions} {locked} />

<div class="divider"></div>

<SourceStep {actions} {locked} {browserMode} {usesMic} {usesSystem} />

<div class="divider"></div>

<LanguageStep {actions} {locked} {languageError} />

<div class="divider"></div>

<EngineStep {actions} {locked} {browserMode} />
