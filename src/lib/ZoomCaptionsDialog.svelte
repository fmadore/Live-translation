<script lang="ts">
	// Captions in Zoom: paste the meeting's caption link, choose what to share, start, stop and
	// test. Opened from the toolbar, idle or mid-session, because sharing is independent of the
	// caption session; what it shows comes from the controller the page owns
	// (`zoomCaptions.svelte.ts`), so closing the dialog forgets nothing.

	import { tick } from 'svelte';
	import ModalPrompt from './ModalPrompt.svelte';
	import Field from './ui/Field.svelte';
	import Preference from './ui/Preference.svelte';
	import Select from './ui/Select.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import { describeError } from './errors';
	import { locale, t } from './i18n';
	import { languageName } from './languages';
	import { options } from './stores';
	import { LANES, ORIGINS, laneCount, laneLanguage, type Lane } from './types';
	import { openZoomGuide, ZOOM_GUIDE_URL, type ZoomCaptions } from './zoomCaptions.svelte';

	let { zoom, onClose }: { zoom: ZoomCaptions; onClose: () => void } = $props();

	let guideError = $state<string | null>(null);

	// A real link, so it reads as one and shows its address on hover, but the click goes to the
	// core, which opens the guide in the default browser rather than in a WebView2 popup.
	async function openGuide(event: MouseEvent) {
		event.preventDefault();
		guideError = null;
		try {
			await openZoomGuide();
		} catch (e) {
			guideError = describeError(e, $t);
		}
	}

	const uid = $props.id();
	const hintId = `${uid}-hint`;

	let linkEl = $state<HTMLInputElement | null>(null);
	let startEl = $state<HTMLButtonElement | null>(null);
	let testEl = $state<HTMLButtonElement | null>(null);

	const status = $derived(zoom.status);
	const active = $derived(zoom.active);
	// Changing what is shared means stopping first: the core takes all three together.
	const locked = $derived(active || zoom.pending !== null);

	// What a start would share: the operator's choice where the setup offers one, otherwise its
	// only caption language and its only source.
	const twoLanes = $derived(laneCount($options) === 2);
	const lane = $derived<Lane>(twoLanes ? zoom.lane : 0);
	const origins = $derived($options.source === 'both' ? zoom.origins : [$options.source]);
	// While sharing, the fields say what is being shared, which a setup changed since Start may
	// no longer match; so the choice also shows whenever that differs from the setup's default.
	const shownLane = $derived(active ? status.lane : lane);
	const shownOrigins = $derived(active ? status.origins : origins);
	const showLanguage = $derived(twoLanes || shownLane === 1);
	const showSources = $derived(
		$options.source === 'both' || shownOrigins.length !== 1 || shownOrigins[0] !== $options.source
	);
	const ready = $derived(zoom.link.trim() !== '' && origins.length > 0);

	function laneLabel(choice: Lane): string {
		const code = laneLanguage($options, choice);
		if (code) return languageName(code, $locale);
		return choice === 1 ? $t.language.second : $t.language.auto;
	}

	// Counts change with every caption, so they stay out of the live region: it speaks when the
	// state changes, never as a number climbs.
	const counts = $derived(
		active
			? [
					status.state === 'sharing' || status.sent > 0 ? $t.zoom.sent(status.sent) : '',
					status.dropped > 0 ? $t.zoom.dropped(status.dropped) : ''
				]
					.filter(Boolean)
					.join(' ')
			: ''
	);
	const tone = $derived(
		{ off: '', sharing: 'live', waiting: 'warn', unreachable: 'warn', refused: 'bad' }[status.state]
	);

	// Start, and Stop with Send test caption, swap places as sharing begins and ends, taking the
	// button the keyboard was on with them. Focus that fell to the page goes to the next thing
	// to do: test the captions once sharing, fix the link after a failed start, start again
	// after stopping.
	async function handOff(target: HTMLElement | null) {
		await tick();
		const lost = document.activeElement === null || document.activeElement === document.body;
		if (lost) (target && !target.matches(':disabled') ? target : linkEl)?.focus();
	}

	async function start() {
		if (!ready || zoom.pending) return;
		await zoom.start(lane, origins);
		await handOff(zoom.active ? testEl : linkEl);
	}

	async function stop() {
		await zoom.stop();
		await handOff(startEl);
	}
</script>

