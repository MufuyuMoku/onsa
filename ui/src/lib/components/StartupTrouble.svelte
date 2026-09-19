<!--
	The screen Onsa opens with when it cannot open its library.

	It exists because the alternative was worse than any message: until
	v1.0.1 the application ended before the window appeared, so somebody
	whose database was damaged double-clicked Onsa and saw nothing happen at
	all. This says what happened, where the file is, and what they can do —
	and Onsa touches nothing of theirs while it waits.
-->
<script lang="ts">
	import { failureKey, startupOpenFolder, type StartupTrouble } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import Icon from './Icon.svelte';

	const { trouble }: { trouble: StartupTrouble } = $props();

	const what = $derived<MessageKey>(
		trouble.kind === 'fromNewerOnsa' ? 'startup.newer' : 'startup.unreadable'
	);
	const next = $derived<MessageKey>(
		trouble.kind === 'fromNewerOnsa' ? 'startup.newerWhat' : 'startup.unreadableWhat'
	);

	let failure = $state<MessageKey | null>(null);
</script>

<main class="trouble">
	<section class="panel">
		<h1>{t('startup.title')}</h1>
		<p>{t(what)}</p>
		<p>{t(next)}</p>

		<div class="where">
			<span class="label">{t('startup.where')}</span>
			<p class="numeric path">{trouble.database}</p>
			<button
				type="button"
				class="btn"
				onclick={() => startupOpenFolder().catch((error) => (failure = failureKey(error)))}
			>
				<Icon name="folder" />{t('startup.openFolder')}
			</button>
			{#if failure}<p class="note fault-text">{t(failure)}</p>{/if}
		</div>

		<p class="note muted">{t('startup.nothingTouched')}</p>

		<details>
			<summary class="label">{t('startup.said')}</summary>
			<p class="numeric said">{trouble.said}</p>
		</details>
	</section>
</main>

<style>
	.trouble {
		display: grid;
		place-items: center;
		height: 100%;
		padding: 24px;
		overflow-y: auto;
		background: var(--onsa-surface-app);
	}

	.panel {
		display: grid;
		gap: 12px;
		width: min(640px, 100%);
		padding: 24px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-md);
		background: var(--onsa-surface-body);
	}

	h1 {
		margin: 0;
		font-size: 20px;
		font-weight: 500;
		color: var(--onsa-role-caution);
	}

	p {
		margin: 0;
		max-width: 62ch;
		font-size: 13.5px;
		line-height: 1.55;
	}

	.where {
		display: grid;
		gap: 6px;
		justify-items: start;
		padding: 12px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
	}

	.path {
		font-size: 12.5px;
		overflow-wrap: anywhere;
	}

	.note {
		font-size: 12px;
	}

	summary {
		cursor: pointer;
		font-size: 11px;
	}

	.said {
		margin-top: 6px;
		font-size: 11.5px;
		color: var(--onsa-text-secondary);
		overflow-wrap: anywhere;
	}
</style>
