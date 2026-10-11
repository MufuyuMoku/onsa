<!--
	Now Playing (SPEC section 9.2): the cover large, the spectrum, and the
	signal told honestly. The lyrics join it in M8.

	Which of those are on screen is the listener's to decide (M14), and the
	decision is data rather than anything this component works out: the bar
	at the top turns a panel on or off, the answer is stored, and the layout
	presets of M16 will arrange the same map. Hiding the visualizer really
	stops the analysis, because the component leaves the page and takes its
	claim on the tap with it.

	How the panels are arranged comes from the layout in force (SPEC M16):
	side by side as Onsa always drew them, the picture large with the words
	(Layar penuh), or as instruments with name plates. Each is a value of
	the arrangement, never a question about which theme is being worn.
-->
<script lang="ts">
	import { tonePosition } from '$lib/analysis.svelte';
	import { coverUrl, seek } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { clock, db, fileName, rate } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import { arrangement } from '$lib/arrangement.svelte';
	import { PANELS, shown, togglePanel, type PanelId } from '$lib/panels.svelte';
	import Icon from './Icon.svelte';
	import LyricsPanel from './LyricsPanel.svelte';
	import PositionDial from './PositionDial.svelte';
	import Spectrum from './Spectrum.svelte';

	const arranged = $derived(arrangement());
	/** Whether the position is a dial of its own rather than the bar. */
	const dialled = $derived(arranged.position === 'dial');

	/** The words on each toggle, which the help quotes as they are. */
	const panelName = (id: PanelId) => t(`panel.${id}` as never);

	const snapshot = $derived(player.snapshot);
	const track = $derived(snapshot?.track ?? null);
	const signal = $derived(snapshot?.signal ?? null);
	const duration = $derived(snapshot?.duration ?? 0);

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

<div
	class="now"
	class:wide={!shown('cover')}
	data-arrangement={arranged.nowPlaying}
	data-cover={arranged.cover}
	class:plated={arranged.plates}
	class:dialled
