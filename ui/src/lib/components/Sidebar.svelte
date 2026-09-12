<!-- Navigation (SPEC section 9.2): only what this build can already do. -->
<script lang="ts">
	import { app, navigate, type SettingsSection, type View } from '$lib/app.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { library, scanSummary, setQuery } from '$lib/library.svelte';

	const settingsPages: { section: SettingsSection; label: `settings.${SettingsSection}` }[] = [
		{ section: 'output', label: 'settings.output' },
		{ section: 'dsp', label: 'settings.dsp' },
		{ section: 'library', label: 'settings.library' },
		{ section: 'appearance', label: 'settings.appearance' },
		{ section: 'about', label: 'settings.about' }
	];

	function go(view: View): void {
		setQuery('');
		navigate(view);
	}

	const view = $derived(app.view);
</script>

<nav class="sidebar">
	<div class="mark">Onsa</div>

	<h2 class="label">{t('nav.library')}</h2>
	<button
		type="button"
		class="item"
		aria-current={view.kind === 'tracks' && !library.query}
		onclick={() => go({ kind: 'tracks' })}>{t('nav.tracks')}</button
	>
	<button
		type="button"
		class="item"
		aria-current={(view.kind === 'albums' || view.kind === 'album') && !library.query}
		onclick={() => go({ kind: 'albums' })}>{t('nav.albums')}</button
	>

	<h2 class="label">{t('nav.settings')}</h2>
	{#each settingsPages as page (page.section)}
		<button
			type="button"
			class="item"
			aria-current={view.kind === 'settings' && view.section === page.section && !library.query}
			onclick={() => go({ kind: 'settings', section: page.section })}>{t(page.label)}</button
		>
	{/each}

	{#if library.scan.running}
		<p class="scan numeric">{scanSummary()}</p>
	{/if}
</nav>

<style>
	.sidebar {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 16px 10px;
		overflow-y: auto;
		border-right: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
	}

	.mark {
		padding: 0 8px 12px;
		font-size: 20px;
		color: var(--onsa-text-primary);
	}

	h2 {
		margin: 14px 8px 4px;
		font-size: 10.5px;
		font-weight: 500;
	}

	.item {
		padding: 6px 8px;
		border: 0;
		border-radius: var(--onsa-radius-sm);
		background: none;
		color: var(--onsa-text-secondary);
		font: inherit;
		font-size: 13px;
		text-align: left;
		cursor: pointer;
	}

	.item:hover {
		color: var(--onsa-text-primary);
		background: var(--onsa-surface-raised);
	}

	.item[aria-current='true'] {
		color: var(--onsa-role-active);
		background: var(--onsa-surface-raised);
	}

	.scan {
		margin: auto 8px 0;
		padding-top: 12px;
		font-size: 11px;
		color: var(--onsa-role-caution);
	}
</style>
