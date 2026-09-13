<!-- Output & Quality (SPEC sections 3.3, 3.4 and 4.1). -->
<script lang="ts">
	import type { BufferChoice, Curve, Quality, RgMode } from '$lib/backend';
	import { db, rate, secondsLabel } from '$lib/format';
	import { t } from '$lib/i18n/index.svelte';
	import type { MessageKey } from '$lib/i18n/dictionary';
	import { player } from '$lib/player.svelte';
	import {
		loadDevices,
		settings,
		updateDsp,
		updateOutput,
		updatePlayback
	} from '$lib/settings.svelte';

	const RATES = [44100, 48000, 88200, 96000, 176400, 192000];
	const qualities: [Quality, MessageKey][] = [
		['fast', 'quality.fast'],
		['balanced', 'quality.balanced'],
		['best', 'quality.best']
	];
	const buffers: [BufferChoice, MessageKey][] = [
		['low', 'buffer.low'],
		['normal', 'buffer.normal'],
		['large', 'buffer.large']
	];
	const curves: [Curve, MessageKey][] = [
		['equalPower', 'curve.equalPower'],
		['linear', 'curve.linear']
	];
	const modes: [RgMode, MessageKey][] = [
		['off', 'rg.off'],
		['track', 'rg.track'],
		['album', 'rg.album'],
		['auto', 'rg.auto']
	];

	$effect(() => {
		void loadDevices();
	});

	const value = $derived(settings.value);
	const output = $derived(player.snapshot?.signal.output ?? null);

	// Slider values follow the pointer; the setting is sent when it is let go.
	let crossfade = $state(0);
	let skip = $state(0.3);
	$effect(() => {
		if (value) {
			crossfade = value.playback.crossfadeSeconds;
			skip = value.playback.skipCrossfadeSeconds;
		}
	});
</script>

