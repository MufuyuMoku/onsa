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
	import { toneColor, tonePosition } from '$lib/analysis.svelte';
	import { coverUrl, seek } from '$lib/backend';
	import { navigate } from '$lib/app.svelte';
	import { clock, db, fileName, rate } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { player } from '$lib/player.svelte';
	import { arrangement } from '$lib/arrangement.svelte';
	import { PANELS, shown, togglePanel, type PanelId } from '$lib/panels.svelte';
	import Icon from './Icon.svelte';
	import LyricsPanel from './LyricsPanel.svelte';
	import DepthScale from './DepthScale.svelte';
	import PositionDial from './PositionDial.svelte';
	import { ornament } from '$lib/ornament.svelte';
	import { activeTheme } from '$lib/theme/index.svelte';
	import Spectrum from './Spectrum.svelte';

	const arranged = $derived(arrangement());
	/** Sizes the layout sets for itself, if any (SPEC M16). */
	const coverSize = $derived(arranged.sizes?.cover ?? null);
	const titleSize = $derived(arranged.sizes?.title ?? null);
	const lyricsSize = $derived(arranged.sizes?.lyrics ?? null);
	/** Whether the position is a dial of its own rather than the bar. */
	const dialled = $derived(arranged.position === 'dial');
	/** Whether it is a depth down the side, which seeks as the bar does. */
	const sounded = $derived(arranged.position === 'depth');
	/** What the theme draws here for its look alone (SPEC section 9.1). */
	const decor = $derived(activeTheme()?.decor ?? []);
	const overTitle = $derived(ornament('title'));
	const overSpectrum = $derived(ornament('spectrum'));
	/** A display's backlight, leaned with the tone where the theme asks. */
	const backlight = $derived(toneColor('backlight', activeTheme()?.color.lit.lcd ?? '#B4C49B'));
	const playing = $derived(player.snapshot?.state === 'playing');

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
	class:sized-cover={coverSize != null}
	class:sized-title={titleSize != null}
	style:--np-cover={coverSize != null ? `${coverSize}px` : undefined}
	style:--np-title={titleSize != null ? `${titleSize}px` : undefined}
	class:sized-lyrics={lyricsSize != null}
	style:--np-lyrics={lyricsSize != null ? `${lyricsSize}px` : undefined}
	class:plated={arranged.plates}
	class:dialled
	class:playing
