/**
 * Output, playback and DSP settings as the interface edits them.
 *
 * A slider sends many changes a second. DSP changes are coalesced: at most
 * one command is in flight, and the newest value always wins, so the engine
 * follows the slider without a queue of stale values building up.
 */

import {
	failureKey,
	outputDevices,
	setDsp,
	setOutput,
	setPlayback,
	settingsGet,
	type Device,
	type DspPrefs,
	type OutputPrefs,
	type PlaybackPrefs,
	type Settings
} from '$lib/backend';
import type { MessageKey } from '$lib/i18n/dictionary';

let current = $state<Settings | null>(null);
let devices = $state<Device[]>([]);
let failure = $state<MessageKey | null>(null);

let dspBusy = false;
let dspAgain = false;

export const settings = {
	/** The settings, once loaded. */
	get value() {
		return current;
	},
	get devices() {
		return devices;
	},
	/** The last failure, if any. */
	get failure() {
		return failure;
	}
};

/** Reads every setting group. */
export async function loadSettings(): Promise<void> {
	current = await settingsGet();
}

/** Lists the output devices. */
export async function loadDevices(): Promise<void> {
	try {
		devices = await outputDevices();
	} catch (error) {
		failure = failureKey(error);
	}
}

/** Changes part of the DSP chain; applied at once, stored by the backend. */
export function updateDsp(patch: Partial<DspPrefs>): void {
	if (!current) return;
	current.dsp = { ...current.dsp, ...patch };
	void sendDsp();
}

async function sendDsp(): Promise<void> {
	if (dspBusy) {
		dspAgain = true;
		return;
	}
	dspBusy = true;
	try {
		do {
			dspAgain = false;
			if (current) await setDsp($state.snapshot(current.dsp));
		} while (dspAgain);
		failure = null;
	} catch (error) {
		failure = failureKey(error);
	} finally {
		dspBusy = false;
	}
}

/** Changes part of the playback settings. */
export async function updatePlayback(patch: Partial<PlaybackPrefs>): Promise<void> {
	if (!current) return;
	current.playback = { ...current.playback, ...patch };
	try {
		await setPlayback($state.snapshot(current.playback));
		failure = null;
	} catch (error) {
		failure = failureKey(error);
	}
}

/** Changes the output device or rate. */
export async function updateOutput(patch: Partial<OutputPrefs>): Promise<void> {
	if (!current) return;
	current.output = { ...current.output, ...patch };
	try {
		await setOutput($state.snapshot(current.output));
		failure = null;
	} catch (error) {
		failure = failureKey(error);
	}
}
