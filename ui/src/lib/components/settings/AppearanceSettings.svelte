<!-- Appearance: theme and language. -->
<script lang="ts">
	import { chooseLocale } from '$lib/app.svelte';
	import { currentLocale, LOCALES, t, type Locale } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { activeTheme, availableThemes, selectTheme } from '$lib/theme/index.svelte';

	const languageKeys: Record<Locale, MessageKey> = { id: 'language.id', en: 'language.en' };
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
