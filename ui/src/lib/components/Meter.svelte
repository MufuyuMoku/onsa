<!--
	Peak meter L/R with CLIP and LIM (SPEC section 9.2), drawn in the theme's
	meter variant: bars, segments, or VU needles.
-->
<script lang="ts">
	import { watch } from '$lib/analysis.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import { activeTheme } from '$lib/theme/index.svelte';

	/** The meter shows the top 48 dB. */
	const RANGE_DB = 48;
	/** Needle swing either side of centre, in degrees. */
	const SWING = 48;

	const variant = $derived(activeTheme()?.variants.meter ?? 'bar');
	const levels = $derived(
		player.meter.peakDb.map((peak) => Math.min(1, Math.max(0, (peak + RANGE_DB) / RANGE_DB)))
	);
	const channels = $derived([t('meter.left'), t('meter.right')]);

	// While a meter is on screen the levels are wanted; when the last one
	// leaves, the analysis tap stops (SPEC section 4.4).
	$effect(() => watch('meters'));

	const ticks = Array.from({ length: 11 }, (_, k) => {
		const angle = ((-SWING + k * ((2 * SWING) / 10)) * Math.PI) / 180;
		const outer = 40;
		const inner = k % 5 === 0 ? 33 : 36;
		return {
			x1: 60 + Math.sin(angle) * outer,
			y1: 68 - Math.cos(angle) * outer,
			x2: 60 + Math.sin(angle) * inner,
			y2: 68 - Math.cos(angle) * inner,
			hot: k > 7,
			major: k % 5 === 0
		};
	});
</script>

<div class="meter" data-variant={variant} role="img" aria-label={t('meter.label')}>
	{#if variant === 'needle'}
		<div class="dials">
			{#each levels as level, channel (channel)}
				<svg class="dial" viewBox="0 0 120 76">
					{#each ticks as tick, k (k)}
						<line
							x1={tick.x1}
							y1={tick.y1}
							x2={tick.x2}
							y2={tick.y2}
							class:hot={tick.hot}
							stroke-width={tick.major ? 1.4 : 0.9}
						/>
					{/each}
					<text x="10" y="70">{t('meter.vu')}</text>
					<text x="104" y="70">{channels[channel]}</text>
					<line
						class="needle"
						x1="60"
						y1="68"
						x2="60"
						y2="22"
						style:transform="rotate({-SWING + level * 2 * SWING}deg)"
					/>
					<circle cx="60" cy="68" r="3.2" />
				</svg>
			{/each}
		</div>
	{:else}
		<div class="lanes">
			{#each levels as level, channel (channel)}
				<div class="channel">
					<span class="label numeric">{channels[channel]}</span>
					<div class="lane"><i style:--level="{level * 100}%"></i></div>
				</div>
			{/each}
		</div>
	{/if}
	<div class="flags label">
		<span class="flag clip" class:lit={player.clip}>{t('meter.clip')}</span>
		<span class="flag limit" class:lit={player.limiting}>{t('meter.limit')}</span>
	</div>
</div>

<style>
	.meter {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto;
		align-items: center;
		gap: 10px;
		min-width: 0;
	}

	.lanes {
		display: grid;
		gap: 7px;
	}

	.channel {
		display: grid;
		grid-template-columns: 12px 1fr;
		align-items: center;
		gap: 6px;
		font-size: 10px;
	}

	.lane {
		position: relative;
		height: 8px;
		border-radius: 2px;
		overflow: hidden;
		background: var(--onsa-lit-ghost);
	}

	.lane i {
		position: absolute;
		inset: 0;
		/* Green up to −12 dB, caution to −3 dB, clip above. */
		background: linear-gradient(
			90deg,
			var(--onsa-role-active) 0 75%,
			var(--onsa-role-caution) 75% 94%,
			var(--onsa-role-clip) 94%
		);
		clip-path: inset(0 calc(100% - var(--level, 0%)) 0 0);
		transition: clip-path 60ms linear;
	}

	.meter[data-variant='segment'] .lane {
		mask: repeating-linear-gradient(90deg, #000 0 4px, transparent 4px 6px);
	}

	:global(:root[data-onsa-glow='true']) .lane i {
		filter: drop-shadow(0 0 3px var(--onsa-lit-glow));
	}

	.dials {
		display: flex;
		gap: 6px;
	}

	.dial {
		width: 76px;
		height: 48px;
		border-radius: 4px;
		background: var(--onsa-lit-meter-face);
		box-shadow:
			inset 0 0 0 2px var(--onsa-surface-well),
			inset 0 6px 14px rgb(0 0 0 / 0.35);
	}

	.dial line {
		stroke: var(--onsa-surface-well);
	}

	.dial line.hot {
		stroke: var(--onsa-role-clip);
	}

	.dial text {
		font-size: 8px;
		fill: var(--onsa-surface-well);
		font-family: var(--onsa-font-label);
	}

	.dial circle {
		fill: var(--onsa-surface-well);
	}

	.dial .needle {
		stroke: var(--onsa-lit-needle);
		stroke-width: 1.3;
		stroke-linecap: round;
		transform-origin: 60px 68px;
		transition: transform 90ms linear;
	}

	.flags {
		display: grid;
		gap: 4px;
		font-size: 9.5px;
	}

	.flag {
		padding: 1px 5px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: 2px;
		color: var(--onsa-lit-ghost);
		text-align: center;
	}

	.flag.clip.lit {
		border-color: var(--onsa-role-clip);
		color: var(--onsa-role-clip);
	}

	.flag.limit.lit {
		border-color: var(--onsa-role-caution);
		color: var(--onsa-role-caution);
	}
</style>
