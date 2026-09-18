<!--
	One song's tags, typed by hand (SPEC section 8).

	Two steps, and the order of them is the point: what is typed here is
	saved in Onsa first, where it can be taken back, and reaches the file
	itself only when the listener asks for that separately. A listener who
	wants their own corrections without touching their files never has to
	press the second button.
-->
<script lang="ts">
	import {
		failureKey,
		trackApply,
		trackFields,
		trackWrite,
		type TrackFields,
		type TypedField
	} from '$lib/backend';
	import { db } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import Modal from './Modal.svelte';

	interface Props {
		/** Which song is being edited. */
		trackId: number;
		/** Called when the editor is done with. */
		onclose: () => void;
	}

	const { trackId, onclose }: Props = $props();

	/** The fields that are a line, not a paragraph. */
	const SHORT = ['track_number', 'disc_number', 'year'];

	let track = $state<TrackFields | null>(null);
	let typed = $state<Record<string, string>>({});
	let failure = $state<MessageKey | null>(null);
	let note = $state<MessageKey | null>(null);
	let busy = $state(false);

	$effect(() => {
		const id = trackId;
		trackFields(id)
			.then((next) => {
				track = next;
				typed = Object.fromEntries(next.fields.map((field) => [field.name, field.value ?? '']));
			})
			.catch((error) => (failure = failureKey(error)));
	});

	/** What has been typed that is not what is already shown. */
	const changed = $derived.by((): TypedField[] => {
		if (!track) return [];
		return track.fields
			.filter((field) => (typed[field.name] ?? '') !== (field.value ?? ''))
			.map((field) => ({ field: field.name, value: typed[field.name]?.trim() || null }));
	});

	const gainShown = $derived.by(() => {
		const gain = track?.gain;
		if (!gain) return false;
		return (
			gain.trackGainDb !== null ||
			gain.albumGainDb !== null ||
			gain.trackPeak !== null ||
			gain.albumPeak !== null
		);
	});

	async function save(): Promise<void> {
		if (!track || changed.length === 0) return;
		busy = true;
		try {
			await trackApply(t('editor.title'), track.trackId, changed);
			track = await trackFields(track.trackId);
			typed = Object.fromEntries(track.fields.map((field) => [field.name, field.value ?? '']));
			note = 'editor.saved';
			failure = null;
		} catch (error) {
			failure = failureKey(error);
			note = null;
		} finally {
			busy = false;
		}
	}

	async function write(): Promise<void> {
		if (!track) return;
		busy = true;
		try {
			const report = await trackWrite(t('editor.title'), track.trackId);
			track = await trackFields(track.trackId);
			note = report.failed.length > 0 ? null : 'editor.written';
			failure = report.failed.length > 0 ? 'editor.writeFailed' : null;
		} catch (error) {
			failure = failureKey(error);
			note = null;
		} finally {
			busy = false;
		}
	}

	/** Puts a field back to what the file says, by clearing Onsa's value. */
	async function revert(name: string): Promise<void> {
		if (!track) return;
		busy = true;
		try {
			await trackApply(t('editor.revert'), track.trackId, [{ field: name, value: null }]);
			track = await trackFields(track.trackId);
			typed = Object.fromEntries(track.fields.map((field) => [field.name, field.value ?? '']));
			note = null;
			failure = null;
		} catch (error) {
			failure = failureKey(error);
		} finally {
			busy = false;
		}
	}
</script>

