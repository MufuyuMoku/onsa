<!--
	Which folder the tidying is held to, said plainly and never out of sight
	(SPEC section 8).

	This is the one thing that must never be in doubt, so it is not a setting
	tucked away somewhere: it sits above the work, it is coloured by what it
	says, and the state that can reach the whole library looks like a warning
	because that is what it is.
-->
<script lang="ts">
	import { fileName } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import { scope, pickScopeFolder, setScopeLimit, clearScope } from '$lib/tidy.svelte';
	import Icon from './Icon.svelte';

	const held = $derived(scope.value?.folder != null);
	/** The last part of the path, which is the part that identifies it. */
	const leaf = $derived(fileName((scope.value?.folder ?? '').replace(/[\\/]+$/, '')));
	let limit = $state('');

	$effect(() => {
		const current = scope.value?.limit;
		if (current !== undefined && limit === '') limit = String(current);
	});
</script>

<section class="bar" class:held aria-live="polite">
	<span class="mark" aria-hidden="true">
		<Icon name={held ? 'shelf' : 'info'} />
	</span>
	<div class="what">
		<p class="label">{held ? t('tidy.scopeHeld') : t('tidy.scopeWhole')}</p>
		{#if held}
			<!-- The name first, because that is what tells one folder from
			     another; the whole path under it, wrapped rather than cut,
			     because a path trimmed in the middle hides its own end. -->
			<p class="folder" title={scope.value?.folder ?? ''}>{leaf}</p>
			<p class="path numeric">{scope.value?.folder}</p>
		{:else}
			<p class="folder warn">{t('tidy.scopeWholeHint')}</p>
		{/if}
	</div>
	<p class="count numeric muted">{t('tidy.inScope', { n: scope.value?.tracks ?? 0 })}</p>
	<label class="limit">
		<span class="label">{t('tidy.limit')}</span>
		<input
			class="field numeric"
			type="number"
			min="1"
			max={scope.value?.maxLimit ?? 500}
			bind:value={limit}
			onchange={() => setScopeLimit(Number(limit) || 1)}
		/>
	</label>
	<button type="button" class="btn" onclick={pickScopeFolder}>{t('tidy.pickFolder')}</button>
	{#if held}
		<button type="button" class="btn" onclick={clearScope}>{t('tidy.clearFolder')}</button>
	{/if}
</section>

<style>
	.bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px 14px;
		padding: 10px 14px;
		border: var(--onsa-hairline) solid var(--onsa-role-caution);
		border-left-width: 3px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
	}

	/* Held to a folder is the safe state, and it says so in a calmer colour
	   than the one that can reach everything. */
	.bar.held {
		border-color: var(--onsa-role-active);
	}

	.mark {
		display: grid;
		place-items: center;
		width: 26px;
		height: 26px;
		border-radius: 50%;
		background: var(--onsa-surface-well);
		color: var(--onsa-role-caution);
	}

	.bar.held .mark {
		color: var(--onsa-role-active);
	}

	.mark :global(svg) {
		width: 15px;
		height: 15px;
	}

	.what {
		flex: 1 1 240px;
		min-width: 0;
	}

	.what p {
		margin: 0;
	}

	.folder {
		font-size: 13px;
		color: var(--onsa-text-primary);
	}

	.folder.warn {
		color: var(--onsa-role-caution);
	}

	.path {
		font-size: 11px;
		color: var(--onsa-text-secondary);
		/* Wrapped, never trimmed: the end of a path is the part that says
		   which folder this is. */
		overflow-wrap: anywhere;
	}

	.count {
		font-size: 12px;
		white-space: nowrap;
	}

	.limit {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.limit input {
		width: 72px;
	}
</style>
