<script module lang="ts">
	export type SettingsTab = 'captions' | 'reading' | 'history' | 'app';
</script>

<script lang="ts">
	import ModalPrompt from './ModalPrompt.svelte';
	import Tabs from './ui/Tabs.svelte';
	import LanguageCard from './ui/LanguageCard.svelte';
	import Preference from './ui/Preference.svelte';
	import ToolButton from './ui/ToolButton.svelte';
	import CaptionAppearance from './CaptionAppearance.svelte';
	import OverlayMoveButton from './OverlayMoveButton.svelte';
	import ReadingPreferences from './ReadingPreferences.svelte';
	import TranscriptHistory from './TranscriptHistory.svelte';
	import KeyboardHelp from './KeyboardHelp.svelte';
	import { LOCALE_NAMES, LOCALES, locale, setLocale, t } from './i18n';
	import { closeToTray } from './stores';
	import { THEMES, theme } from './theme';

	/** The settings panel. Everything in it is persisted and applies live, so there is no
	 *  draft to keep and nothing to cancel — closing is the only exit it needs. */
	let {
		browserMode,
		tab = $bindable('captions'),
		onHideWindow,
		onClose
	}: {
		browserMode: boolean;
		/** Bound, so reopening the panel returns to the tab it was closed on. */
		tab?: SettingsTab;
		onHideWindow: () => void;
		onClose: () => void;
	} = $props();

	const tabs = $derived<readonly { id: SettingsTab; label: string }[]>([
		{ id: 'captions', label: $t.design.captions },
		{ id: 'reading', label: $t.design.reading },
		{ id: 'history', label: $t.history.heading },
		{ id: 'app', label: $t.design.app }
	]);

	// Names the interface-language group for a screen reader; the heading is the only thing
	// that says what those buttons are choosing between.
	const idBase = $props.id();
	const localeHeadingId = `${idBase}-locale`;
	const themeHeadingId = `${idBase}-theme`;
</script>

<!-- Modal for the focus handling rather than because it demands an answer, so Escape and
     Close are the same harmless exit. It is deliberately reachable while a session runs: the
     appearance controls in the rail and the ones in here are the same controls over the same
     stores, and an operator who opens this mid-session to raise the caption size should get
     exactly that. -->
<ModalPrompt
	wide
	stableHeight
	title={$t.settings.heading}
	dismissLabel={$t.settings.closeLabel}
	onDismiss={onClose}
