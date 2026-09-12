<!-- The EQ response curve (SPEC section 4.2), computed by the backend. -->
<script lang="ts">
	import { eqCurve, type DspPrefs, type EqCurve } from '$lib/backend';
	import { frequency } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';

	let { dsp }: { dsp: DspPrefs } = $props();

	const WIDTH = 640;
	const HEIGHT = 170;
	const RANGE_DB = 15;
	const LOW = Math.log10(20);
	const HIGH = Math.log10(20000);

	let curve = $state<EqCurve | null>(null);
	let busy = false;
	let again = false;

	$effect(() => {
		void dsp.eqKind;
		void dsp.graphicGains;
		void dsp.parametric;
		void refresh();
	});

	async function refresh(): Promise<void> {
		if (busy) {
			again = true;
			return;
		}
		busy = true;
		try {
			do {
				again = false;
				curve = await eqCurve($state.snapshot(dsp));
			} while (again);
		} catch {
			curve = null;
		} finally {
			busy = false;
		}
	}

	function x(hz: number): number {
		return ((Math.log10(hz) - LOW) / (HIGH - LOW)) * WIDTH;
	}

	function y(gain: number): number {
		const clamped = Math.max(-RANGE_DB, Math.min(RANGE_DB, gain));
		return HEIGHT / 2 - (clamped / RANGE_DB) * (HEIGHT / 2 - 6);
	}

	const path = $derived(
		curve
			? curve.points
					.map(([hz, gain], i) => `${i === 0 ? 'M' : 'L'}${x(hz).toFixed(1)},${y(gain).toFixed(1)}`)
					.join(' ')
			: ''
	);
	const freqs = [100, 1000, 10000];
	const gains = [-12, -6, 6, 12];
</script>

<svg class="curve well" viewBox="0 0 {WIDTH} {HEIGHT}" role="img" aria-label={t('eq.curve')}>
	{#each freqs as hz (hz)}
		<line class="grid" x1={x(hz)} x2={x(hz)} y1="0" y2={HEIGHT} />
		<text x={x(hz) + 4} y={HEIGHT - 6}>{frequency(hz)}</text>
	{/each}
	{#each gains as gain (gain)}
		<line class="grid" x1="0" x2={WIDTH} y1={y(gain)} y2={y(gain)} />
		<text x="4" y={y(gain) - 3}>{gain > 0 ? `+${gain}` : `−${-gain}`}</text>
	{/each}
	<line class="zero" x1="0" x2={WIDTH} y1={y(0)} y2={y(0)} />
	{#if path}<path d={path} class:off={!dsp.eqEnabled} />{/if}
</svg>

<style>
	.curve {
		display: block;
		width: 100%;
		height: auto;
	}

	.grid {
		stroke: var(--onsa-surface-line);
		stroke-width: 1;
	}

	.zero {
		stroke: var(--onsa-role-label);
		stroke-width: 1;
		opacity: 0.5;
	}

	text {
		font-size: 10px;
		fill: var(--onsa-role-label);
		font-family: var(--onsa-font-numeric);
	}

	path {
		fill: none;
		stroke: var(--onsa-role-adjustable);
		stroke-width: 2;
	}

	path.off {
		opacity: 0.35;
	}

	:global(:root[data-onsa-glow='true']) path:not(.off) {
		filter: drop-shadow(0 0 4px var(--onsa-lit-glow));
	}
</style>
