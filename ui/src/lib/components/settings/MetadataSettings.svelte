<!--
	What Onsa may ask the internet about a track, and the key it asks with
	(SPEC section 8, section 14).

	Nothing here reaches the internet unless the listener turns it on, and
	the key is written here but never read back: the interface can say that
	there is one and where it came from, never what it is.
-->
<script lang="ts">
	import {
		acoustidTest,
		failureKey,
		metadataGet,
		programChoose,
		programPick,
		programStatus,
		setMetadata,
		type KeyTest,
		type MetadataPrefs,
		type ProgramStatus
	} from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	let prefs = $state<MetadataPrefs | null>(null);
	let programs = $state<ProgramStatus[]>([]);
	let looking = $state(false);
	let typed = $state('');
	let failure = $state<MessageKey | null>(null);
	let note = $state<MessageKey | null>(null);

	$effect(() => {
		metadataGet()
			.then((next) => (prefs = next))
			.catch((error) => (failure = failureKey(error)));
	});

	$effect(() => {
		void lookAgain();
	});

	/**
	 * Which programs are there. It means running each of them to ask its
	 * version, so it is done when the page opens and when asked, not on
	 * every keystroke.
	 */
	async function lookAgain(): Promise<void> {
		looking = true;
		try {
			programs = await programStatus();
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		} finally {
			looking = false;
		}
	}

	/** The one M7 needs; the rest belong to the downloader in M10. */
	const fingerprinter = $derived(programs.find((one) => one.key === 'fpcalc') ?? null);

	const WHERE: Record<string, MessageKey> = {
		managed: 'programs.fromManaged',
		chosen: 'programs.fromChosen',
		system: 'programs.fromSystem'
	};

	async function chooseFpcalc(): Promise<void> {
		try {
			const picked = await programPick(t('programs.chooseTitle'));
			if (!picked) return;
			await programChoose('fpcalc', picked);
			await lookAgain();
		} catch (error) {
			failure = failureKey(error);
		}
	}

	async function forgetFpcalc(): Promise<void> {
		try {
			await programChoose('fpcalc', null);
			await lookAgain();
		} catch (error) {
			failure = failureKey(error);
		}
	}

	const source = $derived<MessageKey>(
		prefs?.acoustid.source === 'settings'
			? 'metadata.keyFromSettings'
			: prefs?.acoustid.source === 'environment'
				? 'metadata.keyFromEnvironment'
				: prefs?.acoustid.source === 'build'
					? 'metadata.keyFromBuild'
					: 'metadata.keyMissing'
	);

	/** What the last try said, while it is still worth showing. */
	let tried = $state<KeyTest | null>(null);
	let trying = $state(false);

	const TRIED: Record<KeyTest, MessageKey> = {
		works: 'metadata.keyWorks',
		refused: 'metadata.keyRefused',
		missing: 'metadata.keyMissing',
		offline: 'metadata.keyOffline',
		unreachable: 'metadata.keyUnreachable'
	};

	async function tryKey(): Promise<void> {
		trying = true;
		tried = null;
		try {
			tried = await acoustidTest();
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		} finally {
			trying = false;
		}
	}

	async function save(online: boolean, key?: string): Promise<void> {
		try {
			prefs = await setMetadata(online, key);
			failure = null;
			if (key !== undefined) note = key.trim() ? 'metadata.keySaved' : 'metadata.keyCleared';
			tried = null;
		} catch (error) {
			failure = failureKey(error);
			note = null;
		}
	}
</script>

<div class="page">
	<section>
		<h2 class="label">{t('metadata.internet')}</h2>
		<label class="row">
			<input
				type="checkbox"
				checked={prefs?.online ?? false}
				onchange={(event) => save(event.currentTarget.checked)}
			/>
			<span>{t('metadata.online')}</span>
		</label>
		<p class="note muted">{t('metadata.onlineHint')}</p>
	</section>

	<section>
		<h2 class="label">{t('metadata.acoustid')}</h2>
		<p class="note muted">{t('metadata.acoustidHint')}</p>
		<label class="row stack">
			<span>{t('metadata.key')}</span>
			<!-- A password field: the key is not for reading over a shoulder,
			     and it is never filled in from the backend. -->
			<input
				class="field key"
				type="password"
				autocomplete="off"
				spellcheck="false"
				placeholder={t('metadata.keyPlaceholder')}
				bind:value={typed}
			/>
		</label>
		<div class="actions">
			<button
				type="button"
				class="btn"
				disabled={typed.trim().length === 0}
				onclick={() => {
					save(prefs?.online ?? false, typed);
					typed = '';
				}}
			>
				{t('metadata.keySave')}
			</button>
			<button
				type="button"
				class="btn"
				disabled={prefs?.acoustid.source !== 'settings'}
				onclick={() => {
					save(prefs?.online ?? false, '');
					typed = '';
				}}
			>
				{t('metadata.keyClear')}
			</button>
			<button
				type="button"
				class="btn"
				disabled={trying || !prefs?.acoustid.present}
				onclick={tryKey}
			>
				{trying ? t('metadata.keyTrying') : t('metadata.keyTry')}
			</button>
		</div>
		<p class="note" class:muted={prefs?.acoustid.present}>{t(source)}</p>
		{#if tried}
			<p class="note" class:fault-text={tried !== 'works'} class:muted={tried === 'works'}>
				{t(TRIED[tried])}
			</p>
		{/if}
		{#if note}<p class="note muted">{t(note)}</p>{/if}
	</section>

	<section>
		<h2 class="label">{t('programs.title')}</h2>
		<p class="note muted">{t('programs.hint')}</p>

		<p class="note">
			<span class="name">fpcalc</span>
			{#if fingerprinter?.present}
				<span class="ok">{t('programs.installed')}</span>
				{#if fingerprinter.from}
					<span class="muted">· {t(WHERE[fingerprinter.from] ?? 'programs.fromSystem')}</span>
				{/if}
				{#if fingerprinter.version}
					<span class="muted numeric">· {fingerprinter.version}</span>
				{/if}
			{:else}
				<span class="fault-text">{t('programs.missing')}</span>
			{/if}
		</p>
		{#if fingerprinter?.path}
			<p class="note muted numeric where">{fingerprinter.path}</p>
		{:else}
			<p class="note muted">{t('programs.fpcalcWhere')}</p>
		{/if}

		<div class="actions">
			<button type="button" class="btn" onclick={chooseFpcalc}>{t('programs.choose')}</button>
			<button
				type="button"
				class="btn"
				disabled={fingerprinter?.from !== 'chosen'}
				onclick={forgetFpcalc}>{t('programs.forget')}</button
			>
			<button type="button" class="btn" disabled={looking} onclick={lookAgain}
				>{t('programs.refresh')}</button
			>
		</div>
		{#if failure}<p class="note fault-text">{t(failure)}</p>{/if}
	</section>
</div>

<style>
	@import './settings.css';

	.row {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.row.stack {
		display: grid;
		gap: 6px;
		max-width: 420px;
	}

	.key {
		width: 100%;
		font-family: var(--onsa-font-numeric, inherit);
		letter-spacing: 0.08em;
	}

	.actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 10px;
	}

	.name {
		color: var(--onsa-text-primary);
	}

	.ok {
		color: var(--onsa-role-active);
	}

	/* A path is long and the end of it is what tells it apart. */
	.where {
		overflow-wrap: anywhere;
	}
</style>
