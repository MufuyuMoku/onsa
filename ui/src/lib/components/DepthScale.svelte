<!--
	The play position as a depth (SPEC M16): the start of the song at the
	surface, its end at the floor, four thousand metres down. The marker
	says how deep and what time it is.

	It is a real control, the way the position bar is: press anywhere on the
	scale or drag the marker to seek, and the arrow keys step it. Seeking
	happens when the pointer is let go, so a drag does not stutter through
	the song.
-->
<script lang="ts">
	import { seek } from '$lib/backend';
	import { clock } from '$lib/format';
	import { currentLocale, t } from '$lib/i18n/index.svelte';

	let { position, duration }: { position: number; duration: number } = $props();

	/** How deep the floor is, and where the zones begin on the way down. */
	const FLOOR = 4000;
	const ZONES = [
		{ metres: 200, key: 'depth.twilight' },
		{ metres: 1000, key: 'depth.midnight' },
		{ metres: FLOOR, key: 'depth.floor' }
	] as const;
	/** A keypress moves this far; Page keys a tenth of the song. */
	const STEP_SECONDS = 5;

	let scale: HTMLDivElement | undefined = $state();
	let dragging = $state<number | null>(null);

	const shown = $derived(dragging ?? position);
	const fraction = $derived(duration > 0 ? Math.min(1, Math.max(0, shown / duration)) : 0);
	const metres = $derived(Math.round(fraction * FLOOR));
	/** A zone's name steps aside when the marker's label is on it. */
	const near = (metres: number) => Math.abs(metres / FLOOR - fraction) < 0.07;
	/** Near the surface the label hangs below the marker, not above it. */
	const below = $derived(fraction < 0.08);

	const depthText = $derived(
		`${new Intl.NumberFormat(currentLocale() === 'id' ? 'id-ID' : 'en-GB').format(metres)} m`
	);

	function at(event: PointerEvent): number {
		const box = scale?.getBoundingClientRect();
		if (!box || duration <= 0) return 0;
		const down = Math.min(1, Math.max(0, (event.clientY - box.top) / box.height));
		return down * duration;
	}

	function press(event: PointerEvent): void {
		if (duration <= 0) return;
		scale?.setPointerCapture(event.pointerId);
		dragging = at(event);
	}

	function move(event: PointerEvent): void {
		if (dragging !== null) dragging = at(event);
	}

	function release(): void {
		if (dragging === null) return;
		const target = dragging;
		seek(target)
			.catch(() => {})
			.finally(() => (dragging = null));
	}

	function key(event: KeyboardEvent): void {
		if (duration <= 0) return;
		const steps: Record<string, number> = {
			ArrowDown: STEP_SECONDS,
			ArrowRight: STEP_SECONDS,
			ArrowUp: -STEP_SECONDS,
			ArrowLeft: -STEP_SECONDS,
			PageDown: duration / 10,
			PageUp: -duration / 10
		};
		let target: number | null = null;
		if (event.key in steps) target = position + steps[event.key];
		else if (event.key === 'Home') target = 0;
		else if (event.key === 'End') target = duration;
		if (target === null) return;
		event.preventDefault();
		seek(Math.min(duration, Math.max(0, target))).catch(() => {});
	}
</script>

<div
	class="depth"
	bind:this={scale}
	role="slider"
	tabindex={duration > 0 ? 0 : -1}
	aria-label={t('transport.seek')}
	aria-valuemin={0}
	aria-valuemax={Math.round(duration)}
	aria-valuenow={Math.round(shown)}
	aria-valuetext={t('depth.valueText', { depth: depthText, time: clock(shown) })}
	aria-disabled={duration <= 0}
	onpointerdown={press}
	onpointermove={move}
	onpointerup={release}
	onpointercancel={() => (dragging = null)}
	onkeydown={key}
>
	<div class="line" aria-hidden="true">
		<span class="tick" style:top="0%"></span>
		{#each ZONES as zone (zone.key)}
			<span class="tick" style:top="{(zone.metres / FLOOR) * 100}%"></span>
		{/each}
		<span class="marker" style:top="{fraction * 100}%"></span>
	</div>
	<div class="labels" aria-hidden="true">
		<span class="zone" class:away={near(0)} style:top="0%">0 m</span>
		{#each ZONES as zone (zone.key)}
			<span class="zone" class:away={near(zone.metres)} style:top="{(zone.metres / FLOOR) * 100}%"
				>{zone.metres} · {t(zone.key)}</span
			>
		{/each}
		<span class="here" class:below style:top="{fraction * 100}%">
			<span class="metres numeric">{depthText}</span>
			<span class="time numeric">{clock(shown)}</span>
		</span>
	</div>
</div>

<style>
	.depth {
		position: relative;
		display: grid;
		grid-template-columns: 14px minmax(0, 1fr);
		gap: 8px;
		height: 100%;
		min-height: 240px;
		padding: 8px 0;
		cursor: ns-resize;
		touch-action: none;
		outline: none;
	}

	.depth:focus-visible .marker {
		outline: 2px solid var(--onsa-role-adjustable);
		outline-offset: 3px;
	}

	.depth[aria-disabled='true'] {
		cursor: default;
		opacity: 0.6;
	}

	.line {
		position: relative;
		justify-self: center;
		width: 1px;
		background: linear-gradient(var(--onsa-role-label), var(--onsa-surface-line));
	}

	.tick {
		position: absolute;
		left: -3px;
		width: 7px;
		height: 1px;
		background: var(--onsa-role-label);
	}

	.marker {
		position: absolute;
		left: -12px;
		width: 26px;
		height: 1px;
		background: var(--onsa-role-position);
		box-shadow: 0 0 10px 1px var(--onsa-role-active);
		transition: top var(--onsa-motion-fast) linear;
	}

	.labels {
		position: relative;
		font-family: var(--onsa-font-label);
		font-size: 9.5px;
		letter-spacing: 0.08em;
		color: var(--onsa-role-label);
		text-transform: uppercase;
	}

	.zone {
		position: absolute;
		left: 0;
		transform: translateY(-50%);
		white-space: nowrap;
	}

	.zone.away {
		visibility: hidden;
	}

	.here {
		position: absolute;
		left: 0;
		display: grid;
		transform: translateY(-100%);
		transition: top var(--onsa-motion-fast) linear;
	}

	.here.below {
		transform: translateY(4px);
	}

	.metres {
		color: var(--onsa-role-position);
		font-size: 13px;
		letter-spacing: 0;
		text-transform: none;
		text-shadow: 0 0 10px var(--onsa-role-active);
	}

	.time {
		color: var(--onsa-role-active);
		font-size: 10px;
	}

	@media (prefers-reduced-motion: reduce) {
		.marker,
		.here {
			transition: none;
		}
	}
</style>
