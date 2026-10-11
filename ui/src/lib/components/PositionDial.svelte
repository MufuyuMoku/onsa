<!--
	The play position as a round dial (SPEC M16): one hand that goes round
	once per song, on a face in the theme's meter colours, with the time
	written underneath it.

	It shows; it does not take hold. Seeking stays with the position bar in
	the transport, so the dial is drawn as an instrument and never as a
	control (no fake buttons, SPEC section 9.1).
-->
<script lang="ts">
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';

	let { position, duration }: { position: number; duration: number } = $props();

	const fraction = $derived(duration > 0 ? Math.min(1, Math.max(0, position / duration)) : 0);
	const percent = $derived(Math.round(fraction * 100));

	/** Sixty marks round the face, a longer one every five. */
	const marks = Array.from({ length: 60 }, (_, k) => {
		const angle = (k / 60) * Math.PI * 2;
		const outer = 46;
		const inner = k % 5 === 0 ? 39 : 43;
		return {
			x1: 50 + Math.sin(angle) * outer,
			y1: 50 - Math.cos(angle) * outer,
			x2: 50 + Math.sin(angle) * inner,
			y2: 50 - Math.cos(angle) * inner,
			major: k % 5 === 0
		};
	});
</script>

<svg
	class="dial"
	viewBox="0 0 100 100"
	role="img"
	aria-label={t('nowPlaying.dialLabel', { percent, time: clock(position) })}
>
	<circle class="face" cx="50" cy="50" r="48" />
	{#each marks as mark, k (k)}
		<line
			x1={mark.x1}
			y1={mark.y1}
			x2={mark.x2}
			y2={mark.y2}
			stroke-width={mark.major ? 1.6 : 0.7}
		/>
	{/each}
	<text class="unit" x="50" y="33" text-anchor="middle">{t('nowPlaying.dialUnit')}</text>
	<text class="time" x="50" y="74" text-anchor="middle">{clock(position)}</text>
	<line class="hand" x1="50" y1="50" x2="50" y2="12" style:transform="rotate({fraction * 360}deg)" />
	<circle class="cap" cx="50" cy="50" r="4.5" />
</svg>

<style>
	.dial {
		display: block;
		width: 100%;
		height: auto;
		border-radius: 50%;
		/* The bezel round the instrument. */
		box-shadow:
			0 0 0 6px var(--onsa-surface-well),
			0 0 0 8px var(--onsa-surface-line);
	}

	.face {
		fill: var(--onsa-lit-meter-face);
	}

	line {
		stroke: var(--onsa-surface-well);
	}

	text {
		fill: var(--onsa-surface-well);
		font-family: var(--onsa-font-numeric);
	}

	.unit {
		font-size: 7px;
		letter-spacing: 0.08em;
		text-transform: uppercase;
	}

	.time {
		font-size: 11px;
		font-variant-numeric: tabular-nums;
	}

	.hand {
		stroke: var(--onsa-surface-well);
		stroke-width: 2.2;
		stroke-linecap: round;
		transform-origin: 50px 50px;
		transition: transform var(--onsa-motion-fast) linear;
	}

	.cap {
		fill: var(--onsa-surface-well);
	}

	/* Still means still: under reduced motion the hand jumps rather than
	   glides (SPEC section 9.1). */
	@media (prefers-reduced-motion: reduce) {
		.hand {
			transition: none;
		}
	}
</style>
