<!--
	The smart playlist rule editor (SPEC section 6.3).

	A rule is a field, a comparison and a value. The field decides which
	comparisons are offered and what the value looks like, so a rule the
	backend would refuse cannot be built here. A preview counts what the
	rules match right now, before anything is saved.
-->
<script lang="ts">
	import {
		smartPreview,
		type Rule,
		type RuleField,
		type RuleKind,
		type RuleOp,
		type Rules,
		type SortField,
		type Track
	} from '$lib/backend';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import Icon from './Icon.svelte';
	import Modal from './Modal.svelte';

	interface Props {
		title: string;
		/** The rules the editor starts from. */
		value: Rules;
		/** The word on the button that saves. */
		confirm: string;
		onconfirm: (rules: Rules) => void;
		oncancel: () => void;
	}

	const { title, value, confirm, onconfirm, oncancel }: Props = $props();

	/** How many tracks the preview counts at most. */
	const PREVIEW_LIMIT = 500;
	/** How long the editor waits before asking for a preview again. */
	const PREVIEW_DELAY_MS = 250;

	const FIELDS: { field: RuleField; kind: RuleKind }[] = [
		{ field: 'title', kind: 'text' },
		{ field: 'artist', kind: 'text' },
		{ field: 'album', kind: 'text' },
		{ field: 'album_artist', kind: 'text' },
		{ field: 'genre', kind: 'text' },
		{ field: 'codec', kind: 'text' },
		{ field: 'folder', kind: 'text' },
		{ field: 'year', kind: 'number' },
		{ field: 'rating', kind: 'number' },
		{ field: 'play_count', kind: 'number' },
		{ field: 'duration', kind: 'number' },
		{ field: 'last_played', kind: 'date' },
		{ field: 'added_at', kind: 'date' }
	];

	const OPS: Record<RuleKind, { op: RuleOp; label: MessageKey }[]> = {
		text: [
			{ op: 'contains', label: 'op.contains' },
			{ op: 'not_contains', label: 'op.not_contains' },
			{ op: 'is', label: 'op.is' },
			{ op: 'is_not', label: 'op.is_not' },
			{ op: 'starts_with', label: 'op.starts_with' },
			{ op: 'not_starts_with', label: 'op.not_starts_with' }
		],
		number: [
			{ op: '=', label: 'op.eq' },
			{ op: '!=', label: 'op.ne' },
			{ op: '<', label: 'op.lt' },
			{ op: '<=', label: 'op.le' },
			{ op: '>', label: 'op.gt' },
			{ op: '>=', label: 'op.ge' },
			{ op: 'between', label: 'op.between' }
		],
		date: [
			{ op: 'in_last', label: 'op.in_last' },
			{ op: 'not_in_last', label: 'op.not_in_last' },
			{ op: 'before', label: 'op.before' },
			{ op: 'after', label: 'op.after' }
		]
	};

	const SORT_FIELDS: SortField[] = [
		'title',
		'artist',
		'album',
		'year',
		'duration',
		'rating',
		'play_count',
		'last_played',
		'added_at',
		'random'
	];

	function kindOf(field: RuleField): RuleKind {
		return FIELDS.find((entry) => entry.field === field)?.kind ?? 'text';
	}

	/** A value that suits a comparison the moment it is chosen. */
	function defaultValue(op: RuleOp): Rule['value'] {
		if (op === 'between') return [0, 0];
		if (op === 'in_last' || op === 'not_in_last') return { days: 30 };
		if (op === 'before' || op === 'after') return todayStamp();
		if (OPS.number.some((entry) => entry.op === op)) return 0;
		return '';
	}

	/** Today, as the date field writes it. */
	function todayStamp(): string {
		return new Date().toISOString().slice(0, 10);
	}

	// A copy, so closing without saving leaves the playlist as it was: the
	// editor deliberately takes the rules once and works on its own from
	// there.
	// svelte-ignore state_referenced_locally
	let match = $state<'all' | 'any'>(value.match);
	// svelte-ignore state_referenced_locally
	let rules = $state<Rule[]>(value.rules.map((rule) => ({ ...rule })));
	// svelte-ignore state_referenced_locally
	let sortField = $state<SortField>(value.sort.field);
	// svelte-ignore state_referenced_locally
	let descending = $state(value.sort.descending);
	// svelte-ignore state_referenced_locally
	let limit = $state<string>(value.limit === null ? '' : String(value.limit));

	const built = $derived<Rules>({
		match,
		rules: rules.map((rule) => ({ ...rule, value: cleaned(rule) })),
		sort: { field: sortField, descending },
		limit: limit.trim() === '' ? null : Math.max(1, Number(limit) || 1)
	});

	/** A rule's value as the backend wants it: numbers as numbers. */
	function cleaned(rule: Rule): Rule['value'] {
		const kind = kindOf(rule.field);
		if (rule.op === 'between') {
			const pair = Array.isArray(rule.value) ? rule.value : [0, 0];
			return [Number(pair[0]) || 0, Number(pair[1]) || 0];
		}
		if (rule.op === 'in_last' || rule.op === 'not_in_last') {
			const days = typeof rule.value === 'object' && rule.value !== null && 'days' in rule.value
				? Number(rule.value.days)
				: 30;
			return { days: Number.isFinite(days) && days > 0 ? days : 1 };
		}
		if (kind === 'number') return Number(rule.value) || 0;
		return String(rule.value ?? '');
	}

	function addRule(): void {
		rules = [...rules, { field: 'artist', op: 'contains', value: '' }];
	}

	function removeRule(index: number): void {
		rules = rules.filter((_, at) => at !== index);
	}

	/** Changing the field can leave the comparison behind; it moves with it. */
	function setField(index: number, field: RuleField): void {
		const op = OPS[kindOf(field)][0]!.op;
		rules = rules.map((rule, at) =>
			at === index ? { field, op, value: defaultValue(op) } : rule
		);
	}

	function setOp(index: number, op: RuleOp): void {
		rules = rules.map((rule, at) => (at === index ? { ...rule, op, value: defaultValue(op) } : rule));
	}

	function setValue(index: number, value: Rule['value']): void {
		rules = rules.map((rule, at) => (at === index ? { ...rule, value } : rule));
	}

	function pair(rule: Rule): [number, number] {
		return Array.isArray(rule.value) ? rule.value : [0, 0];
	}

	function days(rule: Rule): number {
		return typeof rule.value === 'object' && rule.value !== null && 'days' in rule.value
			? Number(rule.value.days)
			: 30;
	}

	// The preview follows the rules, a moment behind, so typing is not a
	// query per keystroke.
	let preview = $state<Track[] | null>(null);
	let failed = $state(false);

	$effect(() => {
		const rules = built;
		let cancelled = false;
		const timer = setTimeout(() => {
			smartPreview(rules, PREVIEW_LIMIT)
				.then((tracks) => {
					if (cancelled) return;
					preview = tracks;
					failed = false;
				})
				.catch(() => {
					if (cancelled) return;
					preview = null;
					failed = true;
				});
		}, PREVIEW_DELAY_MS);
		return () => {
			cancelled = true;
			clearTimeout(timer);
		};
	});
