<!--
	The spectrum visualizer (SPEC sections 4.4 and 9.5), drawn in the theme's
	own variant: lit segments behind glass, plain bars, or soft warm columns,
	following `docs/design/palet-preview.html`. Its hue follows the character
	of the sound while tone colour is on.

	While this is on screen the backend is told a spectrum is wanted; when it
	leaves, the analysis tap, the FFT and the frames all stop.
-->
<script lang="ts">
	import { toneFilter, watch } from '$lib/analysis.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import { activeTheme } from '$lib/theme/index.svelte';

	interface Props {
		/** Height of the visualizer in pixels. */
		height?: number;
	}

	const { height = 180 }: Props = $props();

	/** Lit segments per column, behind glass. */
	const SEGMENTS = 20;

	const variant = $derived(activeTheme()?.variants.spectrum ?? 'bar');
	const filter = $derived(toneFilter('spectrum'));

	// The frames stop while nothing plays, and the bands come back empty
	// while no spectrum is wanted; the panel then says so instead of
	// freezing on the last picture it saw.
	const levels = $derived(player.meter.bands.map((value) => value / 255));
	const held = $derived(player.meter.peaks.map((value) => value / 255));

	$effect(() => watch('spectrum'));
</script>

<div
	class="spectrum"
	data-variant={variant}
	style:--height="{height}px"
	style:--segments={SEGMENTS}
	style:filter
	role="img"
	aria-label={t('spectrum.label')}
>
	{#if levels.length === 0}
		<p class="rest label">{t('spectrum.rest')}</p>
	{:else}
		<div class="columns" style:--bands={levels.length}>
			{#each levels as level, index (index)}
				<div class="column">
					<i class="bar" style:--level="{(level * 100).toFixed(1)}%"></i>
					<i class="peak" style:--peak="{((held[index] ?? 0) * 100).toFixed(1)}%"></i>
				</div>
			{/each}
		</div>
	{/if}
</div>

<style>
	.spectrum {
		position: relative;
		height: var(--height);
		min-width: 0;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		overflow: hidden;
		/* Tone colour drifts rather than jumps (SPEC section 9.5). */
		transition: filter 400ms linear;
	}

	.rest {
		position: absolute;
		inset: 0;
		display: grid;
		place-items: center;
		margin: 0;
		font-size: 10.5px;
		color: var(--onsa-lit-ghost);
	}

	.columns {
		display: grid;
		grid-template-columns: repeat(var(--bands), minmax(0, 1fr));
		align-items: end;
		gap: 2px;
		height: 100%;
		padding: 6px;
	}

	.column {
		position: relative;
		height: 100%;
	}

	.bar {
		position: absolute;
		inset: 0;
		background: var(--onsa-lit-on);
		clip-path: inset(calc(100% - var(--level, 0%)) 0 0 0);
		transition: clip-path 70ms linear;
	}

	.peak {
		position: absolute;
		left: 0;
		right: 0;
		height: 2px;
		bottom: var(--peak, 0%);
		background: var(--onsa-lit-on);
		transition: bottom 110ms linear;
	}

	/* Behind glass a column is a stack of segments, and the dead ones stay
	   faintly visible (SPEC section 9.4). */
	.spectrum[data-variant='segment'] .column {
		background: var(--onsa-lit-ghost);
	}

	.spectrum[data-variant='segment'] .column,
	.spectrum[data-variant='segment'] .bar {
		mask: repeating-linear-gradient(
			to top,
			#000 0 calc(100% / var(--segments) - 2px),
			transparent calc(100% / var(--segments) - 2px) calc(100% / var(--segments))
		);
	}

	/* A flight display: plain lit bars over a faint well. */
	.spectrum[data-variant='bar'] .column {
		background: linear-gradient(to top, var(--onsa-lit-ghost), transparent);
	}

	.spectrum[data-variant='bar'] .bar {
		opacity: 0.85;
	}

	.spectrum[data-variant='bar'] .peak {
		background: var(--onsa-role-label);
	}

	/* Warm metal: the column fades in from below and the peak is a red mark. */
	.spectrum[data-variant='soft'] .bar {
		background: linear-gradient(
			to top,
			color-mix(in srgb, var(--onsa-lit-on) 55%, transparent),
			var(--onsa-lit-on)
		);
		border-radius: 2px 2px 0 0;
		opacity: 0.9;
	}

	.spectrum[data-variant='soft'] .peak {
		background: var(--onsa-role-clip);
	}

	:global(:root[data-onsa-glow='true']) .bar {
		filter: drop-shadow(0 0 4px var(--onsa-lit-glow));
	}

	@media (prefers-reduced-motion: reduce) {
		.spectrum {
			transition: none;
		}
	}
</style>
