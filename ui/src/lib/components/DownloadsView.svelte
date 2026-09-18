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
		EVENTS,
		failureKey,
		on,
		type Binaries,
		type BinaryStatus,
		type FetchProgress
	} from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import Icon from './Icon.svelte';

	let programs = $state<BinaryStatus[]>([]);
	let progress = $state<FetchProgress | null>(null);
	let failure = $state<MessageKey | null>(null);
	let useSystem = $state(true);
	let busy = $state(false);

	$effect(() => {
		void look();
		const stop = on<FetchProgress>(EVENTS.binary, (next) => (progress = next));
		return () => {
			void stop.then((off) => off());
		};
	});

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
		system: 'programs.fromSystem'
	};

	const WHY: Record<string, MessageKey> = {
		unreachable: 'downloads.unreachable',
		tooLarge: 'downloads.tooLarge',
		stopped: 'downloads.stopped',
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
					{:else}
						<span class="state" class:fault-text={program.required}>
							{t('programs.missing')}
						</span>
						<span class="muted where">
							{#if program.installable}
								{t('downloads.fromRelease')}
								<span class="numeric">· {size(program.aboutBytes)}</span>
							{:else}
								{t('downloads.notYet')}
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
		{#if ready}
			<p class="muted note">{t('downloads.soon')}</p>
		{:else}
			<p class="muted note">
				<Icon name="info" />
				{t('downloads.needYtDlp')}
			</p>
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
		align-items: center;
		gap: 8px;
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
