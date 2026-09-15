<!--
	What Onsa would write differently about the tags it already has
	(SPEC §8). No internet at all: this is the library reading itself.

	Every row says what it would change and why. The reasons that are plain
	corrections arrive ticked; the one about capitals does not, because how
	somebody capitalised a title is often a decision rather than a mistake,
	and Onsa is not the judge of that.
-->
<script lang="ts">
	import type { Suggestion } from '$lib/backend';
	import { fileName, relativeTo } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';

	interface Props {
		/** What was suggested. */
		found: Suggestion[];
		/** The folder the rows are shown from, when there is one. */
		root: string;
		/** Which rows are ticked, by their key. */
		ticks: Record<string, boolean>;
		/** Ticking one row. */
		ontick: (key: string, on: boolean) => void;
	}

	const { found, root, ticks, ontick }: Props = $props();

	const WHY: Record<string, MessageKey> = {
		residue: 'auto.residue',
		capitals: 'auto.capitals',
		credit: 'auto.credit',
		spelling: 'auto.spelling',
		albumArtist: 'auto.albumArtist'
	};

	/** What a field is called, falling back to its own name. */
	function fieldLabel(field: string): string {
		return t(`field.${field}` as never) || field.replace(/_/g, ' ');
	}

	export function keyOf(one: Suggestion): string {
		return `${one.trackId}:${one.field}`;
	}
</script>

<ul class="suggestions">
	{#each found as one (keyOf(one))}
		<li class="one">
			<label class="take">
				<input
					type="checkbox"
					checked={ticks[keyOf(one)] ?? false}
					onchange={(event) => ontick(keyOf(one), event.currentTarget.checked)}
				/>
				<span class="field label">{fieldLabel(one.field)}</span>
			</label>
			<span class="file ellipsis" title={one.path}>
				{root ? relativeTo(one.path, root) : fileName(one.path)}
			</span>
			<span class="was ellipsis">{one.current ?? ''}</span>
			<span class="arrow" aria-hidden="true">→</span>
			<span class="now ellipsis">{one.value}</span>
			<span class="reasons">
				{#each one.reasons as reason (reason)}
					<span class="reason label">{t(WHY[reason] ?? 'auto.residue')}</span>
				{/each}
			</span>
		</li>
	{/each}
</ul>

<style>
	.suggestions {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 2px;
		width: 100%;
		margin: 0;
		padding: 0;
		list-style: none;
	}

	.one {
		display: grid;
		grid-template-columns:
			minmax(0, auto) minmax(0, 1.2fr) minmax(0, 1fr) auto minmax(0, 1fr)
			minmax(0, auto);
		align-items: center;
		gap: 8px;
		min-width: 0;
		padding: 4px 8px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-well);
		font-size: 12px;
	}

	.take {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.file {
		color: var(--onsa-text-secondary);
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

	.reasons {
		display: flex;
		flex-wrap: wrap;
		gap: 4px;
		min-width: 0;
	}

	.reason {
		padding: 1px 6px;
		border-radius: var(--onsa-radius-sm);
		background: var(--onsa-surface-raised);
		color: var(--onsa-role-adjustable);
		font-size: 10px;
	}

	/* Narrow: the row becomes two lines rather than six squeezed columns. */
	@container content (max-width: 900px) {
		.one {
			grid-template-columns: minmax(0, auto) minmax(0, 1fr) auto minmax(0, 1fr);
		}

		.file {
			grid-column: 1 / -1;
			order: -1;
		}

		.reasons {
			grid-column: 1 / -1;
		}
	}
</style>
