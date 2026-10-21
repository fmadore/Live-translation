<script lang="ts">
	import { tick } from 'svelte';
	import { t } from './i18n';
	import { options } from './stores';
	import {
		addGlossaryTerm,
		MAX_GLOSSARY_TERM_LENGTH,
		MAX_GLOSSARY_TERMS,
		removeGlossaryTerm,
		type GlossaryProblem
	} from './glossary';
	import Field from './ui/Field.svelte';
	import ToolButton from './ui/ToolButton.svelte';

	/** Voxtral's names and terms: shown, added one at a time and removed. Part of the setup, so
	 *  saved with it and in profiles, and sent once at Start: nothing changes mid-session. */
	let { locked }: { locked: boolean } = $props();
	const id = $props.id();
	const m = $derived($t.rail.glossary);
	const glossary = $derived($options.glossary ?? []);

	let draft = $state('');
	let problem = $state<{ kind: Exclude<GlossaryProblem, 'blank'>; term: string } | null>(null);
	let announcement = $state('');
	let chips = $state<HTMLUListElement>();
	let field = $state<HTMLInputElement>();

	const message = $derived.by(() => {
		if (!problem) return '';
		const text = m.problem;
		if (problem.kind === 'length') return text.length(MAX_GLOSSARY_TERM_LENGTH);
		if (problem.kind === 'full') return text.full(MAX_GLOSSARY_TERMS);
		return text.duplicate(problem.term);
	});

	/** Said once by the status region. Cleared first, so removing, re-adding and removing the
	 *  same term is announced every time rather than only the first. */
	async function announce(text: string) {
		announcement = '';
		await tick();
		announcement = text;
	}

	function save(list: string[]) {
		$options = { ...$options, glossary: list };
	}

	function add(event: SubmitEvent) {
		event.preventDefault();
		if (locked) return;
		const result = addGlossaryTerm(glossary, draft);
		if ('problem' in result) {
			problem = result.problem === 'blank' ? null : { kind: result.problem, term: draft.trim() };
			return;
		}
		save(result.list);
		draft = '';
		problem = null;
		void announce(m.added(result.term));
	}

	async function remove(term: string, index: number) {
		if (locked) return;
		save(removeGlossaryTerm(glossary, term));
		problem = null;
		void announce(m.removed(term));
		// The button that had focus is gone. Keep the keyboard in the list: on the term that
		// took its place, the one before it at the end, or the field once the list is empty.
		await tick();
		const buttons = chips?.querySelectorAll('button');
		(buttons?.[Math.min(index, buttons.length - 1)] ?? field)?.focus();
	}
</script>

<div class="glossary" role="group" aria-labelledby="{id}-heading">
	<span class="heading" id="{id}-heading">{m.heading}</span>
	<p class="note">{m.hint}</p>
	{#if glossary.length}
		<ul bind:this={chips} aria-labelledby="{id}-heading">
			{#each glossary as term, index (term.toLowerCase())}
				<li>
					<ToolButton
						size="sm"
						disabled={locked}
						aria-label={m.remove(term)}
						title={m.remove(term)}
						onclick={() => void remove(term, index)}
						><span class="term">{term}</span><svg
							width="10"
							height="10"
							viewBox="0 0 10 10"
							fill="none"
							stroke="currentColor"
							stroke-width="1.6"
							stroke-linecap="round"
							aria-hidden="true"><path d="M2 2l6 6M8 2l-6 6" /></svg
						></ToolButton
					>
				</li>
			{/each}
		</ul>
	{:else}
		<p class="note">{m.empty}</p>
	{/if}
	<form onsubmit={add}>
		<Field label={m.add}
			><input
				bind:this={field}
				bind:value={draft}
				disabled={locked}
				autocomplete="off"
				spellcheck="false"
				aria-invalid={problem ? 'true' : undefined}
				aria-describedby={problem ? `${id}-problem` : undefined}
				oninput={() => (problem = null)}
			/></Field
		>
		<ToolButton type="submit" disabled={locked || !draft.trim()}>{m.addButton}</ToolButton>
	</form>
	{#if problem}<p class="problem" id="{id}-problem" role="alert">{message}</p>{/if}
	<p class="sr-only" role="status">{announcement}</p>
</div>

<style>
	.glossary {
		display: grid;
		gap: var(--space-2);
		min-width: 0;
		margin-top: var(--space-2);
		font-size: var(--type-small);
		color: var(--text-secondary);
	}
	.heading {
		font-weight: 600;
	}
	.note {
		margin: 0;
		line-height: var(--leading-body);
		color: var(--text-muted);
	}
	ul {
		display: flex;
		flex-wrap: wrap;
		gap: var(--space-2);
		margin: 0;
		padding: 0;
		list-style: none;
	}
	/* A term can be a long institution's name: it wraps inside its chip rather than widen it. */
	li {
		min-width: 0;
		max-width: 100%;
	}
	.term {
		min-width: 0;
		overflow-wrap: anywhere;
		line-height: var(--leading-tight);
		text-align: start;
	}
	/* The remove mark keeps its size beside a term that wraps. */
	li svg {
		flex: none;
	}
	form {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: end;
		gap: var(--space-2);
	}
	.problem {
		margin: 0;
		line-height: var(--leading-body);
		color: var(--danger-soft);
	}
</style>
