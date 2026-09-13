<!-- Appearance: theme and language. -->
<script lang="ts">
	import { failureKey, themeOpenFolder, type ToneStrength } from '$lib/backend';
	import { chooseLocale } from '$lib/app.svelte';
	import { currentLocale, LOCALES, t, type Locale } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { settings, updateDisplay } from '$lib/settings.svelte';
	import { activeTheme, availableThemes, selectTheme } from '$lib/theme/index.svelte';

	const languageKeys: Record<Locale, MessageKey> = { id: 'language.id', en: 'language.en' };

	/** How far the colour may follow the sound (SPEC section 9.5). */
	const STRENGTHS: [ToneStrength, MessageKey][] = [
		['off', 'toneColor.off'],
		['subtle', 'toneColor.subtle'],
		['medium', 'toneColor.medium'],
		['strong', 'toneColor.strong']
	];

	let failure = $state<MessageKey | null>(null);
</script>

<div class="page">
	<section>
		<h2 class="label">{t('theme.label')}</h2>
		<div class="chips">
			{#each availableThemes() as theme (theme.id)}
				<button
					type="button"
					class="chip"
					aria-pressed={activeTheme()?.id === theme.id}
					onclick={() => selectTheme(theme.id, document.documentElement)}
					>{theme.name[currentLocale()]}</button
				>
			{/each}
		</div>
	</section>
	<section>
		<h2 class="label">{t('toneColor.label')}</h2>
		<div class="chips">
			{#each STRENGTHS as [strength, label] (strength)}
				<button
					type="button"
					class="chip"
					aria-pressed={(settings.value?.display.toneColor ?? 'medium') === strength}
					disabled={!settings.value || !(activeTheme()?.toneColor.enabled ?? false)}
					onclick={() => updateDisplay({ toneColor: strength })}>{t(label)}</button
				>
			{/each}
		</div>
		<p class="note muted">{t('toneColor.hint')}</p>
	</section>
	<section>
		<h2 class="label">{t('theme.userFolder')}</h2>
		<p class="note muted">{t('theme.userHint')}</p>
		<div class="actions">
			<button
				type="button"
				class="btn"
				onclick={() => themeOpenFolder().catch((error) => (failure = failureKey(error)))}
			>
				{t('theme.openFolder')}
			</button>
			{#if failure}<span class="note fault-text">{t(failure)}</span>{/if}
		</div>
	</section>
	<section>
		<h2 class="label">{t('language.label')}</h2>
		<div class="chips">
			{#each LOCALES as locale (locale)}
				<button
					type="button"
					class="chip"
					aria-pressed={currentLocale() === locale}
					onclick={() => chooseLocale(locale)}>{t(languageKeys[locale])}</button
				>
			{/each}
		</div>
	</section>
</div>

<style>
	@import './settings.css';
</style>