>
	<p class="hint">{$t.design.applies}</p>
	<Tabs
		{tabs}
		bind:selected={tab}
		label={$t.settings.heading}
		idPrefix="settings"
		panelId="settings-panel"
	/>
	{#key tab}
		<div
			class="settings"
			id="settings-panel"
			role="tabpanel"
			aria-labelledby={`settings-${tab}`}
			tabindex="0"
		>
			{#if tab === 'captions'}
				<div class="rail-section">
					<CaptionAppearance heading={$t.settings.appearance} />
					<p class="hint">{$t.settings.appearanceNote}</p>
					<!-- Placement mode is the preview: the overlay stands a sample caption in, set in
					     whatever is chosen above. Same button and same labels as the pre-flight
					     check, because it is the same thing being done. -->
					<OverlayMoveButton wording="place" wide disabled={browserMode} />
				</div>
			{:else if tab === 'reading'}
				<ReadingPreferences />
			{:else if tab === 'history'}
				<TranscriptHistory />
			{:else}
				<!-- What belongs to the app rather than to a session: none of these touches
				     capture, so all of them stay usable mid-session. -->
				<div class="divider"></div>
				<!-- The interface language, not the caption language. -->
				<div class="rail-section">
					<h2 class="kicker" id={localeHeadingId}>{$t.locale.label}</h2>
					<!-- Buttons rather than a select: with a handful of interface languages, both
					     choices fit on screen and the switch costs one click instead of two. Built
					     from LOCALES so another language needs no markup, and grouped under the
					     heading because `aria-pressed` buttons on their own do not say what they
					     are choosing between. -->
					<div class="locale-cards" role="group" aria-labelledby={localeHeadingId}>
						{#each LOCALES as code (code)}
							<LanguageCard
								code={code.toUpperCase()}
								name={LOCALE_NAMES[code]}
								selected={$locale === code}
								onclick={() => setLocale(code)}
							/>
						{/each}
					</div>
					<p class="hint">{$t.locale.note}</p>
				</div>

				<div class="divider"></div>
				<!-- The operator window only: the overlay's captions have their own palette. -->
				<div class="rail-section">
					<h2 class="kicker" id={themeHeadingId}>{$t.theme.label}</h2>
					<div class="theme-cards" role="group" aria-labelledby={themeHeadingId}>
						{#each THEMES as choice (choice)}
							<button
								class="theme-card ui-card"
								aria-pressed={$theme === choice}
								onclick={() => theme.set(choice)}
							>
								<svg
									width="16"
									height="16"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="1.75"
									stroke-linecap="round"
									stroke-linejoin="round"
									aria-hidden="true"
								>
									{#if choice === 'system'}
										<rect x="3" y="4" width="18" height="12" rx="2" /><path d="M8 20h8M12 16v4" />
									{:else if choice === 'light'}
										<circle cx="12" cy="12" r="4" /><path
											d="M12 2.5v2M12 19.5v2M2.5 12h2M19.5 12h2M5.3 5.3l1.4 1.4M17.3 17.3l1.4 1.4M5.3 18.7l1.4-1.4M17.3 6.7l1.4-1.4"
										/>
									{:else}
										<path d="M20 14.5A8 8 0 0 1 9.5 4a8 8 0 1 0 10.5 10.5z" />
									{/if}
								</svg>
								<span>{$t.theme[choice]}</span>
							</button>
						{/each}
					</div>
					<p class="hint">{$t.theme.note}</p>
				</div>

				<div class="divider"></div>
				<div class="rail-section">
					<h2 class="kicker">{$t.window.heading}</h2>
					<ToolButton wide disabled={browserMode} onclick={onHideWindow}>
						<svg
							width="14"
							height="14"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="1.75"
							stroke-linecap="round"
							stroke-linejoin="round"
							aria-hidden="true"
							><path d="M12 4v10" /><path d="M8.5 10.5L12 14l3.5-3.5" /><path
								d="M4.5 17.5h15"
							/></svg
						>
						{$t.window.minimizeToTray}
					</ToolButton>
					<Preference
						label={$t.window.keepRunning}
						note={browserMode ? $t.window.needsDesktop : $t.window.keepRunningNote}
						checked={$closeToTray}
						disabled={browserMode}
						onchange={(value) => closeToTray.set(value)}
					/>
				</div>
				<KeyboardHelp />
			{/if}
		</div>
	{/key}
</ModalPrompt>

<style>
	/* The panel is a column of the same `.rail-section` blocks the rail is built from, so all
	   it contributes is the rhythm between them. */
	.settings {
		display: flex;
		flex-direction: column;
		gap: var(--space-3);
		flex: 1 0 0;
		min-height: 0;
		overflow-y: auto;
		scrollbar-gutter: stable;
		padding: var(--space-1);
	}
	.settings > :global(*) {
		flex-shrink: 0;
	}
	.kicker {
		flex: 0 0 auto;
	}

	/* The language cards of the setup sheet, sized from however many interface languages
	   there are rather than from the two the caption step happens to offer. */
	.locale-cards {
		display: grid;
		grid-auto-flow: column;
		grid-auto-columns: 1fr;
		gap: var(--space-2);
	}
	/* The theme choice, laid out like the language cards above it: one row, equal columns. */
	.theme-cards {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: var(--space-2);
	}
	.theme-card {
		display: flex;
		align-items: center;
		gap: var(--space-2);
		padding: var(--space-3);
		font-size: var(--type-body);
		font-weight: 500;
		line-height: 1;
		color: var(--text-secondary);
	}
	.theme-card svg {
		flex: 0 0 auto;
		color: var(--text-muted);
	}
	.theme-card[aria-pressed='true'] {
		color: var(--text-body);
		font-weight: 600;
	}
	.theme-card[aria-pressed='true'] svg {
		color: var(--accent-soft);
	}
</style>