<Modal title={t('editor.title')} wide {onclose}>
	{#if track}
		<p class="path muted">{track.path}</p>

		<div class="fields">
			{#each track.fields as field (field.name)}
				<label class="field-row" class:wide={field.name === 'lyrics'}>
					<span class="name label">
						{t(`field.${field.name}` as MessageKey)}
						{#if field.edited}
							<span class="mark" title={t('editor.editedHere')}>•</span>
						{/if}
					</span>
					{#if field.name === 'lyrics'}
						<textarea class="field words" rows="8" bind:value={typed[field.name]}></textarea>
					{:else}
						<input
							class="field"
							class:short={SHORT.includes(field.name)}
							type="text"
							bind:value={typed[field.name]}
						/>
					{/if}
					{#if field.edited}
						<button
							type="button"
							class="chip revert"
							disabled={busy}
							onclick={() => revert(field.name)}>{t('editor.revert')}</button
						>
					{/if}
				</label>
			{/each}
		</div>

		<p class="note muted later">{t('editor.laterFields')}</p>

		<section class="gain">
			<h3 class="label">{t('editor.gain')}</h3>
			{#if gainShown}
				<dl>
					<dt class="label">{t('editor.gainTrack')}</dt>
					<dd class="numeric">
						{track.gain.trackGainDb === null ? '—' : db(track.gain.trackGainDb)}
						{#if track.gain.trackPeak !== null}
							· {t('editor.peak', { value: track.gain.trackPeak.toFixed(6) })}
						{/if}
					</dd>
					<dt class="label">{t('editor.gainAlbum')}</dt>
					<dd class="numeric">
						{track.gain.albumGainDb === null ? '—' : db(track.gain.albumGainDb)}
						{#if track.gain.albumPeak !== null}
							· {t('editor.peak', { value: track.gain.albumPeak.toFixed(6) })}
						{/if}
					</dd>
				</dl>
			{:else}
				<p class="note muted">{t('editor.gainNone')}</p>
			{/if}
		</section>

		{#if note}
			<p class="note said">{t(note)}</p>
		{/if}
		{#if failure}
			<p class="note fault-text">{t(failure, { reason: '' })}</p>
		{/if}
	{:else if failure}
		<p class="note fault-text">{t(failure, { reason: '' })}</p>
	{/if}

	{#snippet footer()}
		<button type="button" class="btn" onclick={onclose}>{t('editor.close')}</button>
		<button
			type="button"
			class="btn"
			disabled={busy || !track?.anyUnwritten}
			onclick={write}
			title={track?.anyUnwritten ? t('editor.unwritten') : ''}>{t('editor.write')}</button
		>
		<button
			type="button"
			class="btn primary"
			disabled={busy || changed.length === 0}
			onclick={save}>{t('editor.save')}</button
		>
	{/snippet}
</Modal>

<style>
	.path {
		margin: 0 0 12px;
		font-size: 11.5px;
		overflow-wrap: anywhere;
	}

	.fields {
		display: grid;
		gap: 10px;
	}

	.field-row {
		display: grid;
		grid-template-columns: 120px minmax(0, 1fr) auto;
		align-items: center;
		gap: 10px;
	}

	.field-row.wide {
		align-items: start;
	}

	@container content (max-width: 560px) {
		.field-row {
			grid-template-columns: minmax(0, 1fr);
			gap: 4px;
		}
	}

	.name {
		font-size: 11px;
	}

	/* A field Onsa holds a value for rather than the file. */
	.mark {
		color: var(--onsa-role-adjustable);
	}

	.field {
		min-width: 0;
		width: 100%;
	}

	.field.short {
		max-width: 120px;
	}

	.words {
		font-family: inherit;
		font-size: 12.5px;
		line-height: 1.5;
		resize: vertical;
	}

	.revert {
		white-space: nowrap;
	}

	.later {
		margin: 12px 0 0;
	}

	.gain {
		margin-top: 16px;
	}

	.gain h3 {
		margin: 0 0 6px;
		font-size: 11px;
		font-weight: 500;
	}

	.gain dl {
		display: grid;
		grid-template-columns: auto minmax(0, 1fr);
		gap: 4px 14px;
		margin: 0;
		font-size: 12.5px;
	}

	.gain dd {
		margin: 0;
		color: var(--onsa-role-adjustable);
	}

	.note {
		margin: 10px 0 0;
		font-size: 12px;
	}

	.said {
		color: var(--onsa-role-active);
	}
</style>