</script>

<Modal {title} onclose={oncancel} wide>
	<div class="head">
		<label class="inline">
			<span class="label">{t('rules.match')}</span>
			<select class="field" bind:value={match}>
				<option value="all">{t('rules.matchAll')}</option>
				<option value="any">{t('rules.matchAny')}</option>
			</select>
		</label>
		<button type="button" class="chip" onclick={addRule}>
			<Icon name="plus" />
			{t('rules.add')}
		</button>
	</div>

	{#if rules.length === 0}
		<p class="muted none">{t('rules.none')}</p>
	{/if}

	<ul class="rules">
		{#each rules as rule, index (index)}
			<li class="rule">
				<select
					class="field"
					aria-label={t('rules.field')}
					value={rule.field}
					onchange={(event) => setField(index, event.currentTarget.value as RuleField)}
				>
					{#each FIELDS as entry (entry.field)}
						<option value={entry.field}>{t(`field.${entry.field}` as MessageKey)}</option>
					{/each}
				</select>
				<select
					class="field"
					aria-label={t('rules.op')}
					value={rule.op}
					onchange={(event) => setOp(index, event.currentTarget.value as RuleOp)}
				>
					{#each OPS[kindOf(rule.field)] as entry (entry.op)}
						<option value={entry.op}>{t(entry.label)}</option>
					{/each}
				</select>

				<div class="value">
					{#if rule.op === 'between'}
						<input
							class="field numeric"
							type="number"
							aria-label={t('rules.value')}
							value={pair(rule)[0]}
							oninput={(event) =>
								setValue(index, [Number(event.currentTarget.value) || 0, pair(rule)[1]])}
						/>
						<span class="muted">{t('rules.and')}</span>
						<input
							class="field numeric"
							type="number"
							aria-label={t('rules.value')}
							value={pair(rule)[1]}
							oninput={(event) =>
								setValue(index, [pair(rule)[0], Number(event.currentTarget.value) || 0])}
						/>
					{:else if rule.op === 'in_last' || rule.op === 'not_in_last'}
						<input
							class="field numeric"
							type="number"
							min="1"
							aria-label={t('rules.value')}
							value={days(rule)}
							oninput={(event) =>
								setValue(index, { days: Math.max(1, Number(event.currentTarget.value) || 1) })}
						/>
						<span class="muted">{t('rules.days')}</span>
					{:else if rule.op === 'before' || rule.op === 'after'}
						<input
							class="field"
							type="date"
							aria-label={t('rules.value')}
							value={String(rule.value ?? '')}
							oninput={(event) => setValue(index, event.currentTarget.value)}
						/>
					{:else if kindOf(rule.field) === 'number'}
						<input
							class="field numeric"
							type="number"
							aria-label={t('rules.value')}
							value={Number(rule.value) || 0}
							oninput={(event) => setValue(index, Number(event.currentTarget.value) || 0)}
						/>
					{:else}
						<input
							class="field"
							type="text"
							aria-label={t('rules.value')}
							value={String(rule.value ?? '')}
							oninput={(event) => setValue(index, event.currentTarget.value)}
						/>
					{/if}
				</div>

				<button
					type="button"
					class="icon-btn small"
					aria-label={t('rules.remove')}
					title={t('rules.remove')}
					onclick={() => removeRule(index)}
				>
					<Icon name="trash" />
				</button>
			</li>
		{/each}
	</ul>

	<div class="foot">
		<label class="inline">
			<span class="label">{t('rules.sort')}</span>
			<select class="field" bind:value={sortField}>
				{#each SORT_FIELDS as field (field)}
					<option value={field}>{t(`sortField.${field}` as MessageKey)}</option>
				{/each}
			</select>
		</label>
		<label class="inline">
			<input type="checkbox" bind:checked={descending} />
			<span>{t('rules.descending')}</span>
		</label>
		<label class="inline">
			<span class="label">{t('rules.limit')}</span>
			<input
				class="field numeric limit"
				type="number"
				min="1"
				placeholder={t('rules.noLimit')}
				bind:value={limit}
			/>
		</label>
	</div>

	<p class="count numeric" class:fault-text={failed}>
		{#if failed}
			{t('rules.invalid')}
		{:else if preview === null}
			&nbsp;
		{:else if preview.length >= PREVIEW_LIMIT}
			{t('rules.previewMore', { n: PREVIEW_LIMIT })}
		{:else}
			{t('rules.preview', { n: preview.length })}
		{/if}
	</p>

	{#snippet footer()}
		<button type="button" class="btn" onclick={oncancel}>{t('playlist.cancel')}</button>
		<button type="button" class="btn primary" disabled={failed} onclick={() => onconfirm(built)}>
			{confirm}
		</button>
	{/snippet}
</Modal>

<style>
	.head,
	.foot {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 10px 14px;
	}

	.foot {
		margin-top: 14px;
		padding-top: 14px;
		border-top: var(--onsa-hairline) solid var(--onsa-surface-line);
	}

	.inline {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.chip {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		margin-left: auto;
	}

	.chip :global(svg) {
		width: 13px;
		height: 13px;
	}

	.none {
		margin: 14px 0 0;
		font-size: 12px;
	}

	.rules {
		display: grid;
		gap: 8px;
		margin: 14px 0 0;
		padding: 0;
		list-style: none;
	}

	.rule {
		display: grid;
		grid-template-columns: minmax(0, 1fr) minmax(0, 1fr) minmax(0, 1.4fr) auto;
		gap: 6px;
		align-items: center;
	}

	/* Below this the four parts of a rule cannot sit side by side without
	   squeezing the value out of reach, so they stack instead. */
	@container content (max-width: 620px) {
		.rule {
			grid-template-columns: minmax(0, 1fr) auto;
		}

		.rule .value {
			grid-column: 1 / -1;
		}
	}

	.value {
		display: flex;
		align-items: center;
		gap: 6px;
		min-width: 0;
	}

	.value input {
		min-width: 0;
		flex: 1;
	}

	.limit {
		width: 90px;
	}

	.icon-btn.small {
		width: 28px;
		height: 28px;
	}

	.icon-btn.small :global(svg) {
		width: 14px;
		height: 14px;
	}

	.count {
		margin: 12px 0 0;
		font-size: 12px;
		color: var(--onsa-text-secondary);
	}
</style>
