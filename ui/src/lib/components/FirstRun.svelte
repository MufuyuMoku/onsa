<!--
	The first screen (SPEC section 9.2): choose the music folder, choose a
	theme with a live preview, then scan with visible progress — and, when
	that is done, one closing step naming the three pages nobody would find
	on their own. It can be asked for again from the help page.
-->
<script lang="ts">
	import { addFolder, failureKey, pickFolder } from '$lib/backend';
	import { app, chooseLocale, finishFirstRun, type View } from '$lib/app.svelte';
	import { currentLocale, LOCALES, t, type Locale } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { library, scanSummary } from '$lib/library.svelte';
	import { activeTheme, availableThemes, selectTheme } from '$lib/theme/index.svelte';

	let folder = $state<string | null>(null);
	let phase = $state<'setup' | 'scanning' | 'done'>('setup');
	let sawRunning = $state(false);
	let failure = $state<MessageKey | null>(null);

	const languageKeys: Record<Locale, MessageKey> = { id: 'language.id', en: 'language.en' };

	/** What the closing step names, one sentence each. */
	const elsewhere: { view: View; label: MessageKey; says: MessageKey }[] = [
		{ view: { kind: 'tidy' }, label: 'nav.tidy', says: 'firstRun.moreTidy' },
		{ view: { kind: 'downloads' }, label: 'nav.downloads', says: 'firstRun.moreDownloads' },
		{
			view: { kind: 'settings', section: 'lyrics' },
			label: 'settings.lyrics',
			says: 'firstRun.moreLyrics'
		}
	];

	// Asked for again from the help page, rather than shown because there is
	// no library yet: the closing step is the point of the visit, so it is
	// there from the start.
	const revisit = $derived(app.revisit);

	async function choose(): Promise<void> {
		try {
			const picked = await pickFolder(t('firstRun.dialogTitle'));
			if (picked) folder = picked;
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
	}

	async function begin(): Promise<void> {
		if (!folder) return;
		try {
			phase = 'scanning';
			await addFolder(folder);
		} catch (error) {
			phase = 'setup';
			failure = failureKey(error);
		}
	}

	$effect(() => {
		if (phase !== 'scanning') return;
		if (library.scan.running) sawRunning = true;
		else if (sawRunning) phase = 'done';
	});
</script>

<main class="first">
	<section class="panel">
		<header>
			<h1>{t('firstRun.welcome')}</h1>
			<p class="muted">{revisit ? t('firstRun.again') : t('firstRun.intro')}</p>
		</header>

		<div class="step">
			<h2 class="label">{t('firstRun.folderStep')}</h2>
			<p class="muted">{t('firstRun.folderHint')}</p>
			<div class="row">
				<button type="button" class="btn" disabled={phase !== 'setup'} onclick={choose}>
					{folder ? t('firstRun.changeFolder') : t('firstRun.pickFolder')}
				</button>
				{#if folder}<span class="path numeric ellipsis">{folder}</span>{/if}
			</div>
		</div>

		<div class="step">
			<h2 class="label">{t('firstRun.themeStep')}</h2>
			<p class="muted">{t('firstRun.themeHint')}</p>
			<div class="themes">
				{#each availableThemes() as theme (theme.id)}
					<button
						type="button"
						class="theme"
						aria-pressed={activeTheme()?.id === theme.id}
						onclick={() => selectTheme(theme.id, document.documentElement)}
					>
						<span class="swatches" aria-hidden="true">
							<i style:background={theme.color.surface.body}></i>
							<i style:background={theme.color.role.active}></i>
							<i style:background={theme.color.role.adjustable}></i>
							<i style:background={theme.color.role.position}></i>
							<i style:background={theme.color.role.caution}></i>
						</span>
						<span>{theme.name[currentLocale()]}</span>
					</button>
				{/each}
			</div>
			<div class="row">
				<span class="label">{t('language.label')}</span>
				{#each LOCALES as locale (locale)}
					<button
						type="button"
						class="chip"
						aria-pressed={currentLocale() === locale}
						onclick={() => chooseLocale(locale)}>{t(languageKeys[locale])}</button
					>
				{/each}
			</div>
		</div>

		<div class="step">
			<h2 class="label">{t('firstRun.scanStep')}</h2>
			{#if phase === 'setup'}
				{#if revisit}<p class="muted">{t('firstRun.againScan')}</p>{/if}
				<button type="button" class="btn primary" disabled={!folder} onclick={begin}>
					{t('firstRun.start')}
				</button>
			{:else}
				<div class="progress well">
					<p class="numeric">{scanSummary()}</p>
					<div class="bar" class:running={phase === 'scanning'}><i></i></div>
				</div>
			{/if}
		</div>

		{#if phase === 'done' || revisit}
			<div class="step">
				<h2 class="label">{t('firstRun.moreStep')}</h2>
				<p class="muted">{t('firstRun.moreHint')}</p>
				<ul class="more">
					{#each elsewhere as item (item.label)}
						<li>
							<button type="button" class="go" onclick={() => finishFirstRun(item.view)}>
								{t(item.label)}
							</button>
							<span class="muted">{t(item.says)}</span>
						</li>
					{/each}
				</ul>
			</div>

			<button type="button" class="btn primary" onclick={() => finishFirstRun()}>
				{phase === 'done' ? t('firstRun.open') : t('firstRun.close')}
			</button>
		{/if}

		{#if failure}<p class="fault-text">{t(failure)}</p>{/if}
	</section>
</main>

<style>
	.first {
		display: grid;
		place-items: center;
		height: 100%;
		padding: 24px;
		overflow-y: auto;
	}

	.panel {
		display: grid;
		gap: 22px;
		width: min(720px, 100%);
		padding: 26px 28px;
		border-radius: var(--onsa-radius-lg);
		background: var(--onsa-surface-body);
		box-shadow:
			inset 0 var(--onsa-hairline) 0 var(--onsa-surface-body-highlight),
			0 22px 44px rgb(0 0 0 / 0.45);
	}

	h1 {
		margin: 0;
		font-size: 26px;
		font-weight: 500;
	}

	header p,
	.step p {
		margin: 4px 0 0;
	}

	.step {
		display: grid;
		gap: 8px;
	}

	h2 {
		margin: 0;
		font-size: 11px;
		font-weight: 500;
	}

	.row {
		display: flex;
		align-items: center;
		flex-wrap: wrap;
		gap: 8px;
		min-width: 0;
	}

	.path {
		max-width: 100%;
		font-size: 12.5px;
		color: var(--onsa-role-adjustable);
	}

	.themes {
		display: grid;
		grid-template-columns: repeat(3, minmax(0, 1fr));
		gap: 10px;
	}

	.theme {
		display: grid;
		gap: 8px;
		padding: 10px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-raised);
		color: var(--onsa-text-primary);
		font: inherit;
		text-align: left;
		cursor: pointer;
	}

	.theme[aria-pressed='true'] {
		border-color: var(--onsa-role-active);
	}

	.swatches {
		display: flex;
		height: 26px;
		border-radius: var(--onsa-radius-sm);
		overflow: hidden;
	}

	.swatches i {
		flex: 1;
	}

	.more {
		display: grid;
		gap: 8px;
		margin: 2px 0 0;
		padding: 0;
		list-style: none;
	}

	.more li {
		display: grid;
		grid-template-columns: minmax(0, 8em) minmax(0, 1fr);
		gap: 10px;
		align-items: baseline;
	}

	.go {
		padding: 0;
		border: 0;
		background: none;
		color: var(--onsa-role-adjustable);
		font: inherit;
		text-align: left;
		text-decoration: underline;
		text-underline-offset: 3px;
		cursor: pointer;
	}

	.go:hover {
		color: var(--onsa-role-active);
	}

	.progress {
		display: grid;
		gap: 8px;
		padding: 12px 14px;
	}

	.progress p {
		margin: 0;
		color: var(--onsa-role-active);
	}

	.bar {
		position: relative;
		height: 3px;
		border-radius: 2px;
		background: var(--onsa-lit-ghost);
		overflow: hidden;
	}

	.bar i {
		position: absolute;
		inset: 0;
		background: var(--onsa-role-active);
	}

	.bar.running i {
		width: 30%;
		animation: sweep 1.2s linear infinite;
	}

	@keyframes sweep {
		from {
			transform: translateX(-100%);
		}
		to {
			transform: translateX(340%);
		}
	}
</style>
