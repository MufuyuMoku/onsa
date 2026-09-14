<!--
	Tidying metadata by hand (SPEC section 8), with the rails in plain sight.

	Three things can be done here and each one works the same way: say what
	you want, read the summary of what that would do, then press the button
	underneath it. Every run can be taken back from the list at the bottom.
-->
<script lang="ts">
	import type { EditBatch, RenamePlan, TidyChange, TidyReport, TidySummary } from '$lib/backend';
	import { fileName, relativeTo } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import {
		applyEdits,
		applyRename,
		history,
		loadTidy,
		planRename,
		previewEdits,
		previewWrites,
		scope,
		undo,
		writeToFiles
	} from '$lib/tidy.svelte';
	import Icon from './Icon.svelte';
	import ScopeBar from './ScopeBar.svelte';
	import TidySummaryPanel from './TidySummary.svelte';

	/** Which of the three jobs is open. */
	type Job = 'edit' | 'write' | 'rename';
	let job = $state<Job>('edit');

	/** The fields a bulk edit can set. */
	const FIELDS = ['artist', 'album', 'album_artist', 'genre', 'year'] as const;
	let field = $state<string>('artist');
	let value = $state('');

	let pattern = $state('{album_artist}/{album}/{track:02} {title}');
	let root = $state('');

	let summary = $state<TidySummary | null>(null);
	let plan = $state<RenamePlan | null>(null);
	let report = $state<TidyReport | null>(null);
	let runs = $state<EditBatch[]>([]);

	$effect(() => {
		void loadTidy();
	});

	$effect(() => {
		// The history follows whatever the last run did to it.
		void report;
		history(20).then((list) => {
			if (list) runs = list;
		});
	});

	// Changing what is being asked for makes the old summary stale, and a
	// stale summary above an apply button is the one thing to avoid.
	$effect(() => {
		void job;
		void field;
		void value;
		void pattern;
		void root;
		summary = null;
		plan = null;
	});

	const ids = $derived(scope.tracks.map((track) => track.id));

	function changes(): TidyChange[] {
		const wanted = value.trim();
		return ids.map((trackId) => ({
			trackId,
			field,
			value: wanted === '' ? null : wanted
		}));
	}

	async function look(): Promise<void> {
		report = null;
		if (job === 'edit') {
			summary = await previewEdits(changes());
		} else if (job === 'write') {
			summary = await previewWrites(ids);
		} else {
			const made = await planRename(root.trim() || scope.value?.folder || '', pattern, ids);
			plan = made;
			summary = made?.summary ?? null;
		}
	}

	async function apply(): Promise<void> {
		if (job === 'edit') {
			report = await applyEdits(t('tidy.noteEdit', { field: t(`field.${field}` as never) }), changes());
		} else if (job === 'write') {
			report = await writeToFiles(t('tidy.noteWrite'), ids);
		} else {
			report = await applyRename(
				t('tidy.noteRename'),
				root.trim() || scope.value?.folder || '',
				pattern,
				ids
			);
		}
		summary = null;
		plan = null;
	}

	function when(stamp: number): string {
		return new Date(stamp).toLocaleString();
	}

	/** The folder the rename would build under, as it is actually used. */
	const intoFolder = $derived(root.trim() || scope.value?.folder || '');
</script>

