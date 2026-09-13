<!-- DSP & EQ (SPEC sections 4.2 and 4.3). Every change is heard at once. -->
<script lang="ts">
	import { autoEqExport, autoEqImport, failureKey, type Band, type FilterType } from '$lib/backend';
	import { app } from '$lib/app.svelte';
	import { db, frequency, millis } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { player } from '$lib/player.svelte';
	import { settings, updateDsp } from '$lib/settings.svelte';
	import EqCurve from '../EqCurve.svelte';

	const GRAPHIC_RANGE = 12;
	const filters: [FilterType, MessageKey][] = [
		['peaking', 'filter.peaking'],
		['lowShelf', 'filter.lowShelf'],
		['highShelf', 'filter.highShelf'],
		['lowPass', 'filter.lowPass'],
		['highPass', 'filter.highPass'],
		['notch', 'filter.notch']
	];

	const dsp = $derived(settings.value?.dsp ?? null);
	const freqs = $derived(app.state?.graphicFreqs ?? []);
	const maxBands = $derived(app.state?.maxBands ?? 16);
	const effectivePreamp = $derived(player.snapshot?.signal.eq.preampDb ?? 0);

	let message = $state<string | null>(null);
	let failure = $state<MessageKey | null>(null);

	function setGain(index: number, gain: number): void {
		if (!dsp) return;
		updateDsp({ graphicGains: dsp.graphicGains.map((value, i) => (i === index ? gain : value)) });
	}

	function setBand(index: number, patch: Partial<Band>): void {
		if (!dsp) return;
		updateDsp({
			parametric: dsp.parametric.map((band, i) => (i === index ? { ...band, ...patch } : band))
		});
	}

	function number(text: string, fallback: number): number {
		const value = Number(text.replace(',', '.'));
		return Number.isFinite(value) ? value : fallback;
	}

	async function importPreset(): Promise<void> {
		message = null;
		failure = null;
		try {
			const imported = await autoEqImport(t('eq.importTitle'), t('eq.fileFilter'));
			if (!imported) return;
			updateDsp({
				eqEnabled: true,
				eqKind: 'parametric',
				parametric: imported.bands,
				...(imported.preampDb === null ? {} : { preampDb: imported.preampDb })
			});
			message =
				imported.skipped > 0
					? t('eq.importedSkipped', { n: imported.bands.length, skipped: imported.skipped })
					: t('eq.imported', { n: imported.bands.length });
		} catch (error) {
			failure = failureKey(error);
		}
	}

	async function exportPreset(): Promise<void> {
		message = null;
		failure = null;
		try {
			if (await autoEqExport(t('eq.exportTitle'), t('eq.fileFilter'))) message = t('eq.exported');
		} catch (error) {
			failure = failureKey(error);
		}
	}
</script>

