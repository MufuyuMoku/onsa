<!--
	The spectrum visualizer (SPEC sections 4.4 and 9.5), drawn in the theme's
	own variant: lit segments behind glass, plain bars, or a soft curve. Its
	hue follows the character of the sound while tone colour is on.

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

	/** Lit segments per column in the segment variant. */
	const SEGMENTS = 18;

	const variant = $derived(activeTheme()?.variants.spectrum ?? 'bar');
	const bands = $derived(player.meter.bands);
	const peaks = $derived(player.meter.peaks);
	const filter = $derived(toneFilter('spectrum'));

	// The frames stop while nothing plays; the columns then rest at the floor.
	const levels = $derived(bands.map((value) => value / 255));
	const held = $derived(peaks.map((value) => value / 255));

	/** The soft variant draws one filled curve instead of columns. */
	const curve = $derived.by(() => {
		if (levels.length < 2) return '';
		const step = 100 / (levels.length - 1);
		const points = levels.map(
			(level, index) => `${(index * step).toFixed(2)} ${(100 - level * 100).toFixed(2)}`
		);
		return `M 0 100 L ${points.join(' L ')} L 100 100 Z`;
	});

	$effect(() => watch('spectrum'));
</script>

<div
	class="spectrum"
	data-variant={variant}
	style:--height="{height}px"
	style:filter
	role="img"
	aria-label={t('spectrum.label')}
>
	{#if levels.length === 0}
		<p class="rest label">{t('spectrum.rest')}</p>
	{:else if variant === 'soft'}
		<svg viewBox="0 0 100 100" preserveAspectRatio="none">
			<path class="fill" d={curve} />
		</svg>
	{:else}
		<div class="columns" style:--bands={levels.length} style:--segments={SEGMENTS}>
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
		gap: 1px;
		height: 100%;
		padding: 4px;
	}

	.column {
		position: relative;
		height: 100%;
	}

	.bar {
		position: absolute;
		inset: 0;
		background: linear-gradient(
			0deg,
			var(--onsa-role-active) 0 70%,
			var(--onsa-role-caution) 70% 92%,
			var(--onsa-role-clip) 92%
		);
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
		opacity: 0.85;
		transition: bottom 90ms linear;
	}

	/* Behind glass, a column is a stack of segments and the dead ones stay
	   faintly visible (SPEC section 9.4). */
	.spectrum[data-variant='segment'] .column {
		background: repeating-linear-gradient(
			0deg,
			var(--onsa-lit-ghost) 0 calc(100% / var(--segments) - 2px),
			transparent calc(100% / var(--segments) - 2px) calc(100% / var(--segments))
		);
	}

	.spectrum[data-variant='segment'] .columns {
		gap: 2px;
	}

	.spectrum[data-variant='segment'] .bar {
		mask: repeating-linear-gradient(
			0deg,
			#000 0 calc(100% / var(--segments) - 2px),
			transparent calc(100% / var(--segments) - 2px) calc(100% / var(--segments))
		);
	}

	.spectrum svg {
		width: 100%;
		height: 100%;
	}

	.fill {
		fill: var(--onsa-role-active);
		fill-opacity: 0.55;
		stroke: var(--onsa-lit-on);
		stroke-width: 0.6;
		vector-effect: non-scaling-stroke;
		transition: d 70ms linear;
	}

	:global(:root[data-onsa-glow='true']) .bar,
	:global(:root[data-onsa-glow='true']) .fill {
		filter: drop-shadow(0 0 4px var(--onsa-lit-glow));
	}

	@media (prefers-reduced-motion: reduce) {
		.spectrum {
			transition: none;
		}
	}
</style>
