<!--
	What Onsa may ask the internet about a track, and the key it asks with
	(SPEC section 8, section 14).

	Nothing here reaches the internet unless the listener turns it on, and
	the key is written here but never read back: the interface can say that
	there is one and where it came from, never what it is.
-->
<script lang="ts">
	import { failureKey, metadataGet, setMetadata, type MetadataPrefs } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	let prefs = $state<MetadataPrefs | null>(null);
	let typed = $state('');
	let failure = $state<MessageKey | null>(null);
	let note = $state<MessageKey | null>(null);

	$effect(() => {
		metadataGet()
			.then((next) => (prefs = next))
			.catch((error) => (failure = failureKey(error)));
	});

	const source = $derived<MessageKey>(
		prefs?.acoustid.source === 'settings'
			? 'metadata.keyFromSettings'
			: prefs?.acoustid.source === 'environment'
				? 'metadata.keyFromEnvironment'
				: prefs?.acoustid.source === 'build'
					? 'metadata.keyFromBuild'
					: 'metadata.keyMissing'
	);

	async function save(online: boolean, key?: string): Promise<void> {
		try {
			prefs = await setMetadata(online, key);
			failure = null;
			if (key !== undefined) note = key.trim() ? 'metadata.keySaved' : 'metadata.keyCleared';
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
		</div>
		<p class="note" class:muted={prefs?.acoustid.present}>{t(source)}</p>
		{#if note}<p class="note muted">{t(note)}</p>{/if}
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
</style>
