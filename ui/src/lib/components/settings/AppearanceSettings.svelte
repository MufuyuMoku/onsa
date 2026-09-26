<!-- Appearance: theme and language. -->
<script lang="ts">
	import {
		failureKey,
		themeCopyBuiltin,
		themeOpenFolder,
		themeTroubles,
		type ThemeTrouble,
		type ToneStrength
	} from '$lib/backend';
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
	/** Theme files in the folder that could not be used, and why. */
	let troubles = $state<ThemeTrouble[]>([]);
	/** Where the copy landed, once one has been made. */
	let copied = $state<string | null>(null);
	let copying = $state(false);

	$effect(() => {
		themeTroubles()
			.then((found) => (troubles = found))
			.catch(() => (troubles = []));
	});

	/**
	 * Copies the theme in use into the listener's own folder.
	 *
	 * Nobody can guess the shape of a file they have never seen, and the
	 * hint about where to put one does not help with what to put in it.
	 */
	async function copyTheme(): Promise<void> {
		const from = activeTheme()?.id;
		if (!from) return;
		copying = true;
		try {
			copied = await themeCopyBuiltin(from, t('theme.copyName'));
			failure = null;
			troubles = await themeTroubles();
		} catch (error) {
			failure = failureKey(error);
			copied = null;
		} finally {
			copying = false;
		}
	}

	/** Whether the theme in use moves anything at all with the tone. */
	const leans = $derived(
		(activeTheme()?.toneColor.enabled ?? false) &&
			(activeTheme()?.toneColor.targets.length ?? 0) > 0
	);
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
					disabled={!settings.value}
					onclick={() => updateDisplay({ toneColor: strength })}>{t(label)}</button
				>
			{/each}
		</div>
		<p class="note muted">{t('toneColor.hint')}</p>
		<!-- The theme chooses what follows the tone; switching the feature
		     off is the listener's to do (SPEC section 9.5). A theme that
		     moves nothing says so here rather than leaving four dead
		     buttons and no reason. -->
		{#if !leans}
			<p class="note muted">{t('toneColor.themeQuiet')}</p>
		{/if}
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
			<button type="button" class="btn" disabled={copying} onclick={copyTheme}>
				{t('theme.copy')}
			</button>
			{#if failure}<span class="note fault-text">{t(failure)}</span>{/if}
		</div>
		<p class="note muted">{t('theme.copyHint')}</p>
		{#if copied}
			<p class="note">{t('theme.copied', { file: copied })}</p>
		{/if}

		{#if troubles.length > 0}
			<h3 class="label">{t('theme.troubles')}</h3>
			<p class="note muted">{t('theme.troublesHint')}</p>
			<ul class="troubles">
				{#each troubles as trouble (trouble.file)}
					<li>
						<span class="numeric">{trouble.file}</span>
						<span class="muted said">{trouble.said}</span>
					</li>
				{/each}
			</ul>
		{/if}
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

	h3 {
		margin: 10px 0 0;
		font-size: 11px;
		font-weight: 500;
	}

	.troubles {
		display: grid;
		gap: 6px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.troubles li {
		display: grid;
		gap: 2px;
		font-size: 12px;
	}

	/* The parser's own words, which can be long. */
	.said {
		font-size: 11.5px;
		overflow-wrap: anywhere;
	}
</style>