<ModalPrompt title={$t.zoom.title} onDismiss={onClose} dismissLabel={$t.zoom.close}>
	<p>{$t.zoom.intro}</p>
	<ol class="steps">
		<li>{$t.zoom.steps.settings}</li>
		<li>{$t.zoom.steps.meeting}</li>
		<li>{$t.zoom.steps.paste}</li>
	</ol>
	<!-- The host's part is several menus deep in Zoom's web portal, outside this app: the guide
	     walks through it. -->
	<p class="guide">
		<a href={ZOOM_GUIDE_URL} onclick={openGuide}
			>{$t.zoom.guide}<span class="sr-only">{$t.support.opensInBrowser}</span></a
		>
		{#if guideError}<span class="failed" role="alert">{guideError}</span>{/if}
	</p>

	<form
		class="share"
		onsubmit={(event) => {
			event.preventDefault();
			void start();
		}}
	>
		<div class="link">
			<!-- Masked: the link lets anyone post captions to the meeting, and this screen may be
			     on a projector. Never bound to anything that persists. -->
			<Field label={$t.zoom.link}
				><input
					bind:this={linkEl}
					bind:value={zoom.link}
					type="password"
					autocomplete="off"
					autocapitalize="off"
					autocorrect="off"
					spellcheck="false"
					aria-describedby={hintId}
					disabled={active}
					readonly={zoom.pending !== null}
				/></Field
			>
			<p class="hint" id={hintId}>{$t.zoom.linkHint}</p>
		</div>

		{#if showLanguage}
			<Field label={$t.zoom.language}>
				<Select
					bind:value={() => shownLane, (choice: Lane) => (zoom.lane = choice === 1 ? 1 : 0)}
					disabled={locked}
				>
					{#each LANES as choice (choice)}<option value={choice}>{laneLabel(choice)}</option>{/each}
				</Select>
			</Field>
		{/if}

		{#if showSources}
			<fieldset>
				<legend>{$t.zoom.sources}</legend>
				{#each ORIGINS as origin (origin)}
					<Preference
						label={$t.zoom.origin[origin]}
						disabled={locked}
						bind:checked={
							() => shownOrigins.includes(origin), (shared) => zoom.setOrigin(origin, shared)
						}
					/>
				{/each}
			</fieldset>
		{/if}

		<p class="note">{$t.zoom.privacy}</p>

		<div class="status {tone}">
			<p role="status">{$t.zoom.status[status.state]}</p>
			{#if counts}<p class="hint">{counts}</p>{/if}
		</div>
		{#if zoom.error !== null}
			<p class="error" role="alert">{describeError(zoom.error, $t)}</p>
		{/if}

		<!-- A press while an answer is pending is ignored rather than the button disabled, so the
		     keyboard keeps the button it is on until the swap above hands it on. -->
		<div class="actions">
			{#if active}
				<ToolButton
					size="lg"
					bind:element={testEl}
					aria-busy={zoom.pending === 'test'}
					onclick={() => void zoom.sendTest($t.zoom.testCaption)}>{$t.zoom.test}</ToolButton
				>
				<ToolButton
					variant="ghost"
					size="lg"
					aria-busy={zoom.pending === 'stop'}
					onclick={() => void stop()}>{$t.zoom.stop}</ToolButton
				>
			{:else}
				<ToolButton
					type="submit"
					variant="primary"
					size="lg"
					bind:element={startEl}
					disabled={!ready}
					aria-busy={zoom.pending === 'start'}
					>{zoom.pending === 'start' ? $t.preflight.start.starting : $t.zoom.start}</ToolButton
				>
			{/if}
		</div>
	</form>
</ModalPrompt>

<style>
	.steps {
		display: grid;
		gap: var(--space-1);
		margin: 0;
		padding-left: var(--space-5);
		font-size: var(--type-body);
		line-height: var(--leading-body);
		color: var(--text-secondary);
	}
	/* Underlined, not only coloured, so it reads as a link without relying on colour. */
	.guide {
		margin: 0;
		font-size: var(--type-body);
		line-height: var(--leading-body);
	}
	.guide a {
		color: var(--accent-soft);
		text-decoration: underline;
		text-underline-offset: 0.15em;
	}
	.guide a:hover {
		color: var(--text-bright);
	}
	.failed {
		display: block;
		font-size: var(--type-small);
		color: var(--warn);
	}
	.share {
		display: grid;
		gap: var(--space-3);
		min-width: 0;
	}
	.link {
		display: grid;
		gap: var(--space-1);
		min-width: 0;
	}
	/* The link's field fills the row: a token is a long opaque string, and seeing that the
	   whole of it was pasted matters more than the field's proportions. */
	.link :global(input) {
		width: 100%;
	}
	.link :global(input:focus) {
		border-color: var(--accent-border);
	}
	fieldset {
		display: grid;
		gap: var(--space-2);
		min-width: 0;
		margin: 0;
		padding: 0;
		border: 0;
	}
	/* Worded and sized as the label over a field, which is what it is for the group. */
	legend {
		margin-bottom: var(--space-2);
		padding: 0;
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	/* The state in words, on the wash of its colour: the pill's language, so a refused link
	   reads as the same kind of trouble as a session error. */
	.status {
		display: grid;
		gap: var(--space-1);
		padding: var(--space-2) var(--space-3);
		border: 1px solid var(--line-strong);
		border-radius: var(--radius-control);
		background: var(--surface-2);
	}
	.status p[role='status'] {
		color: var(--text-body);
	}
	.status.live {
		border-color: var(--accent-border);
		background: var(--accent-wash);
	}
	.status.warn {
		border-color: var(--warn-border);
		background: var(--warn-wash);
	}
	.status.bad {
		border-color: var(--danger-border);
		background: var(--danger-wash);
	}
</style>
