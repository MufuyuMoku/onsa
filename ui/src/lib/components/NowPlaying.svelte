<!--
	Now Playing (SPEC section 9.2): the cover large, the spectrum, and the
	signal told honestly. The lyrics join it in M8.
-->
<script lang="ts">
	import { tonePosition } from '$lib/analysis.svelte';
	import { coverUrl, seek } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { clock, db, rate } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import Icon from './Icon.svelte';
	import Spectrum from './Spectrum.svelte';

	const snapshot = $derived(player.snapshot);
	const track = $derived(snapshot?.track ?? null);
	const signal = $derived(snapshot?.signal ?? null);
	const duration = $derived(snapshot?.duration ?? 0);

	function fileName(path: string): string {
		return path.split(/[\\/]/).pop() ?? path;
	}

	const format = $derived.by(() => {
		const source = signal?.source;
		if (!source) return null;
		const parts = [source.codec ?? '', rate(source.sampleRate)];
		if (source.bitDepth) parts.push(`${source.bitDepth}-bit`);
		parts.push(`${source.channels} ch`);
		return parts.filter(Boolean).join(' · ');
	});

	const output = $derived.by(() => {
		const stage = signal?.output;
		if (!stage) return null;
		return `${stage.backend} · ${stage.name} · ${rate(stage.sampleRate)}`;
	});
</script>

<div class="now">
	<button type="button" class="back label" onclick={() => navigate({ kind: 'tracks' })}>
		<Icon name="back" />{t('nowPlaying.close')}
	</button>

	{#if track}
		<div class="art">
			{#if track.coverId !== null}<img src={coverUrl(track.coverId, 512)} alt="" />{/if}
		</div>
		<div class="text">
			<h1 class="ellipsis">{track.title ?? fileName(track.path)}</h1>
			<p class="artist ellipsis">{track.artist ?? t('track.unknownArtist')}</p>
			{#if track.album}
				<p class="muted ellipsis">{track.album}{track.year ? ` · ${track.year}` : ''}</p>
			{/if}

			<div class="timeline">
				<span class="numeric position">{clock(player.position)}</span>
				<input
					class="seek"
					style:accent-color={tonePosition()}
					type="range"
					min="0"
					max={duration || 1}
					step="0.1"
					value={player.position}
					disabled={!duration}
					aria-label={t('transport.seek')}
					onchange={(event) => seek(Number(event.currentTarget.value)).catch(() => {})}
				/>
				<span class="numeric muted">{clock(snapshot?.duration)}</span>
			</div>

			<dl class="facts">
				{#if format}
					<dt class="label">{t('nowPlaying.format')}</dt>
					<dd class="numeric">{format}</dd>
				{/if}
				{#if signal?.resample}
					<dt class="label">{t('signal.resample')}</dt>
					<dd class="numeric">{rate(signal.resample.from)} → {rate(signal.resample.to)}</dd>
				{/if}
				{#if signal && signal.replaygain.mode !== 'off'}
					<dt class="label">{t('signal.replaygain')}</dt>
					<dd class="numeric">
						{signal.replaygain.album ? t('signal.album') : t('signal.track')}
						{signal.replaygain.gainDb === null ? '' : db(signal.replaygain.gainDb)}
					</dd>
				{/if}
				{#if signal?.eq.enabled}
					<dt class="label">{t('signal.eq')}</dt>
					<dd class="numeric">
						{signal.eq.kind === 'graphic'
							? t('signal.bands', { n: signal.eq.bands })
							: t('signal.filters', { n: signal.eq.bands })} · {db(signal.eq.preampDb)}
					</dd>
				{/if}
				{#if output}
					<dt class="label">{t('nowPlaying.output')}</dt>
					<dd class="numeric">{output}</dd>
				{/if}
			</dl>
		</div>
		<div class="visualizer">
			<Spectrum height={170} />
		</div>
	{:else}
		<p class="muted nothing">{t('transport.nothing')}</p>
	{/if}
</div>

<style>
	.now {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr);
		grid-template-rows: auto auto 1fr;
		align-content: start;
		gap: 18px 28px;
		height: 100%;
		padding: 16px 24px 24px;
		overflow-x: hidden;
		overflow-y: auto;
	}

	/* Narrow enough that the cover and the words would fight: the cover
	   steps above the words instead (SPEC section 9.2). */
	@container content (max-width: 700px) {
		.now {
			grid-template-columns: minmax(0, 1fr);
			gap: 14px;
			padding: 14px 16px 20px;
		}

		.art {
			width: min(240px, 60cqw);
			height: min(240px, 60cqw);
			justify-self: center;
		}
	}

	.visualizer {
		grid-column: 1 / -1;
		min-width: 0;
	}

	.back {
		grid-column: 1 / -1;
		display: inline-flex;
		align-items: center;
		justify-self: start;
		gap: 4px;
		padding: 0;
		border: 0;
		background: none;
		font-size: 11px;
		cursor: pointer;
	}

	.back:hover {
		color: var(--onsa-role-adjustable);
	}

	/* Both sides are given: a grid row cannot work a height out of an
	   aspect ratio, and the cover would sit on top of the words. */
	.art {
		width: min(340px, 32cqw);
		height: min(340px, 32cqw);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-well);
		box-shadow: 0 18px 40px rgb(0 0 0 / 0.45);
		overflow: hidden;
	}

	.art img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.text {
		display: flex;
		flex-direction: column;
		min-width: 0;
		gap: 6px;
		min-width: 0;
		align-self: center;
	}

	h1 {
		margin: 0;
		max-width: 100%;
		font-size: 30px;
		font-weight: 500;
	}

	:global(:root[data-onsa-glow='true']) h1 {
		text-shadow: 0 0 12px var(--onsa-lit-glow);
	}

	.artist {
		margin: 0;
		font-size: 16px;
		color: var(--onsa-lit-secondary);
	}

	p {
		margin: 0;
		max-width: 100%;
	}

	.timeline {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr) auto;
		align-items: center;
		gap: 10px;
		margin: 14px 0 6px;
		max-width: 620px;
	}

	.position {
		font-size: 18px;
		color: var(--onsa-role-position);
	}

	.seek {
		width: 100%;
		accent-color: var(--onsa-role-position);
	}

	.facts {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr);
		gap: 4px 16px;
		margin: 10px 0 0;
		font-size: 12.5px;
	}

	.facts dt {
		font-size: 10.5px;
	}

	.facts dd {
		margin: 0;
		color: var(--onsa-role-adjustable);
	}

	.nothing {
		grid-column: 1 / -1;
		align-self: center;
		justify-self: center;
	}
</style>
