<!--
	What a run would do, read before there is anything to press (SPEC §8).

	The apply button belongs to this component and sits underneath the
	counts, never above them: a summary you have to scroll back to is a
	summary nobody reads.
-->
<script lang="ts">
	import type { Skip, TidySummary } from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	interface Props {
		summary: TidySummary | null;
		/** The word on the button that goes ahead. */
		confirm: string;
		busy?: boolean;
		onapply: () => void;
	}

	const { summary, confirm, busy = false, onapply }: Props = $props();

	const REASON: Record<Skip['reason'], MessageKey> = {
		outsideFolder: 'tidy.skipOutside',
		overLimit: 'tidy.skipOverLimit',
		noChange: 'tidy.skipNoChange',
		nameTaken: 'tidy.skipNameTaken'
	};
</script>

{#if summary}
	<section class="summary">
		<h3 class="label">{t('tidy.summary')}</h3>
		<ul class="counts">
			<li><span class="numeric">{summary.tracks}</span> {t('tidy.countTracks')}</li>
			{#if summary.fields > 0}
				<li><span class="numeric">{summary.fields}</span> {t('tidy.countFields')}</li>
			{/if}
			{#if summary.filesWritten > 0}
				<li><span class="numeric">{summary.filesWritten}</span> {t('tidy.countWritten')}</li>
			{/if}
			{#if summary.filesMoved > 0}
				<li><span class="numeric">{summary.filesMoved}</span> {t('tidy.countMoved')}</li>
			{/if}
			{#if summary.covers > 0}
				<li><span class="numeric">{summary.covers}</span> {t('tidy.countCovers')}</li>
			{/if}
		</ul>

		{#if summary.skipped.length > 0}
			<h3 class="label">{t('tidy.skipped')}</h3>
			<ul class="counts skips">
				{#each summary.skipped as skip (skip.reason)}
					<li><span class="numeric">{skip.count}</span> {t(REASON[skip.reason])}</li>
				{/each}
			</ul>
		{/if}

		{#if !summary.any}
			<p class="muted nothing">{t('tidy.nothingToDo')}</p>
		{/if}

		<!-- The button comes after what it would do, not before it. -->
		<button
			type="button"
			class="btn primary apply"
			disabled={!summary.any || busy}
			onclick={onapply}
		>
			{busy ? t('tidy.working') : confirm}
		</button>
	</section>
{/if}

<style>
	.summary {
		margin-top: 14px;
		padding: 12px 14px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
	}

	h3 {
		margin: 0 0 6px;
		font-size: 10.5px;
	}

	h3:not(:first-child) {
		margin-top: 12px;
	}

	.counts {
		display: grid;
		gap: 3px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 13px;
		color: var(--onsa-text-primary);
	}

	.counts .numeric {
		display: inline-block;
		min-width: 2.5em;
		color: var(--onsa-role-active);
		text-align: right;
	}

	.skips .numeric {
		color: var(--onsa-role-caution);
	}

	.nothing {
		margin: 10px 0 0;
		font-size: 12px;
	}

	.apply {
		margin-top: 14px;
	}
</style>
