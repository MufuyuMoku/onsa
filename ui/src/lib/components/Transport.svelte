<!--
	Transport (SPEC section 9.2): cover, title, controls, position, volume,
	meter.

	The play controls, the position and the volume are always there. As the
	window narrows the meter goes first, then the cover, and the buttons
	beside them move into a button of their own (SPEC section 9.2).
-->
<script lang="ts">
	import { tonePosition } from '$lib/analysis.svelte';
	import { coverUrl, nextTrack, previousTrack, seek, togglePlay } from '$lib/backend';
	import { app, navigate, setMiniPlayer } from '$lib/app.svelte';
	import { clock, db } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { measure } from '$lib/layout.svelte';
	import { player } from '$lib/player.svelte';
	import { settings, updateDsp } from '$lib/settings.svelte';
	import Icon from './Icon.svelte';
	import Meter from './Meter.svelte';
	import SleepTimer from './SleepTimer.svelte';

	/** The volume slider's range; its bottom is silence. */
	const VOLUME_FLOOR = -60;

	const snapshot = $derived(player.snapshot);
	const track = $derived(snapshot?.track ?? null);
	const playing = $derived(snapshot?.state === 'playing');
	const duration = $derived(snapshot?.duration ?? 0);
	const volume = $derived(settings.value?.dsp.volumeDb ?? 0);

	/** Width of the transport itself. */
	let width = $state(1200);
	/** The meters have room beside everything else. */
	const metered = $derived(width >= 1000);
	/** The extra buttons stand on their own rather than in a menu. */
	const roomy = $derived(width >= 1120);
	/** The cover is worth its 76 pixels. */
	const covered = $derived(width >= 760);

	let extras = $state(false);
	$effect(() => {
		if (roomy) extras = false;
	});

	let dragging = $state(false);
	let dragValue = $state(0);
	const shown = $derived(dragging ? dragValue : player.position);
	const time = $derived(clock(shown));

	function setVolume(value: number): void {
		updateDsp({ volumeDb: value <= VOLUME_FLOOR ? -90 : value });
	}

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}
</script>