>
	{#if decor.includes('marine-snow')}
		<!-- Specks drifting down, for the look alone: hidden from screen
		     readers, and still in power saving and under reduced motion. -->
		<div class="snow drifts" aria-hidden="true"><i></i><i></i></div>
	{/if}

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
				{#if decor.includes('lure')}
					<!-- A lure of light on its line, hanging over the cover's top
					     corner, the light the corner is lit by. -->
					<svg class="lure" viewBox="0 0 260 120" aria-hidden="true">
						<path d="M170 0 C 178 22, 184 42, 188 63" />
						<circle class="halo wide" cx="188" cy="70" r="44" />
						<circle class="halo" cx="188" cy="70" r="22" />
						<circle class="bulb" cx="188" cy="70" r="7" />
					</svg>
				{/if}
				<div class="art" class:drifts={arranged.cover === 'disc'}>
					{#if track.coverId !== null}<img src={coverUrl(track.coverId, 512)} alt="" />{/if}
					{#if arranged.cover === 'lit'}
						<!-- Lit from the top corner; the far corner is darkened by
						     half at the most, so the picture stays clear. -->
						<span class="shade" aria-hidden="true"></span>
					{/if}
					{#if arranged.cover === 'instrument'}
						<!-- The crosshair of an instrument's glass; a drawing over
						     the picture, not a part of it. -->
						<span class="sight" aria-hidden="true"></span>
					{/if}
				</div>
				{#if arranged.plates}<figcaption class="plate">{t('plate.cover')}</figcaption>{/if}
			</figure>
		{/if}
		{#if arranged.nowPlaying === 'pocket'}
			<!-- The display of a pocket player: the track, the time, the
			     format. The same facts the transport says, on a screen. -->
			<div class="lcd lcd-box" style:--onsa-lcd-lit={backlight}>
				<span class="lcd-small">{track.trackNumber ? t('lcd.track', { n: String(track.trackNumber).padStart(2, '0') }) : ''}</span>
				<span class="lcd-time numeric">{clock(player.position)}</span>
				<span class="lcd-small">{signal?.source?.codec?.toUpperCase() ?? ''} {signal?.source ? rate(signal.source.sampleRate) : ''}</span>
			</div>
		{/if}
		<div class="text">
			{#if arranged.plates && track.trackNumber}
				<span class="plate">{t('plate.track', { n: track.trackNumber })}</span>
			{/if}
			{#if overTitle}
				<span class="over-title" aria-hidden="true">{overTitle}</span>
			{/if}
			<h1 class="ellipsis">{track.title ?? fileName(track.path)}</h1>
			<p class="artist ellipsis">{track.artist ?? t('track.unknownArtist')}</p>
			{#if track.album}
				<p class="muted ellipsis">{track.album}{track.year ? ` · ${track.year}` : ''}</p>
			{/if}
			{#if !['classic', 'immersive'].includes(arranged.nowPlaying) && !shown('details') && format}
				<!-- The one line of the signal an instrument panel prints under
				     the title; the full account is the details panel's. -->
				<p class="numeric format-line ellipsis">{format}</p>
			{/if}

			<div
				class="timeline"
				class:away={arranged.nowPlaying !== 'classic'}
			>
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
		{#if sounded}
			<div class="depth-box"><DepthScale position={player.position} {duration} /></div>
		{/if}
		{#if dialled}
			<figure class="dial-box">
				<div class="dial-face"><PositionDial position={player.position} {duration} /></div>
				{#if arranged.plates}<figcaption class="plate">{t('plate.position')}</figcaption>{/if}
			</figure>
		{/if}
		{#if shown('visualizer')}
			<figure class="visualizer">
				{#if overSpectrum}
					<span class="over-spectrum" aria-hidden="true">{overSpectrum}</span>
				{/if}
				<Spectrum
					height={arranged.sizes?.spectrum ??
						(['instrument', 'memo'].includes(arranged.nowPlaying) ? 120 : 170)}
				/>
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
		/* The row takes what is left; the words scroll inside it. */
		height: auto;
		min-height: 90px;
	}

	.now[data-arrangement='instrument'] > .dial-box {
		grid-area: dial;
	}

	.now[data-arrangement='instrument'] > .visualizer {
		grid-area: viz;
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

	/* Menyelam: the cover and a large title side by side, the words under
	   them, the spectrum along the bottom, and the depth down the right. */
	.now[data-arrangement='dive'] {
		align-content: stretch;
		grid-template-columns: auto minmax(0, 1fr) 130px;
		grid-template-areas:
			'bar bar bar'
			'cover text depth'
			'words words depth'
			'viz viz depth';
		grid-template-rows: auto auto minmax(110px, 1fr) auto;
		gap: 18px 30px;
	}

	.now[data-arrangement='dive'] > .bar {
		grid-area: bar;
	}

	.now[data-arrangement='dive'] > .cover {
		grid-area: cover;
		margin-top: 40px;
	}

	.now[data-arrangement='dive'] > .text {
		grid-area: text;
		align-self: end;
	}

	.now[data-arrangement='dive'] > .words {
		grid-area: words;
		height: auto;
		min-height: 0;
	}

	.now[data-arrangement='dive'] > .visualizer {
		grid-area: viz;
	}

	.now[data-arrangement='dive'] > .depth-box {
		grid-area: depth;
		min-height: 0;
	}

	.now[data-arrangement='dive'] .art {
		width: min(220px, 24cqw);
		height: min(220px, 24cqw);
		border-radius: 0;
		box-shadow: none;
	}

	.now[data-arrangement='dive'] h1 {
		font-size: clamp(36px, 5cqw, 60px);
		font-weight: 300;
		line-height: 0.95;
	}

	.now[data-arrangement='dive'] .artist {
		font-family: var(--onsa-font-display);
		font-style: italic;
		font-size: 22px;
		color: var(--onsa-text-secondary);
	}

	.now[data-arrangement='dive'] .words {
		font-size: 19px;
	}

	.now[data-arrangement='dive'].wide {
		grid-template-areas:
			'bar bar depth'
			'text text depth'
			'words words depth'
			'viz viz depth';
	}

	.over-title {
		font-family: var(--onsa-font-label);
		font-size: 10px;
		letter-spacing: 0.2em;
		color: var(--onsa-lit-ornament);
		text-transform: uppercase;
	}

	.now {
		position: relative;
		isolation: isolate;
	}

	/* Specks of marine snow: two layers of points drifting down at two
	   speeds, so they read as depth rather than as a pattern. */
	.snow {
		position: absolute;
		inset: 0;
		z-index: -1;
		pointer-events: none;
		overflow: hidden;
	}

	.snow i {
		position: absolute;
		inset: -600px 0 0;
		background-image:
			radial-gradient(1.5px 1.5px at 12% 18%, rgb(191 239 255 / 0.4), transparent),
			radial-gradient(2px 2px at 38% 42%, rgb(191 239 255 / 0.25), transparent),
			radial-gradient(1.5px 1.5px at 61% 9%, rgb(191 239 255 / 0.3), transparent),
			radial-gradient(1.5px 1.5px at 84% 33%, rgb(191 239 255 / 0.2), transparent),
			radial-gradient(2.5px 2.5px at 72% 71%, rgb(191 239 255 / 0.12), transparent),
			radial-gradient(1.5px 1.5px at 27% 83%, rgb(191 239 255 / 0.3), transparent);
		background-size: 100% 600px;
		animation: drift 90s linear infinite;
	}

	.snow i + i {
		background-size: 70% 420px;
		opacity: 0.6;
		animation-duration: 140s;
	}

	@keyframes drift {
		to {
			transform: translateY(600px);
		}
	}

	.cover {
		position: relative;
	}

	/* The bulb sits on the cover's top right corner: the drawing is moved
	   so its centre (188, 70) lands there. */
	.lure {
		position: absolute;
		top: -70px;
		right: -72px;
		width: 260px;
		height: 120px;
		pointer-events: none;
		z-index: 2;
	}

	.lure path {
		fill: none;
		stroke: var(--onsa-surface-line);
		stroke-width: 1.5;
	}

	.lure .bulb {
		fill: var(--onsa-role-position);
	}

	.lure .halo {
		fill: var(--onsa-role-active);
		opacity: 0.18;
	}

	.lure .halo.wide {
		opacity: 0.06;
	}

	.now[data-cover='lit'] .art {
		position: relative;
	}

	.shade {
		position: absolute;
		inset: 0;
		pointer-events: none;
		background: radial-gradient(
			circle at 92% 0%,
			transparent 0 30%,
			color-mix(in srgb, var(--onsa-surface-well) 50%, transparent) 78%
		);
	}

	/* Memo: the cover stuck on beside one card that holds the title and the
	   words, and the spectrum on a card of its own under them. The two
	   halves of the card are two elements drawn as one. */
	.now[data-arrangement='memo'] {
		align-content: stretch;
		grid-template-columns: auto minmax(0, 1fr);
		grid-template-areas:
			'bar bar'
			'cover text'
			'cover words'
			'viz viz';
		grid-template-rows: auto auto minmax(110px, 1fr) auto;
		row-gap: 0;
		column-gap: 20px;
	}

	.now[data-arrangement='memo'] > .bar {
		grid-area: bar;
		margin-bottom: 14px;
	}

	.now[data-arrangement='memo'] > .cover {
		grid-area: cover;
		align-self: start;
	}

	.now[data-arrangement='memo'] > .text,
	.now[data-arrangement='memo'] > .words,
	.now[data-arrangement='memo'] > .visualizer {
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
		box-shadow: 2px 3px 0 color-mix(in srgb, var(--onsa-edge-dark) 35%, transparent);
	}

	.now[data-arrangement='memo'] > .text {
		grid-area: text;
		align-self: stretch;
		padding: 14px 18px 4px;
		border-bottom: 0;
		box-shadow: none;
	}

	.now[data-arrangement='memo'] > .words {
		grid-area: words;
		height: auto;
		min-height: 0;
		padding: 0 4px 10px;
		border-top: 0;
	}

	.now[data-arrangement='memo'] > .visualizer {
		grid-area: viz;
		margin-top: 14px;
		padding: 10px 12px;
	}

	.now[data-arrangement='memo'] .art {
		width: min(200px, 24cqw);
		height: min(200px, 24cqw);
	}

	.now[data-arrangement='memo'].wide {
		grid-template-areas:
			'bar bar'
			'text text'
			'words words'
			'viz viz';
	}

	/* A photograph stuck on: a paper border and a hard shadow. */
	.now[data-cover='photo'] .art {
		border: 6px solid var(--onsa-surface-body);
		border-radius: 0;
		box-shadow: 2px 3px 0 color-mix(in srgb, var(--onsa-edge-dark) 45%, transparent);
	}

	.over-spectrum {
		align-self: stretch;
		font-family: var(--onsa-font-label);
		font-size: 11px;
		letter-spacing: 0.14em;
		color: var(--onsa-lit-ornament);
		text-transform: uppercase;
	}

	/* Padang: the title and the words floating in the sky, the cover seen
	   through a door standing on the ground, and the spectrum growing out of
	   the ground as grass. */
	.now[data-arrangement='meadow'] {
		align-content: stretch;
		grid-template-columns: minmax(0, 1fr) auto;
		grid-template-areas:
			'bar bar'
			'text cover'
			'words cover'
			'viz viz';
		grid-template-rows: auto auto minmax(110px, 1fr) auto;
		gap: 10px 40px;
		padding-bottom: 0;
	}

	.now[data-arrangement='meadow'] > .bar {
		grid-area: bar;
	}

	.now[data-arrangement='meadow'] > .text {
		grid-area: text;
		align-self: start;
	}

	.now[data-arrangement='meadow'] > .words {
		grid-area: words;
		height: auto;
		min-height: 0;
		font-size: 19px;
	}

	.now[data-arrangement='meadow'] > .cover {
		grid-area: cover;
		align-self: end;
		margin-right: 70px;
	}

	/* The ground, and the grass growing from it. */
	.now[data-arrangement='meadow'] > .visualizer {
		grid-area: viz;
		margin: 0 -24px;
		padding: 0 24px;
		background: linear-gradient(
			180deg,
			color-mix(in srgb, var(--onsa-lit-ground) 55%, white) 0%,
			var(--onsa-lit-ground) 100%
		);
	}

	.now[data-arrangement='meadow'] h1 {
		font-size: clamp(36px, 5cqw, 58px);
		font-weight: 400;
		line-height: 1;
		text-shadow: 0 0 18px rgb(255 255 255 / 0.9);
	}

	.now[data-arrangement='meadow'] .artist {
		color: var(--onsa-text-primary);
	}

	.now[data-arrangement='meadow'].wide {
		grid-template-areas:
			'bar bar'
			'text text'
			'words words'
			'viz viz';
	}

	/* A door standing open on its own: the frame round the picture, the
	   leaf swung out to one side, and its shadow on the grass. */
	.now[data-cover='door'] .cover {
		position: relative;
	}

	.now[data-cover='door'] .art {
		width: min(156px, 16cqw);
		height: min(256px, 26cqw);
		padding: 12px;
		border: 2px solid var(--onsa-lit-paper-edge);
		border-radius: 0;
		background: var(--onsa-lit-paper);
		box-shadow: 12px 6px 0 color-mix(in srgb, var(--onsa-lit-ground) 35%, transparent);
	}

	.now[data-cover='door'] .art img {
		box-shadow: inset 0 0 30px rgb(255 255 255 / 0.9);
	}

	.now[data-cover='door'] .cover::before {
		content: '';
		position: absolute;
		right: calc(100% - 2px);
		bottom: 30px;
		width: 30%;
		height: 88%;
		border: 2px solid var(--onsa-lit-paper-edge);
		background: linear-gradient(90deg, color-mix(in srgb, var(--onsa-lit-paper) 85%, black), var(--onsa-lit-paper));
		transform: skewY(-14deg);
		transform-origin: right bottom;
		pointer-events: none;
	}

	.now[data-cover='door'] .cover::after {
		content: '';
		width: 110%;
		height: 8px;
		margin-top: -6px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--onsa-lit-ground) 60%, black);
		opacity: 0.35;
		filter: blur(2px);
	}

	/* Pemutar Saku: the disc to one side; the display, the title and the
	   words beside it; the spectrum on a wide display along the bottom. */
	.now[data-arrangement='pocket'] {
		align-content: stretch;
		grid-template-columns: auto minmax(0, 1fr);
		grid-template-areas:
			'bar bar'
			'cover lcd'
			'cover text'
			'cover words'
			'viz viz';
		grid-template-rows: auto auto auto minmax(100px, 1fr) auto;
		gap: 10px 24px;
	}

	.now[data-arrangement='pocket'] > .bar {
		grid-area: bar;
	}

	.now[data-arrangement='pocket'] > .cover {
		grid-area: cover;
		align-self: center;
	}

	.now[data-arrangement='pocket'] > .lcd-box {
		grid-area: lcd;
	}

	.now[data-arrangement='pocket'] > .text {
		grid-area: text;
	}

	/* The words in a box of clear pink plastic. */
	.now[data-arrangement='pocket'] > .words {
		grid-area: words;
		height: auto;
		min-height: 0;
		padding: 8px 6px 4px;
		border: 1px solid rgb(255 255 255 / 0.8);
		border-radius: 14px;
		background: linear-gradient(
			160deg,
			color-mix(in srgb, var(--onsa-role-position) 30%, white),
			color-mix(in srgb, var(--onsa-role-position) 14%, white)
		);
		box-shadow: inset 0 2px 0 rgb(255 255 255 / 0.7);
	}

	.now[data-arrangement='pocket'] > .visualizer {
		grid-area: viz;
	}

	.now[data-arrangement='pocket'] h1 {
		font-size: 38px;
		font-weight: 700;
	}

	.now[data-arrangement='pocket'] .artist {
		color: var(--onsa-text-secondary);
		font-weight: 600;
	}

	.now[data-arrangement='pocket'].wide {
		grid-template-areas:
			'bar bar'
			'lcd lcd'
			'text text'
			'words words'
			'viz viz';
	}

	.lcd-box {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		padding: 10px 14px;
		border-radius: 8px;
	}

	.lcd-small {
		font-size: 12px;
		letter-spacing: 0.12em;
		white-space: nowrap;
	}

	.lcd-time {
		font-size: 34px;
		font-weight: 700;
		letter-spacing: 0.08em;
	}

	/* The label of a disc in a chromed ring: the cover itself, with a hole
	   in the middle and a rainbow across its face. It turns slowly while
	   the song plays, and stops when it pauses. */
	.now[data-cover='disc'] .art {
		position: relative;
		width: min(236px, 26cqw);
		height: min(236px, 26cqw);
		padding: 15px;
		border: 1px solid var(--onsa-edge-dark);
		border-radius: 50%;
		background: linear-gradient(
			180deg,
			var(--onsa-edge-light) 0%,
			color-mix(in srgb, var(--onsa-edge-dark) 30%, var(--onsa-edge-light)) 45%,
			color-mix(in srgb, var(--onsa-edge-dark) 55%, var(--onsa-edge-light)) 52%,
			color-mix(in srgb, var(--onsa-edge-dark) 15%, var(--onsa-edge-light)) 100%
		);
		box-shadow: inset 0 1px 0 var(--onsa-edge-light);
		overflow: visible;
	}

	.now[data-cover='disc'] .art img {
		border-radius: 50%;
		mask: radial-gradient(circle, transparent 0 9%, #000 9.5%);
	}

	.now.playing[data-cover='disc'] .art img {
		animation: turn 12s linear infinite;
	}

	@keyframes turn {
		to {
			transform: rotate(360deg);
		}
	}

	/* The rainbow and the gloss lie on the disc's face; they do not turn. */
	.now[data-cover='disc'] .art::after {
		content: '';
		position: absolute;
		inset: 15px;
		border-radius: 50%;
		pointer-events: none;
		background:
			linear-gradient(125deg, rgb(255 255 255 / 0.5) 0 30%, transparent 45%),
			conic-gradient(
				from 20deg,
				rgb(230 240 255 / 0.25),
				rgb(255 217 243 / 0.25),
				rgb(217 255 244 / 0.25),
				rgb(255 244 201 / 0.25),
				rgb(226 217 255 / 0.25),
				rgb(230 240 255 / 0.25)
			);
		mask: radial-gradient(circle, transparent 0 9%, #000 9.5%);
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

	/* A layout's own sizes win over the style it is drawn in (SPEC M16):
	   the most the cover takes, and the size of the title. A narrow window
	   still shrinks the cover. */
	.now.sized-cover[data-arrangement][data-cover] .art {
		width: min(var(--np-cover), 32cqw);
		height: min(var(--np-cover), 32cqw);
	}

	.now.sized-title[data-arrangement] h1 {
		font-size: var(--np-title);
	}

	.now.sized-lyrics[data-arrangement] .words {
		height: min(42vh, var(--np-lyrics));
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

		.now.sized-cover[data-arrangement][data-cover] .art {
			width: min(var(--np-cover), 60cqw);
			height: min(var(--np-cover), 60cqw);
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
