<!--
	What the internet says about the tracks in the folder (SPEC §8).

	One row per track. How sure Onsa is comes first, as a number and as a
	bar, because it is what decides whether anything is ticked: a sure match
	arrives ticked, an unsure one arrives to be read. When the sound and the
	file name disagree, both are shown side by side and neither is ticked —
	that one is the listener's to settle, and the row says so.

	Nothing here applies anything. It only decides what the ticks add up to.
-->
<script lang="ts">
	import { proposedCoverUrl, type Proposal, type TrackMatch } from '$lib/backend';
	import { fileName, relativeTo } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { matching, pick, tick, tickCover } from '$lib/matching.svelte';

	interface Props {
		/** The folder the rows are shown from, when there is one. */
		root: string;
	}

	const { root }: Props = $props();

	const SOURCE: Record<Proposal['source'], MessageKey> = {
		acoustId: 'match.fromSound',
		musicBrainz: 'match.fromTags',
		fileName: 'match.fromName'
	};

	const WHY: Record<string, MessageKey> = {
		noFingerprinter: 'match.noFingerprinter',
		unreadable: 'match.unreadable',
		noKey: 'match.noKey',
		offline: 'match.offline',
		refused: 'match.refused',
		nothingFound: 'match.nothingFound'
	};

	function why(failure: string): MessageKey {
		return WHY[failure] ?? 'match.nothingFound';
	}

	/** How sure, as whole percent. */
	const percent = (confidence: number) => Math.round(confidence * 100);

	/** What a field is called, falling back to its own name. */
	function fieldLabel(field: string): string {
		return t(`field.${field}` as never);
	}
</script>

<ul class="matches">
	{#each matching.found as track (track.trackId)}
		{@const chosen = matching.picked(track)}
		<li class="match" class:settle={track.disagree && chosen === null}>
			<p class="file ellipsis" title={track.path}>
				{root ? relativeTo(track.path, root) : fileName(track.path)}
			</p>

			{#if track.failure}
				<p class="muted note">{t(why(track.failure))}</p>
			{:else}
				{#if track.disagree}
					<p class="note caution-text">{t('match.disagree')}</p>
				{/if}

				<div class="answers">
					{#each track.candidates as candidate, index (index)}
						{@const sure = percent(candidate.confidence)}
						<div class="answer" class:using={chosen === index}>
							<div class="who">
								<label class="use">
									<input
										type="radio"
										name={`match-${track.trackId}`}
										checked={chosen === index}
										onchange={() => pick(track, index)}
									/>
									<span class="label">{t(SOURCE[candidate.source])}</span>
								</label>
								<span
									class="sure numeric"
									class:trusted={candidate.trusted}
									title={t('match.sureTitle')}>{sure}%</span
								>
								<span class="bar" aria-hidden="true">
									<span
										class="fill"
										class:trusted={candidate.trusted}
										style:width={`${sure}%`}
									></span>
								</span>
							</div>

							<ul class="fields">
								{#each candidate.fields as field (field.field)}
									<li class="field" class:same={field.current === field.suggested}>
										<label class="take">
											<input
												type="checkbox"
												checked={matching.ticked(track, index, field.field)}
												disabled={chosen !== index || field.current === field.suggested}
												onchange={(event) =>
													tick(track, index, field.field, event.currentTarget.checked)}
											/>
											<span class="name label">{fieldLabel(field.field)}</span>
										</label>
										<span class="was ellipsis">{field.current ?? t('match.nothing')}</span>
										<span class="arrow" aria-hidden="true">→</span>
										<span class="now ellipsis">{field.suggested ?? ''}</span>
									</li>
								{/each}
							</ul>

							{#if candidate.cover}
								<div class="cover">
									<!-- The picture is looked at before it is agreed to. -->
									<img
										src={proposedCoverUrl(candidate.cover.key, 128)}
										alt={t('match.coverAlt')}
										width="64"
										height="64"
									/>
									<label class="take">
										<input
											type="checkbox"
											checked={matching.coverTicked(track)}
											disabled={chosen !== index}
											onchange={(event) => tickCover(track, event.currentTarget.checked)}
										/>
										<span class="label">{t('match.takeCover')}</span>
									</label>
									<span class="muted numeric size"
										>{candidate.cover.width}×{candidate.cover.height}</span
									>
								</div>
							{/if}
						</div>
					{/each}

					{#if track.candidates.length > 1 || chosen !== null}
						<label class="use none">
							<input
								type="radio"
								name={`match-${track.trackId}`}
								checked={chosen === null}
								onchange={() => pick(track, null)}
							/>
							<span class="muted">{t('match.useNone')}</span>
						</label>
					{/if}
				</div>
			{/if}
		</li>
	{/each}
</ul>

<style>
	.matches {
		display: grid;
		gap: 6px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.match {
		padding: 8px 10px;
		border: var(--onsa-hairline) solid var(--onsa-surface-line);
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
	}

	/* A row nobody has settled is marked, not hidden. */
	.match.settle {
		border-color: var(--onsa-role-caution);
	}

	.file {
		margin: 0 0 6px;
		font-size: 12.5px;
		color: var(--onsa-text-primary);
	}

	.note {
		margin: 0 0 6px;
		font-size: 12px;
	}

	.caution-text {
		color: var(--onsa-role-caution);
	}

	.answers {
		display: grid;
		gap: 6px;
	}

	.answer {
		padding: 6px 8px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		opacity: 0.72;
	}

	.answer.using {
		opacity: 1;
		outline: var(--onsa-hairline) solid var(--onsa-role-active);
	}

	.who {
		display: grid;
		grid-template-columns: auto auto minmax(60px, 1fr);
		align-items: center;
		gap: 8px;
	}

	.use,
	.take {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.use.none {
		margin-top: 2px;
		font-size: 12px;
	}

	.sure {
		font-size: 12px;
		color: var(--onsa-role-caution);
	}

	.sure.trusted {
		color: var(--onsa-role-active);
	}

	.bar {
		display: block;
		height: 4px;
		border-radius: 2px;
		background: var(--onsa-surface-well);
		overflow: hidden;
	}

	.fill {
		display: block;
		height: 100%;
		background: var(--onsa-role-caution);
	}

	.fill.trusted {
		background: var(--onsa-role-active);
	}

	.fields {
		display: grid;
		gap: 2px;
		margin: 6px 0 0;
		padding: 0;
		list-style: none;
	}

	.field {
		display: grid;
		grid-template-columns: minmax(100px, auto) minmax(0, 1fr) auto minmax(0, 1.2fr);
		align-items: center;
		gap: 8px;
		font-size: 12px;
	}

	.field.same {
		opacity: 0.55;
	}

	.was {
		color: var(--onsa-text-secondary);
	}

	.now {
		color: var(--onsa-text-primary);
	}

	.arrow {
		color: var(--onsa-text-secondary);
	}

	.cover {
		display: flex;
		align-items: center;
		gap: 10px;
		margin-top: 8px;
	}

	.cover img {
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		object-fit: cover;
	}

	.size {
		font-size: 11px;
	}

	@container content (max-width: 640px) {
		.field {
			grid-template-columns: minmax(0, 1fr) auto minmax(0, 1fr);
		}

		.field .name {
			grid-column: 1 / -1;
		}
	}
</style>
