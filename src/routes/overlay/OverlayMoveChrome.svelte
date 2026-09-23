<script lang="ts">
	import { t } from '$lib/i18n';
	import ToolButton from '$lib/ui/ToolButton.svelte';

	/** Move mode's placement chrome. The overlay window *is* the caption region, so the chrome
	 *  hugs the window edges rather than being drawn inside a larger screen. Everything here is
	 *  pointer-events:none except the toolbar, so the stage behind stays the drag region. */
	let {
		fontSize,
		width,
		height,
		onBump,
		onSnap,
		onLock
	}: {
		fontSize: number;
		/** The window's own size, which is the caption region's. */
		width: number;
		height: number;
		onBump: (delta: number) => void;
		onSnap: () => void;
		onLock: () => void;
	} = $props();
</script>

<div class="region" aria-hidden="true">
	<span class="handle tl"></span>
	<span class="handle tr"></span>
	<span class="handle bl"></span>
	<span class="handle br"></span>
	<span class="edge top"></span>
	<span class="edge bottom"></span>
</div>

<!-- Dropped in a short region: there the chrome fills the window and the placeholder would
     run under the toolbar, which reads worse than no placeholder at all. -->
{#if height >= 340}
	<p class="placeholder">{$t.overlay.placeholder(fontSize)}</p>
{/if}

<div class="chrome">
	<div class="drag-pill">
		<svg width="13" height="13" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
			<circle cx="9" cy="6" r="1.6" />
			<circle cx="15" cy="6" r="1.6" />
			<circle cx="9" cy="12" r="1.6" />
			<circle cx="15" cy="12" r="1.6" />
			<circle cx="9" cy="18" r="1.6" />
			<circle cx="15" cy="18" r="1.6" />
		</svg>
		<span class="drag-label">{$t.overlay.dragToPlace}</span>
		<span class="drag-size">{width} × {height}</span>
	</div>

	<div class="toolbar">
		<div class="mode">
			<span class="mode-title">{$t.overlay.moveMode}</span>
			<span class="mode-sub">{$t.overlay.paused}</span>
			<!-- The operator's own controls can be hidden under this window, so the way out has
			     to be printed where the operator is already looking. -->
			<span class="keys">
				<kbd>{$t.overlay.keyEnter}</kbd>
				{$t.overlay.keysLocks} · <kbd>{$t.overlay.keyEscape}</kbd>
				{$t.overlay.keysCancels} · <kbd>{$t.overlay.keyArrows}</kbd>
				{$t.overlay.keysNudge}
			</span>
		</div>
		<span class="divider"></span>
		<div class="size">
			<span class="size-label">{$t.overlay.size}</span>
			<button class="step" onclick={() => onBump(-2)} aria-label={$t.overlay.smaller}>−</button>
			<span class="size-value">{fontSize}</span>
			<button class="step" onclick={() => onBump(2)} aria-label={$t.overlay.larger}>+</button>
		</div>
		<span class="divider"></span>
		<ToolButton onclick={onSnap}>{$t.overlay.snapToBottom}</ToolButton>
		<ToolButton variant="primary" onclick={onLock}>
			<svg
				width="13"
				height="13"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
				stroke-linecap="round"
				aria-hidden="true"
			>
				<rect x="4.5" y="10.5" width="15" height="10" rx="2.5" />
				<path d="M8 10.5V8a4 4 0 0 1 8 0v2.5" />
			</svg>
			{$t.overlay.lock}
		</ToolButton>
	</div>
</div>

<style>
	.region {
		position: absolute;
		inset: 0;
		border: 2px solid var(--accent);
		background: var(--accent-bg);
		pointer-events: none;
	}
	/* Affordances only: the resize itself is the OS window edge-drag. */
	.handle {
		position: absolute;
		width: 11px;
		height: 11px;
		border-radius: 3px;
		background: var(--accent);
	}
	.handle.tl {
		left: 3px;
		top: 3px;
	}
	.handle.tr {
		right: 3px;
		top: 3px;
	}
	.handle.bl {
		left: 3px;
		bottom: 3px;
	}
	.handle.br {
		right: 3px;
		bottom: 3px;
	}
	.edge {
		position: absolute;
		left: 50%;
		transform: translateX(-50%);
		width: 34px;
		height: 9px;
		border-radius: 3px;
		background: color-mix(in srgb, var(--accent) 55%, transparent);
	}
	.edge.top {
		top: 3px;
	}
	.edge.bottom {
		bottom: 3px;
	}
	/* Stands in for a caption while the region is being placed, so it previews the chosen
	   face and size together — which is the moment an operator can still act on either. */
	.placeholder {
		position: absolute;
		inset: 0;
		font-family: var(--caption-face);
		display: grid;
		place-items: center;
		margin: 0;
		padding: 0 34px;
		font-weight: 600;
		/* Never larger than the caption it stands in for, and never so large it wraps to
		   nothing in a short region. */
		font-size: min(34px, var(--fs));
		line-height: 1.3;
		text-align: center;
		text-wrap: pretty;
		color: rgba(255, 255, 255, 0.55);
		pointer-events: none;
	}

	/* The pill and toolbar float just inside the top edge: in the real window there is no
	   surrounding screen to hang them on. */
	.chrome {
		position: absolute;
		top: 14px;
		left: 50%;
		transform: translateX(-50%);
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 12px;
		pointer-events: none;
	}
	/* Stays transparent to the pointer so dragging it drags the window (the stage below
	   carries data-tauri-drag-region). */
	.drag-pill {
		display: flex;
		align-items: center;
		gap: 10px;
		padding: 6px 12px;
		border-radius: 8px;
		background: var(--accent);
		color: var(--on-accent);
		pointer-events: none;
	}
	.drag-label {
		font-weight: 600;
		font-size: var(--type-small);
		line-height: 1;
	}
	.drag-size {
		font-family: var(--font-mono);
		font-weight: 500;
		font-size: var(--type-small);
		line-height: 1;
		font-variant-numeric: tabular-nums;
		opacity: 0.72;
	}

	.toolbar {
		display: flex;
		align-items: center;
		gap: 14px;
		padding: 12px 14px;
		border: 1px solid var(--line-hover);
		border-radius: 14px;
		/* Nearly opaque, because what sits behind this window is a slide nobody controls: at
		   0.92 a white slide lifted the panel enough to cost the dimmest text its 4.5:1. At
		   0.96 the panel over white stays darker than --surface-2, so every text level that
		   passes there passes here; `palette.test.ts` holds it to that. */
		background: color-mix(in srgb, var(--surface-0) 96%, transparent);
		box-shadow: 0 24px 60px -20px rgba(0, 0, 0, 0.8);
		color: var(--text-body);
		/* Clickable while the rest of the stage drags the window. */
		pointer-events: auto;
		cursor: default;
	}
	.mode {
		display: flex;
		flex-direction: column;
		gap: 3px;
		padding-right: 4px;
	}
	.mode-title {
		font-weight: 600;
		font-size: var(--type-caption);
		line-height: 1;
		letter-spacing: 0.14em;
		text-transform: uppercase;
		color: var(--warn);
	}
	.mode-sub {
		font-size: var(--type-small);
		line-height: 1;
		color: var(--text-muted);
	}
	.keys {
		margin-top: 3px;
		font-family: var(--font-mono);
		font-size: var(--type-caption);
		line-height: 1.7;
		color: var(--text-muted);
		white-space: nowrap;
	}
	.keys kbd {
		padding: 3px 5px;
		border: 1px solid var(--line-strong);
		border-radius: 5px;
		background: var(--surface-2);
		font-family: inherit;
		font-weight: 500;
		font-size: inherit;
		color: var(--text-secondary);
	}
	.divider {
		width: 1px;
		height: 30px;
		background: var(--line-strong);
	}
	.size {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.size-label {
		font-size: var(--type-small);
		line-height: 1;
		color: var(--text-muted);
	}
	.size-value {
		min-width: 24px;
		font-family: var(--font-mono);
		font-weight: 500;
		font-size: var(--type-body);
		line-height: 1;
		font-variant-numeric: tabular-nums;
		text-align: center;
	}
	.step {
		display: grid;
		place-items: center;
		width: 28px;
		height: 28px;
		border: 1px solid var(--line-strong);
		border-radius: 7px;
		background: var(--surface-2);
		color: var(--text-secondary);
		font-weight: 500;
		font-size: var(--type-body);
		line-height: 1;
	}
	.step:hover {
		border-color: var(--line-hover);
		color: var(--text-body);
	}

	/* Windows contrast themes. The placement preview keeps its own colours, like the audience
	   view it stands for; the toolbar, which is chrome, keeps the system palette (Lock drops its
	   gradient in `app.css`, with every other primary). */
	@media (forced-colors: active) {
		.placeholder,
		.region,
		.handle,
		.edge {
			forced-color-adjust: none;
		}
	}
</style>