{#if value}
	<div class="page">
		<section>
			<h2 class="label">{t('output.title')}</h2>
			<label class="field-row">
				<span>{t('output.device')}</span>
				<select
					class="field"
					value={value.output.deviceId ?? ''}
					onchange={(event) => updateOutput({ deviceId: event.currentTarget.value || null })}
				>
					<option value="">{t('output.systemDefault')}</option>
					{#each settings.devices as device (device.id)}
						<option value={device.id}>{device.name}</option>
					{/each}
				</select>
			</label>
			<label class="field-row">
				<span>{t('output.sampleRate')}</span>
				<select
					class="field"
					disabled={value.output.matchSource}
					value={value.output.sampleRate === null ? '' : String(value.output.sampleRate)}
					onchange={(event) =>
						updateOutput({
							sampleRate: event.currentTarget.value ? Number(event.currentTarget.value) : null
						})}
				>
					<option value="">{t('output.followDevice')}</option>
					{#each RATES as hz (hz)}
						<option value={String(hz)}>{rate(hz)}</option>
					{/each}
				</select>
			</label>
			<label class="check">
				<input
					type="checkbox"
					checked={value.output.matchSource}
					onchange={(event) => updateOutput({ matchSource: event.currentTarget.checked })}
				/>
				{t('output.matchSource')}
			</label>
			<p class="muted note">{t('output.matchSourceHint')}</p>
			<p class="muted note numeric">
				{output ? t('output.current', { name: output.name, rate: rate(output.sampleRate) }) : t('output.none')}
			</p>

			<div class="field-row">
				<span>{t('output.quality')}</span>
				<div class="chips">
					{#each qualities as [quality, label] (quality)}
						<button
							type="button"
							class="chip"
							aria-pressed={value.playback.quality === quality}
							onclick={() => updatePlayback({ quality })}>{t(label)}</button
						>
					{/each}
				</div>
			</div>
			<div class="field-row">
				<span>{t('output.buffer')}</span>
				<div class="chips">
					{#each buffers as [buffer, label] (buffer)}
						<button
							type="button"
							class="chip"
							aria-pressed={value.playback.buffer === buffer}
							onclick={() => updatePlayback({ buffer })}>{t(label)}</button
						>
					{/each}
				</div>
			</div>
			<p class="muted note">{t('output.bufferNote')}</p>
			<label class="check">
				<input
					type="checkbox"
					checked={value.dsp.dither}
					onchange={(event) => updateDsp({ dither: event.currentTarget.checked })}
				/>
				{t('output.dither')}
			</label>
			<label class="check">
				<input
					type="checkbox"
					checked={value.playback.powerSave}
					onchange={(event) => updatePlayback({ powerSave: event.currentTarget.checked })}
				/>
				{t('output.powerSave')}
			</label>
			<p class="muted note">{t('output.powerSaveHint')}</p>
		</section>

		<section>
			<h2 class="label">{t('crossfade.title')}</h2>
			<label class="field-row">
				<span>{t('crossfade.duration')}</span>
				<input
					type="range"
					min="0"
					max="12"
					step="0.5"
					bind:value={crossfade}
					onchange={() => updatePlayback({ crossfadeSeconds: crossfade })}
				/>
				<span class="numeric value">{crossfade === 0 ? t('crossfade.off') : secondsLabel(crossfade)}</span>
			</label>
			<div class="field-row">
				<span>{t('crossfade.curve')}</span>
				<div class="chips">
					{#each curves as [curve, label] (curve)}
						<button
							type="button"
							class="chip"
							aria-pressed={value.playback.curve === curve}
							onclick={() => updatePlayback({ curve })}>{t(label)}</button
						>
					{/each}
				</div>
			</div>
			<label class="check">
				<input
					type="checkbox"
					checked={value.playback.albumGapless}
					onchange={(event) => updatePlayback({ albumGapless: event.currentTarget.checked })}
				/>
				{t('crossfade.albumGapless')}
			</label>
			<label class="field-row">
				<span>{t('crossfade.skip')}</span>
				<input
					type="range"
					min="0"
					max="2"
					step="0.1"
					bind:value={skip}
					onchange={() => updatePlayback({ skipCrossfadeSeconds: skip })}
				/>
				<span class="numeric value">{skip === 0 ? t('crossfade.off') : secondsLabel(skip)}</span>
			</label>
		</section>

		<section>
			<h2 class="label">{t('replaygain.title')}</h2>
			<div class="field-row">
				<span>{t('replaygain.mode')}</span>
				<div class="chips">
					{#each modes as [mode, label] (mode)}
						<button
							type="button"
							class="chip"
							aria-pressed={value.dsp.replaygainMode === mode}
							onclick={() => updateDsp({ replaygainMode: mode })}>{t(label)}</button
						>
					{/each}
				</div>
			</div>
			<label class="field-row">
				<span>{t('replaygain.preamp')}</span>
				<input
					type="range"
					min="-15"
					max="15"
					step="0.5"
					value={value.dsp.replaygainPreampDb}
					oninput={(event) => updateDsp({ replaygainPreampDb: Number(event.currentTarget.value) })}
				/>
				<span class="numeric value">{db(value.dsp.replaygainPreampDb)}</span>
			</label>
			<label class="field-row">
				<span>{t('replaygain.fallback')}</span>
				<input
					type="range"
					min="-15"
					max="15"
					step="0.5"
					value={value.dsp.replaygainFallbackDb}
					oninput={(event) => updateDsp({ replaygainFallbackDb: Number(event.currentTarget.value) })}
				/>
				<span class="numeric value">{db(value.dsp.replaygainFallbackDb)}</span>
			</label>
			<label class="check">
				<input
					type="checkbox"
					checked={value.dsp.replaygainPreventClipping}
					onchange={(event) =>
						updateDsp({ replaygainPreventClipping: event.currentTarget.checked })}
				/>
				{t('replaygain.preventClipping')}
			</label>
		</section>

		{#if settings.failure}<p class="fault-text">{t(settings.failure)}</p>{/if}
	</div>
{/if}

<style>
	@import './settings.css';
</style>
