<!--
	The spectrum visualizer (SPEC sections 4.4 and 9.5), drawn in the theme's
	own variant: lit segments behind glass, plain bars, or soft warm columns,
	following `docs/design/palet-preview.html`. Its hue follows the character
	of the sound while tone colour is on.

	While this is on screen the backend is told a spectrum is wanted; when it
	leaves, the analysis tap, the FFT and the frames all stop.
-->
<script lang="ts">
	import { toneColor, watch } from '$lib/analysis.svelte';
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

	const theme = $derived(activeTheme());
	const variant = $derived(theme?.variants.spectrum ?? 'bar');
	// Tone colour leans the lit colour towards the theme's warm or cool end
	// (SPEC section 9.5); everything else in here is drawn from it.
	const lit = $derived(toneColor('spectrum', theme?.color.lit.on ?? '#ffffff'));
	// A display's backlight leans instead of its pixels (Krom Saku).
	const backlight = $derived(toneColor('backlight', theme?.color.lit.lcd ?? '#B4C49B'));

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
	style:--onsa-tone-lit={lit}
	style:--onsa-lcd-lit={backlight}
	class:lcd={variant === 'lcd'}
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
		background: var(--onsa-tone-lit, var(--onsa-lit-on));
		clip-path: inset(calc(100% - var(--level, 0%)) 0 0 0);
		transition: clip-path 70ms linear;
	}

	.peak {
		position: absolute;
		left: 0;
		right: 0;
		height: 2px;
		bottom: var(--peak, 0%);
		background: var(--onsa-tone-lit, var(--onsa-lit-on));
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
			color-mix(in srgb, var(--onsa-tone-lit, var(--onsa-lit-on)) 55%, transparent),
			var(--onsa-tone-lit, var(--onsa-lit-on))
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

	/* Points of light rising in columns, with no well behind them; every
	   few columns glow in the theme's second colour. */
	.spectrum[data-variant='dots'] {
		background: none;
	}

	.spectrum[data-variant='dots'] .columns {
		gap: 5px;
	}

	.spectrum[data-variant='dots'] .bar {
		background: radial-gradient(
				circle,
				var(--onsa-tone-lit, var(--onsa-lit-on)) 0 1.6px,
				transparent 2.2px
			)
			center bottom / 9px 9px;
	}

	.spectrum[data-variant='dots'] .column:nth-child(7n + 4) .bar {
		background-image: radial-gradient(circle, var(--onsa-lit-secondary) 0 1.6px, transparent 2.2px);
	}

	.spectrum[data-variant='dots'] .peak {
		display: none;
	}

	/* Blades of grass growing from the ground with the sound. */
	.spectrum[data-variant='grass'] {
		background: none;
	}

	.spectrum[data-variant='grass'] .columns {
		gap: 3px;
		padding: 0;
	}

	.spectrum[data-variant='grass'] .bar {
		background: linear-gradient(
			0deg,
			color-mix(in srgb, var(--onsa-tone-lit, var(--onsa-lit-on)) 60%, black),
			var(--onsa-tone-lit, var(--onsa-lit-on)) 80%,
			color-mix(in srgb, var(--onsa-tone-lit, var(--onsa-lit-on)) 40%, white)
		);
		clip-path: polygon(50% calc(100% - var(--level, 0%)), 100% 100%, 0 100%);
		transition: clip-path 90ms linear;
	}

	.spectrum[data-variant='grass'] .peak {
		display: none;
	}

	/* Dark segments on a liquid crystal display, the dead ones faintly
	   there behind them. */
	.spectrum[data-variant='lcd'] {
		border: 2px solid var(--onsa-lit-lcd-edge);
		border-radius: 12px;
		background: var(--onsa-lcd-lit, var(--onsa-lit-lcd));
		box-shadow: inset 0 2px 6px rgb(0 0 0 / 0.25);
	}

	.spectrum[data-variant='lcd'] .column {
		background: repeating-linear-gradient(
			0deg,
			color-mix(in srgb, var(--onsa-lit-lcd-ink) 8%, transparent) 0 5px,
			transparent 5px 7px
		);
	}

	.spectrum[data-variant='lcd'] .bar {
		background: repeating-linear-gradient(0deg, var(--onsa-lit-lcd-ink) 0 5px, transparent 5px 7px);
	}

	.spectrum[data-variant='lcd'] .peak {
		background: var(--onsa-lit-lcd-ink);
	}

</style>
