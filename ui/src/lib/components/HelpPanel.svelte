<!--
	The help for the page being shown, beside the page rather than over it.

	It explains one page and nothing else: what the page is for, every
	control it draws, what happens to the listener's own files, and what it
	cannot do in this version. Nothing here stops the page being used — no
	scrim, no focus trap, and the page keeps its own scroll.
-->
<script lang="ts">
	import { app, navigate } from '$lib/app.svelte';
	import { closeHelp, layout } from '$lib/layout.svelte';
	import { library } from '$lib/library.svelte';
	import { t } from '$lib/i18n/index.svelte';
	import { topic as topicById, topicFor } from '$lib/help/topics';
	import Icon from './Icon.svelte';

	const topic = $derived(
		(layout.helpTopic ? topicById(layout.helpTopic) : undefined) ??
			topicFor(app.view, library.query.trim().length > 0)
	);
</script>

<aside class="help" aria-label={t('help.open')}>
	<header>
		<h2>{topic ? t(topic.title) : t('nav.help')}</h2>
		<button type="button" class="icon-btn" aria-label={t('help.close')} onclick={closeHelp}>
			<Icon name="close" />
		</button>
	</header>

	<div class="scroll">
		{#if topic}
			<p class="what">{t(topic.what)}</p>

			{#if topic.controls.length > 0}
				<h3 class="label">{t('help.controls')}</h3>
				<dl>
					{#each topic.controls as one (one.says)}
						<dt>{t(one.name)}</dt>
						<dd>{t(one.says)}</dd>
					{/each}
				</dl>
			{/if}

			{#if topic.files}
				<h3 class="label">{t('help.filesHead')}</h3>
				<p>{t(topic.files)}</p>
			{/if}

			{#if topic.limits.length > 0}
				<h3 class="label">{t('help.limitsHead')}</h3>
				<ul>
					{#each topic.limits as limit (limit)}
						<li>{t(limit)}</li>
					{/each}
				</ul>
			{/if}

			<p class="muted small">{t('help.elsewhere')}</p>
		{/if}

		{#if app.view.kind !== 'help'}
			<button type="button" class="btn" onclick={() => navigate({ kind: 'help' })}>
				{t('help.toHelpPage')}
			</button>
		{/if}
	</div>
</aside>

<style>
	.help {
		display: grid;
		grid-template-rows: auto minmax(0, 1fr);
		min-height: 0;
		border-left: var(--onsa-hairline) solid var(--onsa-surface-line);
		background: var(--onsa-surface-body);
	}

	header {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 12px 10px 8px 14px;
	}

	h2 {
		flex: 1;
		margin: 0;
		font-size: 13px;
		font-weight: 500;
		color: var(--onsa-role-active);
	}

	.scroll {
		display: grid;
		align-content: start;
		gap: 10px;
		padding: 0 14px 16px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	h3 {
		margin: 6px 0 0;
		font-size: 11px;
		font-weight: 500;
	}

	p {
		margin: 0;
		font-size: 12.5px;
		line-height: 1.55;
	}

	.what {
		color: var(--onsa-text-primary);
	}

	.small {
		font-size: 11.5px;
	}

	dl {
		display: grid;
		gap: 10px;
		margin: 0;
	}

	dt {
		font-size: 12.5px;
		color: var(--onsa-role-adjustable);
	}

	dd {
		margin: 2px 0 0;
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--onsa-text-secondary);
	}

	ul {
		margin: 0;
		padding-left: 18px;
		font-size: 12.5px;
		line-height: 1.55;
		color: var(--onsa-text-secondary);
	}

	li + li {
		margin-top: 6px;
	}
</style>