{#snippet buttons()}
	<button
		type="button"
		class="icon-btn"
		aria-label={t('nowPlaying.open')}
		aria-pressed={app.view.kind === 'nowPlaying'}
		disabled={!track}
		onclick={() => {
			extras = false;
			navigate({ kind: 'nowPlaying' });
		}}
	>
		<Icon name="disc" />
	</button>
	<SleepTimer />
	<button
		type="button"
		class="icon-btn"
		aria-label={t('mini.open')}
		onclick={() => {
			extras = false;
			setMiniPlayer(true);
		}}
	>
		<Icon name="mini" />
	</button>
{/snippet}

<footer
	class="transport"
	class:metered
	class:roomy
	class:covered
	use:measure={(seen) => (width = seen)}
>
	{#if covered}
		<div class="art">
			{#if track?.coverId != null}<img src={coverUrl(track.coverId, 128)} alt="" />{/if}
		</div>
	{/if}

	<div class="info well">
		{#if track}
			<div class="title ellipsis">{track.title ?? fileName(track.path)}</div>
			<div class="artist ellipsis">{track.artist ?? t('track.unknownArtist')}</div>
		{:else}
			<div class="title ellipsis muted">{t('transport.nothing')}</div>
			<div class="artist">&nbsp;</div>
		{/if}
		{#if snapshot?.outputError}
			<div class="notice fault-text ellipsis">{t('transport.noOutput')}</div>
		{:else if snapshot?.failedTrack}
			<div class="notice fault-text ellipsis">
				{t('transport.failed', { path: fileName(snapshot.failedTrack) })}
			</div>
		{/if}
		<div class="timeline">
			<span class="clock numeric">
				<span class="ghost" aria-hidden="true">{time.replace(/\d/g, '8')}</span>
				<span class="lit">{time}</span>
			</span>
			<input
				class="seek"
				style:accent-color={tonePosition()}
				type="range"
				min="0"
				max={duration || 1}
				step="0.1"
				value={shown}
				disabled={!track || !duration}
				aria-label={t('transport.seek')}
				oninput={(event) => {
					dragging = true;
					dragValue = Number(event.currentTarget.value);
				}}
				onchange={(event) => {
					const target = Number(event.currentTarget.value);
					seek(target)
						.catch(() => {})
						.finally(() => (dragging = false));
				}}
			/>
			<span class="numeric muted total">{clock(snapshot?.duration)}</span>
		</div>
	</div>

	<div class="controls">
		<button
			type="button"
			class="icon-btn"
			aria-label={t('transport.previous')}
			disabled={!track}
			onclick={() => previousTrack().catch(() => {})}><Icon name="previous" /></button
		>
		<button
			type="button"
			class="icon-btn large"
			aria-label={playing ? t('transport.pause') : t('transport.play')}
			disabled={!track}
			onclick={() => togglePlay().catch(() => {})}
			><Icon name={playing ? 'pause' : 'play'} /></button
		>
		<button
			type="button"
			class="icon-btn"
			aria-label={t('transport.next')}
			disabled={!track}
			onclick={() => nextTrack().catch(() => {})}><Icon name="next" /></button
		>
	</div>

	<div class="volume">
		<span class="muted"><Icon name="volume" /></span>
		<input
			type="range"
			min={VOLUME_FLOOR}
			max="0"
			step="0.5"
			value={Math.max(VOLUME_FLOOR, volume)}
			aria-label={t('transport.volume')}
			oninput={(event) => setVolume(Number(event.currentTarget.value))}
		/>
		<span class="numeric value">{volume <= VOLUME_FLOOR ? db(-Infinity) : db(volume)}</span>
	</div>


	<div class="extras">
		{#if roomy}
			{@render buttons()}
		{:else}
			<button
				type="button"
				class="icon-btn"
				aria-label={t('transport.more')}
				aria-expanded={extras}
				title={t('transport.more')}
				onclick={() => (extras = !extras)}
			>
				<Icon name="more" />
			</button>
		{/if}
	</div>

	{#if metered}
		<div class="meters well"><Meter /></div>
	{/if}
</footer>

{#if extras && !roomy}
	<!-- Closing the row is what the backdrop is for; it is not a control. -->
	<div class="backdrop" role="presentation" onclick={() => (extras = false)}></div>
	<div class="popover">{@render buttons()}</div>
{/if}

<style>
	.transport {
		display: grid;
		/* Info, controls, volume and the extras button: what is always here. */
		grid-template-columns: minmax(150px, 1.6fr) auto minmax(120px, 0.7fr) auto;
		align-items: center;
		gap: 12px;
		padding: 12px 16px;
		min-width: 0;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
	}

	.transport.covered {
		grid-template-columns: 76px minmax(150px, 1.6fr) auto minmax(120px, 0.7fr) auto;
	}

	.transport.metered {
		grid-template-columns: 76px minmax(190px, 1.6fr) auto minmax(130px, 0.7fr) auto minmax(
				170px,
				0.9fr
			);
	}

	.transport > * {
		min-width: 0;
	}

	.backdrop {
		position: fixed;
		inset: 0;
		z-index: 10;
	}

	.popover {
		position: fixed;
		right: 12px;
		bottom: 86px;
		z-index: 11;
		display: flex;
		align-items: center;
		gap: 6px;
		padding: 6px 8px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		box-shadow: 0 10px 24px rgb(0 0 0 / 0.45);
	}

	:global(:root[data-onsa-panel-texture='brushed']) .transport {
		background:
			repeating-linear-gradient(90deg, rgb(255 255 255 / 0.022) 0 1px, transparent 1px 3px),
			linear-gradient(180deg, var(--onsa-surface-body-highlight), var(--onsa-surface-body) 55%);
	}

	.art {
		flex: none;
		width: 76px;
		height: 76px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		overflow: hidden;
	}

	.art img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.info {
		display: grid;
		gap: 1px;
		min-width: 0;
		padding: 8px 12px;
	}

	:global(:root[data-onsa-glass-reflection='true']) .info {
		background-image: linear-gradient(118deg, rgb(255 255 255 / 0.045) 0 28%, transparent 28.5%);
	}

	.title {
		font-size: 15px;
		color: var(--onsa-text-primary);
	}

	:global(:root[data-onsa-glow='true']) .title {
		text-shadow: 0 0 6px var(--onsa-lit-glow);
	}

	.artist {
		font-size: 12.5px;
		color: var(--onsa-lit-secondary);
	}

	.notice {
		font-size: 11.5px;
	}

	.timeline {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto;
		align-items: center;
		gap: 10px;
		margin-top: 4px;
	}

	.clock {
		position: relative;
		font-size: 20px;
		line-height: 1;
		color: var(--onsa-role-position);
	}

	.clock .ghost {
		position: absolute;
		inset: 0;
		color: var(--onsa-lit-ghost);
		visibility: hidden;
	}

	:global(:root[data-onsa-time-display='ghost-segment']) .clock .ghost {
		visibility: visible;
	}

	.clock .lit {
		position: relative;
	}

	.seek {
		width: 100%;
		accent-color: var(--onsa-role-position);
	}

	.total {
		font-size: 12px;
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.volume {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.volume input {
		width: 100%;
	}

	.value {
		min-width: 64px;
		font-size: 12px;
		color: var(--onsa-role-adjustable);
		text-align: right;
	}

	.extras {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.icon-btn[aria-pressed='true'] {
		border-color: var(--onsa-role-active);
		color: var(--onsa-role-active);
	}

	.meters {
		padding: 8px 10px;
		min-width: 0;
	}
</style>
