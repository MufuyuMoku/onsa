<script lang="ts">
	import { application, applicationFailure } from '$lib/app.svelte';
	import { currentLocale, LOCALES, setLocale, t, type Locale } from '$lib/i18n/index.svelte';
	import { activeTheme, availableThemes, selectTheme, themeFailure } from '$lib/theme/index.svelte';

	const fault = $derived(applicationFailure() ?? themeFailure());

	const languageKeys = { id: 'language.id', en: 'language.en' } as const;

	function languageLabel(locale: Locale): string {
		return t(languageKeys[locale]);
	}
</script>

<main class="app">
	<section class="panel" aria-label="Onsa">
		<header class="panel__head">
			<span class="mark">Onsa</span>
			<span class="numeric version">{application()?.version ?? ''}</span>
		</header>

		<div class="panel__well">
			{#if fault}
				<p class="fault">{t(fault)}</p>
			{:else if activeTheme()}
				<p class="label milestone">{t('shell.milestone')}</p>
				<p class="note">{t('shell.emptyPanel')}</p>
			{:else}
				<p class="note">{t('shell.loading')}</p>
			{/if}
		</div>

		<footer class="panel__foot">
			<fieldset class="group">
				<legend class="label">{t('theme.label')}</legend>
				{#each availableThemes() as theme (theme.id)}
					<button
						type="button"
						class="chip"
						aria-pressed={activeTheme()?.id === theme.id}
						onclick={() => selectTheme(theme.id, document.documentElement)}
					>
						{theme.name[currentLocale()]}
					</button>
				{/each}
			</fieldset>

			<fieldset class="group">
				<legend class="label">{t('language.label')}</legend>
				{#each LOCALES as locale (locale)}
					<button
						type="button"
						class="chip"
						aria-pressed={currentLocale() === locale}
						onclick={() => setLocale(locale)}
					>
						{languageLabel(locale)}
					</button>
				{/each}
			</fieldset>
		</footer>
	</section>
</main>

<style>
	.app {
		display: grid;
		place-items: center;
		height: 100%;
		padding: 24px;
	}

	.panel {
		display: grid;
		grid-template-rows: auto 1fr auto;
		gap: 16px;
		width: min(760px, 100%);
		height: min(440px, 100%);
		padding: 18px;
		border-radius: var(--onsa-radius-lg);
		background: var(--onsa-surface-body);
		box-shadow:
			inset 0 var(--onsa-hairline) 0 var(--onsa-surface-body-highlight),
			0 22px 44px rgb(0 0 0 / 0.45);
	}

	/* Layered bezel, brushed metal or dark glass, as the theme asks. */
	:global(:root[data-onsa-panel-texture='bezel']) .panel {
		box-shadow:
			inset 0 0 0 var(--onsa-hairline) var(--onsa-surface-line),
			inset 0 0 0 5px var(--onsa-surface-body),
			inset 0 0 0 6px var(--onsa-surface-body-highlight),
			0 22px 44px rgb(0 0 0 / 0.45);
	}

	:global(:root[data-onsa-panel-texture='brushed']) .panel {
		background:
			repeating-linear-gradient(90deg, rgb(255 255 255 / 0.022) 0 1px, transparent 1px 3px),
			linear-gradient(180deg, var(--onsa-surface-body-highlight), var(--onsa-surface-body) 55%);
	}

	:global(:root[data-onsa-panel-texture='glass']) .panel {
		background: linear-gradient(
			180deg,
			var(--onsa-surface-body-highlight),
			var(--onsa-surface-body) 40%
		);
	}

	.panel__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}

	.mark {
		font-size: 22px;
		letter-spacing: 0.02em;
		color: var(--onsa-text-primary);
	}

	.version {
		font-size: 12px;
		color: var(--onsa-text-secondary);
	}

	.panel__well {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 8px;
		padding: 16px 18px;
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-well);
		box-shadow: inset 0 2px 10px rgb(0 0 0 / 0.6);
	}

	.milestone {
		margin: 0;
		font-size: 12px;
	}

	.note {
		margin: 0;
		max-width: 52ch;
		color: var(--onsa-text-secondary);
	}

	.fault {
		margin: 0;
		color: var(--onsa-role-clip);
	}

	.panel__foot {
		display: flex;
		flex-wrap: wrap;
		gap: 8px 24px;
	}

	.group {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 0;
		padding: 0;
		border: 0;
	}

	.group legend {
		float: left;
		margin-right: 8px;
		padding: 0;
		font-size: 11px;
		line-height: 26px;
	}

	.chip {
		padding: 4px 10px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		color: var(--onsa-text-secondary);
		font: inherit;
		font-size: 12px;
		cursor: pointer;
		transition:
			color var(--onsa-motion-fast) var(--onsa-ease),
			border-color var(--onsa-motion-fast) var(--onsa-ease);
	}

	.chip:hover {
		color: var(--onsa-text-primary);
	}

	.chip[aria-pressed='true'] {
		border-color: var(--onsa-role-active);
		color: var(--onsa-role-active);
	}

	:global(:root[data-onsa-glow='true']) .chip[aria-pressed='true'] {
		box-shadow: 0 0 8px var(--onsa-lit-glow);
	}
</style>