<div class="tidy">
	<header class="head">
		<h1>{t('nav.tidy')}</h1>
	</header>

	<!-- Always there, whatever else is on screen. -->
	<ScopeBar />

	<div class="jobs">
		{#each [['edit', 'tidy.jobEdit'], ['write', 'tidy.jobWrite'], ['rename', 'tidy.jobRename']] as [key, label] (key)}
			<button
				type="button"
				class="chip"
				aria-pressed={job === key}
				onclick={() => (job = key as Job)}>{t(label as never)}</button
			>
		{/each}
	</div>

	<section class="job">
		{#if job === 'edit'}
			<p class="muted what">{t('tidy.editWhat')}</p>
			<div class="row">
				<label class="inline">
					<span class="label">{t('rules.field')}</span>
					<select class="field" bind:value={field}>
						{#each FIELDS as name (name)}
							<option value={name}>{t(`field.${name}` as never)}</option>
						{/each}
					</select>
				</label>
				<label class="inline grow">
					<span class="label">{t('rules.value')}</span>
					<input class="field grow" type="text" bind:value placeholder={t('tidy.valueEmpty')} />
				</label>
			</div>
		{:else if job === 'write'}
			<p class="muted what">{t('tidy.writeWhat')}</p>
		{:else}
			<p class="muted what">{t('tidy.renameWhat')}</p>
			<div class="row">
				<label class="inline grow">
					<span class="label">{t('tidy.pattern')}</span>
					<input class="field grow numeric" type="text" bind:value={pattern} />
				</label>
			</div>
			<div class="row">
				<label class="inline grow">
					<span class="label">{t('tidy.renameRoot')}</span>
					<input
						class="field grow numeric"
						type="text"
						bind:value={root}
						placeholder={scope.value?.folder ?? ''}
					/>
				</label>
			</div>
		{/if}

		<button type="button" class="btn" disabled={scope.busy} onclick={look}>
			{t('tidy.look')}
		</button>

		{#if plan && plan.moves.length > 0}
			<ol class="moves">
				{#each plan.moves.slice(0, 12) as one (one.trackId)}
					<li class="move" class:clash={one.clash != null}>
						<span class="ellipsis">{fileName(one.from)}</span>
						<span class="arrow" aria-hidden="true">→</span>
						<span class="ellipsis numeric">{one.clash ?? relativeTo(one.to, intoFolder)}</span>
					</li>
				{/each}
			</ol>
			{#if plan.moves.length > 12}
				<p class="muted rest numeric">{t('rules.previewRest', { n: plan.moves.length - 12 })}</p>
			{/if}
		{/if}

		<TidySummaryPanel
			{summary}
			busy={scope.busy}
			confirm={t('tidy.apply')}
			onapply={apply}
		/>

		{#if report}
			<p class="done" class:fault-text={report.failed.length > 0}>
				{t('tidy.done', { n: report.changed })}
				{#if report.failed.length > 0}
					· {t('tidy.someFailed', { n: report.failed.length })}
				{/if}
			</p>
			{#if report.failed.length > 0}
				<ul class="failures">
					{#each report.failed.slice(0, 5) as why (why)}
						<li class="ellipsis numeric">{why}</li>
					{/each}
				</ul>
			{/if}
		{/if}

		{#if scope.failure}
			<p class="fault-text done">{t(scope.failure)}</p>
		{/if}
	</section>

	<section class="history">
		<h2 class="label">{t('tidy.history')}</h2>
		{#if runs.length === 0}
			<p class="muted what">{t('tidy.noHistory')}</p>
		{:else}
			<ul class="runs">
				{#each runs as run (run.id)}
					<li class="run" class:undone={run.undoneAt != null}>
						<span class="ellipsis">{run.note}</span>
						<span class="numeric muted">{t('tidy.steps', { n: run.steps })}</span>
						<span class="numeric muted when">{when(run.createdAt)}</span>
						{#if run.undoneAt == null}
							<button
								type="button"
								class="btn"
								disabled={scope.busy}
								onclick={() => undo(run.id).then((r) => (report = r ? null : report))}
							>
								<Icon name="back" />
								{t('tidy.undo')}
							</button>
						{:else}
							<span class="muted numeric">{t('tidy.undone')}</span>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>

<style>
	.tidy {
		display: flex;
		flex-direction: column;
		gap: 12px;
		min-height: 0;
		height: 100%;
		padding: 14px 16px;
		overflow-y: auto;
		scrollbar-width: thin;
	}

	.head h1 {
		margin: 0;
		font-size: 15px;
		font-weight: 500;
		color: var(--onsa-text-primary);
	}

	.jobs {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.job {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 10px;
	}

	.what {
		margin: 0;
		font-size: 12px;
	}

	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px 14px;
		width: 100%;
	}

	.inline {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.grow {
		flex: 1 1 220px;
		min-width: 0;
	}

	.moves {
		display: grid;
		gap: 1px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.move {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto minmax(0, 1.4fr);
		align-items: center;
		gap: 8px;
		padding: 3px 8px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		font-size: 12px;
	}

	.move.clash {
		color: var(--onsa-role-caution);
	}

	.arrow {
		color: var(--onsa-text-secondary);
	}

	.rest,
	.done {
		margin: 0;
		font-size: 12px;
	}

	.failures {
		display: grid;
		gap: 2px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 11.5px;
		color: var(--onsa-role-caution);
	}

	.history {
		margin-top: auto;
		padding-top: 12px;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
	}

	.history h2 {
		margin: 0 0 8px;
		font-size: 10.5px;
	}

	.runs {
		display: grid;
		gap: 2px;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.run {
		display: grid;
		grid-template-columns: minmax(0, 1fr) auto auto auto;
		align-items: center;
		gap: 10px;
		padding: 4px 8px;
		border-radius: var(--onsa-radius-sm);
		font-size: 13px;
	}

	.run:hover {
		background: var(--onsa-surface-raised);
	}

	.run.undone {
		color: var(--onsa-text-secondary);
	}

	.run .btn {
		display: inline-flex;
		align-items: center;
		gap: 5px;
	}

	.run .btn :global(svg) {
		width: 13px;
		height: 13px;
	}

	.run span {
		font-size: 12px;
	}

	/* At the narrowest the time of the run is the first thing to go. */
	@container content (max-width: 640px) {
		.run {
			grid-template-columns: minmax(0, 1fr) auto auto;
		}

		.when {
			display: none;
		}

		.move {
			grid-template-columns: minmax(0, 1fr);
		}

		.arrow {
			display: none;
		}
	}
</style>
