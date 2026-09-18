<!--
	What Onsa may do about a song's words (SPEC section 10, section 14).

	The two sources that cost nothing — a `.lrc` beside the song, and the
	words in its own tags — are read whatever is set here. Only the service
	on the internet is a choice, and like every other way out it starts off
	and can be turned off again at any moment, lookups in the air included.
-->
<script lang="ts">
	import { failureKey, lyricsSettings, setLyricsSettings, type LyricsPrefs } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	let prefs = $state<LyricsPrefs | null>(null);
	let failure = $state<MessageKey | null>(null);

	$effect(() => {
		lyricsSettings()
			.then((next) => (prefs = next))
			.catch((error) => (failure = failureKey(error)));
	});

	async function save(online: boolean, writeBeside: boolean): Promise<void> {
		try {
			prefs = await setLyricsSettings(online, writeBeside);
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		}
	}
</script>

<div class="page">
	<section>
		<h2 class="label">{t('lyrics.settingsTitle')}</h2>
		<p class="note muted">{t('lyrics.sourcesHint')}</p>
		<label class="row">
			<input
				type="checkbox"
				checked={prefs?.online ?? false}
				onchange={(event) => save(event.currentTarget.checked, prefs?.writeBeside ?? false)}
			/>
			<span>{t('lyrics.onlineLabel')}</span>
		</label>
		<p class="note muted">{t('lyrics.onlineHint')}</p>

		<label class="row">
			<input
				type="checkbox"
				disabled={!prefs?.online}
				checked={prefs?.writeBeside ?? false}
				onchange={(event) => save(prefs?.online ?? false, event.currentTarget.checked)}
			/>
			<span>{t('lyrics.writeBesideLabel')}</span>
		</label>
		<p class="note muted">{t('lyrics.writeBesideHint')}</p>
	</section>

	{#if failure}
		<p class="note fault-text">{t(failure)}</p>
	{/if}
</div>

<style>
	@import './settings.css';
</style>
