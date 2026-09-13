<!--
	The mini player (SPEC section 9.2): one line of instrument panel, like a
	single rack unit. The window itself is resized by the backend.
-->
<script lang="ts">
	import { tonePosition } from '$lib/analysis.svelte';
	import { coverUrl, nextTrack, previousTrack, seek, togglePlay } from '$lib/backend';
	import { setMiniPlayer } from '$lib/app.svelte';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import Icon from './Icon.svelte';
	import Meter from './Meter.svelte';

	const snapshot = $derived(player.snapshot);
	const track = $derived(snapshot?.track ?? null);
	const playing = $derived(snapshot?.state === 'playing');
	const duration = $derived(snapshot?.duration ?? 0);

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}
</script>

<div class="mini">
	<div class="art">
		{#if track?.coverId != null}<img src={coverUrl(track.coverId, 128)} alt="" />{/if}
	</div>

	<div class="info well">
		<div class="ellipsis title">
			{track ? (track.title ?? fileName(track.path)) : t('transport.nothing')}
		</div>
		<div class="ellipsis artist">{track?.artist ?? ''}</div>
		<div class="timeline">
			<span class="numeric time">{clock(player.position)}</span>
			<input
				class="seek"
				style:accent-color={tonePosition()}
				type="range"
				min="0"
				max={duration || 1}
				step="0.1"
				value={player.position}
				disabled={!track || !duration}
				aria-label={t('transport.seek')}
				onchange={(event) => seek(Number(event.currentTarget.value)).catch(() => {})}
			/>
			<span class="numeric muted time">{clock(snapshot?.duration)}</span>
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

	<div class="meters well"><Meter /></div>

	<button
		type="button"
		class="icon-btn"
		aria-label={t('mini.close')}
		onclick={() => setMiniPlayer(false)}
	>
		<Icon name="full" />
	</button>
</div>

<style>
	.mini {
		display: grid;
		grid-template-columns: 60px minmax(0, 1fr) auto auto auto;
		align-items: center;
		gap: 10px;
		height: 100%;
		padding: 8px 10px;
		background: var(--onsa-surface-body);
	}

	:global(:root[data-onsa-panel-texture='brushed']) .mini {
		background:
			repeating-linear-gradient(90deg, rgb(255 255 255 / 0.022) 0 1px, transparent 1px 3px),
			linear-gradient(180deg, var(--onsa-surface-body-highlight), var(--onsa-surface-body) 55%);
	}

	.art {
		width: 60px;
		height: 60px;
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
		padding: 6px 10px;
	}

	.title {
		font-size: 13.5px;
	}

	.artist {
		font-size: 11.5px;
		color: var(--onsa-lit-secondary);
	}

	.timeline {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto;
		align-items: center;
		gap: 8px;
		margin-top: 2px;
	}

	.time {
		font-size: 11px;
		color: var(--onsa-role-position);
	}

	.seek {
		width: 100%;
		accent-color: var(--onsa-role-position);
	}

	.controls {
		display: flex;
		align-items: center;
		gap: 4px;
	}

	.controls :global(.icon-btn) {
		width: 30px;
		height: 30px;
	}

	.controls :global(.icon-btn.large) {
		width: 36px;
		height: 36px;
	}

	.meters {
		width: 150px;
		padding: 5px 7px;
	}

	.meters :global(.dial) {
		width: 56px;
		height: 34px;
	}
</style>