{#if dsp}
	<div class="page">
		<section>
			<h2 class="label">{t('eq.title')}</h2>
			<div class="actions">
				<label class="check">
					<input
						type="checkbox"
						checked={dsp.eqEnabled}
						onchange={(event) => updateDsp({ eqEnabled: event.currentTarget.checked })}
					/>
					{t('eq.enabled')}
				</label>
				<span class="spacer"></span>
				<button
					type="button"
					class="chip"
					aria-pressed={dsp.eqKind === 'graphic'}
					onclick={() => updateDsp({ eqKind: 'graphic' })}>{t('eq.graphic')}</button
				>
				<button
					type="button"
					class="chip"
					aria-pressed={dsp.eqKind === 'parametric'}
					onclick={() => updateDsp({ eqKind: 'parametric' })}>{t('eq.parametric')}</button
				>
			</div>

			<EqCurve {dsp} />

			{#if dsp.eqKind === 'graphic'}
				<div class="faders">
					{#each freqs as hz, index (hz)}
						<label class="fader">
							<span class="numeric value-small">{db(dsp.graphicGains[index] ?? 0, 1)}</span>
							<input
								type="range"
								min={-GRAPHIC_RANGE}
								max={GRAPHIC_RANGE}
								step="0.5"
								value={dsp.graphicGains[index] ?? 0}
								aria-label={frequency(hz)}
								oninput={(event) => setGain(index, Number(event.currentTarget.value))}
							/>
							<span class="numeric label">{frequency(hz)}</span>
						</label>
					{/each}
				</div>
				<div class="actions">
					<button
						type="button"
						class="btn"
						onclick={() => updateDsp({ graphicGains: dsp.graphicGains.map(() => 0) })}
						>{t('eq.flat')}</button
					>
				</div>
			{:else}
				{#if dsp.parametric.length === 0}
					<p class="muted note">{t('eq.noBands')}</p>
				{:else}
					<div class="bands-scroll">
						<div class="bands">
							<span class="label">{t('eq.bandOn')}</span>
							<span class="label">{t('eq.type')}</span>
							<span class="label">{t('eq.freq')}</span>
							<span class="label">{t('eq.gain')}</span>
							<span class="label">{t('eq.q')}</span>
							<span></span>
							{#each dsp.parametric as band, index (index)}
								<input
									type="checkbox"
									checked={band.enabled}
									aria-label={t('eq.bandOn')}
									onchange={(event) => setBand(index, { enabled: event.currentTarget.checked })}
								/>
								<select
									class="field"
									value={band.kind}
									aria-label={t('eq.type')}
									onchange={(event) => setBand(index, { kind: event.currentTarget.value as FilterType })}
								>
									{#each filters as [kind, label] (kind)}
										<option value={kind}>{t(label)}</option>
									{/each}
								</select>
								<input
									class="field numeric"
									type="number"
									min="10"
									max="22000"
									step="1"
									value={band.freq}
									aria-label={t('eq.freq')}
									onchange={(event) => setBand(index, { freq: number(event.currentTarget.value, band.freq) })}
								/>
								<input
									class="field numeric"
									type="number"
									min="-24"
									max="24"
									step="0.1"
									value={band.gainDb}
									aria-label={t('eq.gain')}
									onchange={(event) =>
										setBand(index, { gainDb: number(event.currentTarget.value, band.gainDb) })}
								/>
								<input
									class="field numeric"
									type="number"
									min="0.05"
									max="20"
									step="0.01"
									value={band.q}
									aria-label={t('eq.q')}
									onchange={(event) => setBand(index, { q: number(event.currentTarget.value, band.q) })}
								/>
								<button
									type="button"
									class="btn"
									aria-label={t('eq.remove')}
									onclick={() => updateDsp({ parametric: dsp.parametric.filter((_, i) => i !== index) })}
									>×</button
								>
							{/each}
						</div>
					</div>
				{/if}
				<div class="actions">
					<button
						type="button"
						class="btn"
						disabled={dsp.parametric.length >= maxBands}
						onclick={() =>
							updateDsp({
								parametric: [
									...dsp.parametric,
									{ kind: 'peaking', freq: 1000, gainDb: 0, q: 1, enabled: true }
								]
							})}>{t('eq.addBand')}</button
					>
					{#if dsp.parametric.length >= maxBands}<span class="muted note"
							>{t('eq.maxBands', { n: maxBands })}</span
						>{/if}
				</div>
			{/if}

			<div class="actions">
				<button type="button" class="btn" onclick={importPreset}>{t('eq.import')}</button>
				<button type="button" class="btn" onclick={exportPreset}>{t('eq.export')}</button>
				{#if message}<span class="note">{message}</span>{/if}
				{#if failure}<span class="note fault-text">{t(failure)}</span>{/if}
			</div>
		</section>

		<section>
			<h2 class="label">{t('preamp.title')}</h2>
			<label class="check">
				<input
					type="checkbox"
					checked={dsp.autoPreamp}
					onchange={(event) => updateDsp({ autoPreamp: event.currentTarget.checked })}
				/>
				{t('preamp.auto')}
			</label>
			<label class="field-row">
				<span>{t('preamp.manual')}</span>
				<input
					type="range"
					min="-20"
					max="10"
					step="0.5"
					value={dsp.preampDb}
					disabled={dsp.autoPreamp}
					oninput={(event) => updateDsp({ preampDb: Number(event.currentTarget.value) })}
				/>
				<span class="numeric value">{db(dsp.preampDb)}</span>
			</label>
			<p class="note numeric">{t('preamp.effective', { value: db(effectivePreamp) })}</p>
		</section>

		<section>
			<h2 class="label">{t('limiter.title')}</h2>
			<label class="check">
				<input
					type="checkbox"
					checked={dsp.limiterEnabled}
					onchange={(event) => updateDsp({ limiterEnabled: event.currentTarget.checked })}
				/>
				{t('limiter.enabled')}
			</label>
			<label class="field-row">
				<span>{t('limiter.release')}</span>
				<input
					type="range"
					min="10"
					max="1000"
					step="10"
					value={dsp.limiterReleaseMs}
					oninput={(event) => updateDsp({ limiterReleaseMs: Number(event.currentTarget.value) })}
				/>
				<span class="numeric value">{millis(dsp.limiterReleaseMs)}</span>
			</label>
		</section>
	</div>
{/if}

<style>
	@import './settings.css';

	.spacer {
		flex: 1;
	}

	.faders {
		display: grid;
		grid-template-columns: repeat(10, minmax(0, 1fr));
		gap: 4px;
		min-width: 0;
	}

	@container content (max-width: 520px) {
		.faders {
			gap: 2px;
		}

		.fader input {
			width: 18px;
			height: 120px;
		}

		.fader .label {
			font-size: 9.5px;
		}
	}

	.fader {
		display: grid;
		justify-items: center;
		gap: 6px;
	}

	.fader input {
		writing-mode: vertical-lr;
		direction: rtl;
		width: 22px;
		height: 150px;
	}

	.fader .label {
		font-size: 10.5px;
	}

	.value-small {
		font-size: 10.5px;
		color: var(--onsa-role-adjustable);
	}

	/* A row of numbers this wide cannot be folded without lying about what
	   it says, so below its width it keeps a scroll of its own rather than
	   pushing the whole page sideways. */
	.bands {
		display: grid;
		grid-template-columns: auto minmax(96px, 1.2fr) repeat(3, minmax(64px, 1fr)) auto;
		align-items: center;
		gap: 6px 8px;
		min-width: 460px;
	}

	.bands-scroll {
		min-width: 0;
		overflow-x: auto;
		scrollbar-width: thin;
	}

	.bands .label {
		font-size: 10.5px;
	}

	.bands .field {
		width: 100%;
	}
</style>
