<!--
	The downloader (SPEC §7).

	The first thing this page does is say what Onsa would fetch, from where,
	and roughly how large it is — and then wait. Nothing is fetched until the
	button is pressed (SPEC §7.1). What comes back is checked against the
	checksum the release published before it is written anywhere.
-->
<script lang="ts">
	import {
		binariesStatus,
		binariesUseSystem,
		binaryInstall,
		binaryStop,
		downloadProbe,
		downloadQueue,
		downloadStart,
		downloadStop,
		EVENTS,
		failureKey,
		on,
		urlTroubleOf,
		type Binaries,
		type BinaryStatus,
		type DownloadFormat,
		type DownloadQueue,
		type FetchProgress,
		type Probe,
		type QueueItem,
		type UrlTrouble
	} from '$lib/backend';
	import { clock } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import Icon from './Icon.svelte';

	let programs = $state<BinaryStatus[]>([]);
	let progress = $state<FetchProgress | null>(null);
	let failure = $state<MessageKey | null>(null);
	let useSystem = $state(true);
	let canConvert = $state(false);
	let busy = $state(false);

	/** The URL half of the page. */
	let url = $state('');
	let looking = $state(false);
	let probe = $state<Probe | null>(null);
	/**
	 * What went wrong with the URL, kept apart from the page's own failure.
	 *
	 * The programs list is read again on every download event, and each of
	 * those reads cleared the shared one — so the answer to a bad URL was
	 * wiped a moment after it appeared, and nothing was ever seen.
	 */
	let urlTrouble = $state<UrlTrouble | null>(null);
	let picked = $state<Record<string, boolean>>({});
	let format = $state<DownloadFormat>('original');
	let queue = $state<DownloadQueue | null>(null);

	$effect(() => {
		void look();
		void readQueue();
		const fetching = on<FetchProgress>(EVENTS.binary, (next) => (progress = next));
		// The queue says only that it changed; what it changed to is asked
		// for, so a burst of progress lines is one read rather than many.
		const going = on(EVENTS.downloads, () => void readQueue());
		return () => {
			void fetching.then((off) => off());
			void going.then((off) => off());
		};
	});

	async function readQueue(): Promise<void> {
		try {
			queue = await downloadQueue();
		} catch {
			// A queue that cannot be read leaves the last one standing.
		}
	}

	/** What is at the URL, without fetching any of it. */
	async function lookAtUrl(): Promise<void> {
		looking = true;
		probe = null;
		urlTrouble = null;
		try {
			const found = await downloadProbe(url.trim());
			probe = found;
			picked = Object.fromEntries(found.entries.map((one) => [one.url, true]));
		} catch (error) {
			urlTrouble = urlTroubleOf(error) ?? { code: 'other', said: null };
		} finally {
			looking = false;
		}
	}

	const chosen = $derived(
		(probe?.entries ?? []).filter((one) => picked[one.url] ?? true)
	);

	async function start(): Promise<void> {
		try {
			queue = await downloadStart(
				chosen.map((one) => ({ url: one.url, title: one.title })),
				format
			);
			probe = null;
			url = '';
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
	}

	const STATE: Record<QueueItem['state'], MessageKey> = {
		waiting: 'downloads.waiting',
		running: 'downloads.running',
		done: 'downloads.done',
		failed: 'downloads.failed',
		stopped: 'downloads.wasStopped'
	};

	/**
	 * Reading the list means running each program to ask its version, which
	 * takes a moment. Two reads can therefore be in the air at once — the
	 * one the page started with, and the one a switch just asked for — and
	 * the slower one must not land on top of the newer.
	 */
	let asked = 0;

	async function look(): Promise<void> {
		const mine = ++asked;
		try {
			const answer = await binariesStatus();
			if (mine === asked) take(answer);
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
	}

	/** What came back: the programs, and the setting as it is stored. */
	function take(answer: Binaries): void {
		programs = answer.programs;
		useSystem = answer.useSystem;
		canConvert = answer.canConvert;
		// A format that cannot be made is not a format to be left chosen.
		if (!canConvert) format = 'original';
	}

	async function install(key: string): Promise<void> {
		busy = true;
		progress = { key, done: 0, total: null, finished: false, failed: null };
		try {
			const after = await binaryInstall(key);
			asked += 1;
			programs = programs.map((one) => (one.key === after.key ? after : one));
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		} finally {
			busy = false;
			progress = null;
		}
	}

	async function toggleSystem(allowed: boolean): Promise<void> {
		useSystem = allowed;
		const mine = ++asked;
		try {
			const answer = await binariesUseSystem(allowed);
			if (mine === asked) take(answer);
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
	}

	const WHERE: Record<string, MessageKey> = {
		managed: 'programs.fromManaged',
		chosen: 'programs.fromChosen',
		system: 'programs.fromSystem',
		systemAnyway: 'programs.fromSystemAnyway'
	};

	const WHY: Record<string, MessageKey> = {
		notAUrl: 'downloads.notAUrl',
		noYtDlp: 'downloads.needYtDlp',
		refused: 'downloads.refused',
		failed: 'downloads.failed',
		noFfmpeg: 'downloads.noFfmpeg',
		noPlayableFormat: 'downloads.noPlayableFormat',
		unsupportedSite: 'downloads.unsupportedSite',
		needsSignIn: 'downloads.needsSignIn',
		geoBlocked: 'downloads.geoBlocked',
		gone: 'downloads.gone',
		tooManyAsks: 'downloads.tooManyAsks',
		siteRefused: 'downloads.siteRefused',
		cannotReach: 'downloads.cannotReach',
		siteBroken: 'downloads.siteBroken',
		other: 'downloads.otherTrouble',
		cannotRun: 'downloads.cannotRun',
		stopped: 'downloads.stopped',
		unreachable: 'downloads.unreachable',
		tooLarge: 'downloads.tooLarge',
		noChecksum: 'downloads.noChecksum',
		checksum: 'downloads.checksumFailed',
		cannotWrite: 'downloads.cannotWrite',
		unreadable: 'downloads.unreadable'
	};

	/** A size a person can read, from bytes. */
	function size(bytes: number | null | undefined): string {
		if (!bytes) return '';
		const mb = bytes / (1024 * 1024);
		return `${mb < 10 ? mb.toFixed(1) : Math.round(mb)} MB`;
	}

	/** Which programs still have to arrive before anything can be downloaded. */
	const missing = $derived(programs.filter((one) => one.required && !one.present));
	const ready = $derived(missing.length === 0);
</script>

<div class="downloads">
	<header class="head">
		<h1>{t('nav.downloads')}</h1>
	</header>

	<p class="muted what">{t('downloads.intro')}</p>
	<p class="muted what">{t('downloads.rights')}</p>

	<section class="programs">
		<h2 class="label">{t('downloads.needs')}</h2>
		<p class="muted note">{t('downloads.needsWhat')}</p>

		{#if programs.length === 0}
			<p class="muted note">{t('downloads.reading')}</p>
		{/if}

		<ul class="list">
			{#each programs as program (program.key)}
				<li class="program" class:missing={program.required && !program.present}>
					<span class="name">{program.key}</span>

					{#if program.present}
						<span class="state ok">{t('programs.installed')}</span>
						<span class="muted where">
							{t(WHERE[program.from ?? 'system'] ?? 'programs.fromSystem')}
							{#if program.version}· <span class="numeric">{program.version}</span>{/if}
						</span>
						{#if program.from === 'systemAnyway'}
							<p class="muted note anyway">{t('programs.systemAnywayWhat')}</p>
						{/if}
					{:else}
						<span class="state" class:fault-text={program.required}>
							{t('programs.missing')}
						</span>
						<span class="muted where">
							{#if program.installable}
								{t('downloads.fromRelease')}
								<span class="numeric">· {size(program.aboutBytes)}</span>
							{:else}
								{t('downloads.installYourself')}
							{/if}
						</span>
					{/if}

					<span class="action">
						{#if program.installable}
							<button
								type="button"
								class="btn"
								disabled={busy}
								onclick={() => install(program.key)}
							>
								{program.present ? t('downloads.update') : t('downloads.agree')}
							</button>
						{/if}
					</span>

					{#if program.url}
						<span class="source numeric muted ellipsis" title={program.url}>{program.url}</span>
					{/if}

					{#if !program.installable && !program.present}
						<span class="source muted">{t('downloads.ffmpegWhere')}</span>
					{/if}

					{#if progress && progress.key === program.key && !progress.finished}
						<!-- Until the far end says how large it is there is nothing
						     to fill, and a full bar would read as finished. The
						     count beside it carries the news in the meantime. -->
						<span class="bar" aria-hidden="true">
							<span
								class="fill"
								style:width={progress.total
									? `${Math.round((progress.done / progress.total) * 100)}%`
									: '0%'}
							></span>
						</span>
						<span class="numeric muted done">
							{size(progress.done)}{progress.total ? ` / ${size(progress.total)}` : ''}
						</span>
						<button type="button" class="btn" onclick={() => binaryStop()}>
							{t('downloads.stop')}
						</button>
					{/if}
				</li>
			{/each}
		</ul>

		<label class="row">
			<input
				type="checkbox"
				checked={useSystem}
				onchange={(event) => toggleSystem(event.currentTarget.checked)}
			/>
			<span>{t('downloads.useSystem')}</span>
		</label>
		<p class="muted note">{t('downloads.useSystemWhat')}</p>

		{#if progress?.failed}
			<p class="fault-text note">{t(WHY[progress.failed] ?? 'downloads.unreachable')}</p>
		{/if}
		{#if failure}<p class="fault-text note">{t(failure)}</p>{/if}
	</section>

	<section class="fetch">
		<h2 class="label">{t('downloads.fromUrl')}</h2>

		{#if !ready}
			<p class="muted note">
				<Icon name="info" />
				{t('downloads.needYtDlp')}
			</p>
		{:else}
			<div class="row">
				<input
					class="field grow numeric"
					type="text"
					placeholder={t('downloads.urlPlaceholder')}
					bind:value={url}
					onkeydown={(event) => {
						if (event.key === 'Enter') void lookAtUrl();
					}}
				/>
				<button type="button" class="btn" disabled={looking || url.trim() === ''} onclick={lookAtUrl}>
					{looking ? t('downloads.looking') : t('downloads.look')}
				</button>
			</div>

			{#if urlTrouble}
				<p class="fault-text note">{t(WHY[urlTrouble.code] ?? 'downloads.otherTrouble')}</p>
				{#if urlTrouble.said}
					<details class="detail">
						<summary>{t('downloads.seeDetail')}</summary>
						<p class="numeric said">{urlTrouble.said}</p>
					</details>
				{/if}
			{/if}

			{#if probe}
				<p class="title ellipsis" title={probe.title}>{probe.title}</p>

				<ul class="items candidates">
					{#each probe.entries as entry, index (entry.url)}
						<li class="item">
							<label class="take">
								<input
									type="checkbox"
									checked={picked[entry.url] ?? true}
									onchange={(event) =>
										(picked = { ...picked, [entry.url]: event.currentTarget.checked })}
								/>
								<span class="ellipsis">{index + 1}. {entry.title}</span>
							</label>
							<span class="numeric muted">{clock(entry.seconds)}</span>
							<span class="muted ellipsis by">{entry.uploader ?? ''}</span>
						</li>
					{/each}
				</ul>

				<div class="row">
					<label class="inline">
						<span class="label">{t('downloads.format')}</span>
						<select class="field" bind:value={format}>
							<option value="original">{t('downloads.formatOriginal')}</option>
							<option value="mp3" disabled={!canConvert}>MP3</option>
							<option value="flac" disabled={!canConvert}>FLAC</option>
						</select>
					</label>
					<button type="button" class="btn primary" disabled={chosen.length === 0} onclick={start}>
						{t('downloads.fetchCount', { n: chosen.length })}
					</button>
				</div>
				{#if format !== 'original'}
					<p class="muted note">{t('downloads.conversionNote')}</p>
				{/if}
				{#if !canConvert}
					<p class="muted note">{t('downloads.withoutFfmpeg')}</p>
				{/if}
			{/if}

			{#if queue && queue.items.length > 0}
				<h2 class="label">{t('downloads.queue')}</h2>
				{#if queue.folder}
					<p class="muted note numeric where">{queue.folder}</p>
				{/if}
				<ul class="items queued">
					{#each queue.items as item (item.id)}
						<li class="item run" class:failed={item.state === 'failed'}>
							<span class="ellipsis">{item.title}</span>
							<span class="muted state">{t(STATE[item.state])}</span>
							{#if item.state === 'running'}
								<span class="bar" aria-hidden="true">
									<span
										class="fill"
										style:width={item.fraction ? `${Math.round(item.fraction * 100)}%` : '0%'}
									></span>
								</span>
								<span class="numeric muted rate">
									{item.speed ? `${size(item.speed)}/s` : ''}
									{item.eta ? `· ${clock(item.eta)}` : ''}
								</span>
							{:else if item.failed}
								<!-- The reason in the listener's own language, and
								     yt-dlp's own sentence kept under it for whoever
								     looks into it. Neither is worth losing. -->
								<span class="fault-text state">
									{t(WHY[item.failed] ?? 'downloads.otherTrouble')}
								</span>
								{#if item.said}
									<details class="detail">
										<summary>{t('downloads.seeDetail')}</summary>
										<p class="numeric said">{item.said}</p>
									</details>
								{/if}
							{/if}
						</li>
					{/each}
				</ul>
				{#if queue.running}
					<div class="row">
						<button type="button" class="btn" onclick={() => downloadStop()}>
							{t('downloads.stopAll')}
						</button>
					</div>
				{/if}
			{/if}
		{/if}
	</section>
</div>

<style>
	.downloads {
		display: flex;
		flex-direction: column;
		gap: 12px;
		min-height: 0;
		height: 100%;
		padding: 14px 16px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	.head h1 {
		margin: 0;
		font-size: 15px;
		font-weight: 500;
		color: var(--onsa-text-primary);
	}

	.what,
	.note {
		margin: 0;
		font-size: 12px;
	}

	.programs,
	.fetch {
		display: grid;
		gap: 8px;
		padding: 12px 14px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
	}

	h2 {
		margin: 0;
		font-size: 10.5px;
	}

	.list {
		display: grid;
		gap: 6px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.program {
		display: grid;
		grid-template-columns: minmax(0, auto) auto minmax(0, 1fr) auto;
		align-items: center;
		gap: 8px 12px;
		min-width: 0;
		padding: 6px 8px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		font-size: 13px;
	}

	.program.missing {
		outline: var(--onsa-hairline) solid var(--onsa-role-caution);
	}

	.name {
		color: var(--onsa-text-primary);
		font-family: var(--onsa-font-numeric);
	}

	.state {
		font-size: 12px;
	}

	.state.ok {
		color: var(--onsa-role-active);
	}

	.where {
		font-size: 12px;
	}

	.source {
		grid-column: 1 / -1;
		font-size: 11px;
	}

	.bar {
		grid-column: 1 / -2;
		display: block;
		height: 5px;
		border-radius: 3px;
		background: var(--onsa-surface-well);
		overflow: hidden;
	}

	.fill {
		display: block;
		height: 100%;
		background: var(--onsa-role-active);
		transition: width var(--onsa-motion-fast) var(--onsa-ease);
	}

	.done {
		font-size: 11px;
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}

	.inline {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.grow {
		flex: 1 1 260px;
		min-width: 0;
	}

	.title {
		margin: 0;
		font-size: 13px;
		color: var(--onsa-text-primary);
	}

	.items {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 2px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.item {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto minmax(0, 0.6fr);
		align-items: center;
		gap: 10px;
		min-width: 0;
		padding: 3px 8px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		font-size: 12px;
	}

	/* A program's own sentence, which is longer than a word. */
	.detail {
		grid-column: 1 / -1;
		font-size: 12px;
	}

	.detail summary {
		color: var(--onsa-role-adjustable);
		cursor: pointer;
	}

	.detail p {
		margin: 4px 0 0;
		color: var(--onsa-text-secondary);
		overflow-wrap: anywhere;
	}

	.anyway {
		grid-column: 1 / -1;
		margin: 2px 0 0;
	}

	.said {
		grid-column: 1 / -1;
		font-size: 11.5px;
		overflow-wrap: anywhere;
	}

	.item.run {
		grid-template-columns: minmax(0, 1fr) auto minmax(60px, 0.8fr) auto;
	}

	.item.failed {
		outline: var(--onsa-hairline) solid var(--onsa-role-clip);
	}

	.take {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.state,
	.rate,
	.by {
		font-size: 11.5px;
	}

	/* A path is long, and the end of it is what tells it apart. */
	.where {
		overflow-wrap: anywhere;
	}

	.fetch :global(svg) {
		width: 13px;
		height: 13px;
		vertical-align: -2px;
	}

	@container content (max-width: 720px) {
		.program {
			grid-template-columns: minmax(0, 1fr) auto;
		}

		.where {
			grid-column: 1 / -1;
		}
	}
</style>