>
	<div class="bar">
		<button type="button" class="back label" onclick={() => navigate({ kind: 'tracks' })}>
			<Icon name="back" />{t('nowPlaying.close')}
		</button>
		<div class="panels" role="group" aria-label={t('panel.title')}>
			{#each PANELS as id (id)}
				<button
					type="button"
					class="toggle label"
					class:on={shown(id)}
					aria-pressed={shown(id)}
					onclick={() => togglePanel(id)}
				>
					{panelName(id)}
				</button>
			{/each}
		</div>
	</div>

	{#if track}
		{#if shown('cover')}
			<figure class="cover">
				<div class="art">
					{#if track.coverId !== null}<img src={coverUrl(track.coverId, 512)} alt="" />{/if}
					{#if arranged.cover === 'instrument'}
						<!-- The crosshair of an instrument's glass; a drawing over
						     the picture, not a part of it. -->
						<span class="sight" aria-hidden="true"></span>
					{/if}
				</div>
				{#if arranged.plates}<figcaption class="plate">{t('plate.cover')}</figcaption>{/if}
			</figure>
		{/if}
		<div class="text">
			{#if arranged.plates && track.trackNumber}
				<span class="plate">{t('plate.track', { n: track.trackNumber })}</span>
			{/if}
			<h1 class="ellipsis">{track.title ?? fileName(track.path)}</h1>
			<p class="artist ellipsis">{track.artist ?? t('track.unknownArtist')}</p>
			{#if track.album}
				<p class="muted ellipsis">{track.album}{track.year ? ` · ${track.year}` : ''}</p>
			{/if}
			{#if arranged.plates && !shown('details') && format}
				<!-- The one line of the signal an instrument panel prints under
				     the title; the full account is the details panel's. -->
				<p class="numeric format-line ellipsis">{format}</p>
			{/if}

			<div class="timeline" class:away={dialled || arranged.nowPlaying === 'immersive'}>
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

			{#if shown('details')}
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
			{/if}
		</div>
		{#if shown('lyrics')}
			<section class="words">
				<h2 class="label">{t('lyrics.title')}</h2>
				<LyricsPanel large />
			</section>
		{/if}
		{#if dialled}
			<figure class="dial-box">
				<div class="dial-face"><PositionDial position={player.position} {duration} /></div>
				{#if arranged.plates}<figcaption class="plate">{t('plate.position')}</figcaption>{/if}
			</figure>
		{/if}
		{#if shown('visualizer')}
			<figure class="visualizer">
				<Spectrum height={arranged.nowPlaying === 'instrument' ? 120 : 170} />
				{#if arranged.plates}<figcaption class="plate">{t('plate.spectrum')}</figcaption>{/if}
			</figure>
		{/if}
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

	/* With the cover away there is no second column for the words to sit
	   beside, and a text block in an `auto` column would size itself to its
	   longest line. */
	.now.wide {
		grid-template-columns: minmax(0, 1fr);
	}

	.bar {
		grid-column: 1 / -1;
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 16px;
	}

	.panels {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.toggle {
		padding: 3px 9px;
		border: 1px solid var(--onsa-border);
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-secondary);
		font-size: 11px;
		cursor: pointer;
	}

	.toggle:hover {
		color: var(--onsa-text-primary);
	}

	/* A panel that is showing is lit the way every other thing you can
	   adjust is lit, so the row reads as a set of switches rather than as
	   four buttons that do something. */
	.toggle.on {
		border-color: var(--onsa-role-adjustable);
		color: var(--onsa-role-adjustable);
	}

	figure {
		margin: 0;
	}

	.cover,
	.dial-box,
	.visualizer {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}

	.visualizer {
		grid-column: 1 / -1;
		align-items: stretch;
	}

	.visualizer .plate {
		align-self: center;
	}

	.format-line {
		color: var(--onsa-text-secondary);
		font-family: var(--onsa-font-numeric);
		font-size: 13px;
		letter-spacing: 0.04em;
	}

	.timeline.away {
		display: none;
	}

	/* A name plate: printed on a dark strip, the way an instrument panel
	   labels what each instrument is. */
	.plate {
		align-self: flex-start;
		padding: 2px 8px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		color: var(--onsa-text-silkscreen);
		font-family: var(--onsa-font-label);
		font-size: 11px;
		letter-spacing: 0.18em;
		text-transform: uppercase;
		white-space: nowrap;
	}

	.cover .plate,
	.dial-box .plate {
		align-self: center;
	}

	/* The cover in an instrument's bezel, with its glass crossed. */
	.now[data-cover='instrument'] .art {
		position: relative;
		border: 8px solid var(--onsa-surface-well);
		box-shadow: 0 0 0 2px var(--onsa-surface-line);
		border-radius: var(--onsa-radius-md);
	}

	.sight {
		position: absolute;
		inset: 12%;
		pointer-events: none;
		background:
			linear-gradient(var(--onsa-text-primary), var(--onsa-text-primary)) center / 100% 1px no-repeat,
			linear-gradient(var(--onsa-text-primary), var(--onsa-text-primary)) center / 1px 100% no-repeat;
		opacity: 0.45;
	}

	.sight::after {
		content: '';
		position: absolute;
		left: 50%;
		top: 50%;
		width: 46px;
		height: 46px;
		margin: -23px 0 0 -23px;
		border: 1px solid var(--onsa-text-primary);
		border-radius: 50%;
	}

	/* Instruments: the cover in its frame beside the title and the words,
	   and the position and the spectrum as instruments underneath. */
	.now[data-arrangement='instrument'],
	.now[data-arrangement='immersive'] {
		align-content: stretch;
	}

	.now[data-arrangement='instrument'] {
		grid-template-columns: auto minmax(0, 1fr);
		grid-template-areas:
			'bar bar'
			'cover text'
			'cover words'
			'dial viz';
		/* The words keep three lines at the least; on a short window the
		   page scrolls rather than the words disappearing. */
		grid-template-rows: auto auto minmax(110px, 1fr) auto;
	}

	.now[data-arrangement='instrument'] > .bar {
		grid-area: bar;
	}

	.now[data-arrangement='instrument'] > .cover {
		grid-area: cover;
		align-self: start;
	}

	.now[data-arrangement='instrument'] > .text {
		grid-area: text;
		align-self: start;
	}

	.now[data-arrangement='instrument'] > .words {
		grid-area: words;
		grid-column: auto;
		/* The row takes what is left; the words scroll inside it. */
		height: auto;
		min-height: 90px;
	}

	.now[data-arrangement='instrument'] > .dial-box {
		grid-area: dial;
	}

	.now[data-arrangement='instrument'] > .visualizer {
		grid-area: viz;
		grid-column: auto;
	}

	.now[data-arrangement='instrument'] .art {
		width: min(220px, 26cqw);
		height: min(220px, 26cqw);
	}

	.dial-face {
		width: 138px;
		padding: 8px;
	}

	/* Without the cover there is no left column for the cover to fill; the
	   dial keeps it. */
	.now[data-arrangement='instrument'].wide {
		grid-template-areas:
			'bar bar'
			'text text'
			'words words'
			'dial viz';
	}

	/* Layar penuh: the picture large, the title over the words beside it. */
	.now[data-arrangement='immersive'] {
		grid-template-columns: auto minmax(0, 1fr);
		grid-template-areas:
			'bar bar'
			'cover text'
			'cover words'
			'viz viz';
		grid-template-rows: auto auto minmax(0, 1fr) auto;
		gap: 18px 40px;
	}

	.now[data-arrangement='immersive'] > .bar {
		grid-area: bar;
	}

	.now[data-arrangement='immersive'] > .cover {
		grid-area: cover;
	}

	.now[data-arrangement='immersive'] > .text {
		grid-area: text;
		align-self: end;
	}

	.now[data-arrangement='immersive'] > .words {
		grid-area: words;
		grid-column: auto;
		height: auto;
		min-height: 120px;
	}

	.now[data-arrangement='immersive'] > .visualizer {
		grid-area: viz;
	}

	.now[data-arrangement='immersive'] .art {
		width: min(52cqw, 62vh);
		height: min(52cqw, 62vh);
	}

	.now[data-arrangement='immersive'] h1 {
		font-size: 40px;
	}

	.now[data-arrangement='immersive'].wide {
		grid-template-areas:
			'bar bar'
			'text text'
			'words words'
			'viz viz';
	}

	/* Narrow, every arrangement stacks: one column, in reading order. */
	@container content (max-width: 700px) {
		.now[data-arrangement] {
			grid-template-columns: minmax(0, 1fr);
			grid-template-areas: none;
			grid-template-rows: none;
		}

		.now[data-arrangement] > * {
			grid-area: auto;
		}

		.now[data-arrangement] .art {
			width: min(240px, 60cqw);
			height: min(240px, 60cqw);
		}
	}

	/* The words get a box of their own rather than the whole page's
	   scrolling: the line being sung has to stay where the eye expects it,
	   and it cannot if the page moves underneath. */
	.words {
		grid-column: 1 / -1;
		font-family: var(--onsa-font-lyrics);
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		gap: 6px;
		min-width: 0;
		height: min(42vh, 360px);
	}

	.words h2 {
		margin: 0;
		font-size: 11px;
		font-weight: 500;
	}

	.back {
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
		font-family: var(--onsa-font-display);
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
