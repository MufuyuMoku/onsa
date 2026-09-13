/**
 * What the interface has on screen that needs the analysis tap, and the tone
 * colour that follows the sound (SPEC sections 4.4 and 9.5).
 *
 * The tap, the FFT behind the spectrum, and the frames themselves all cost
 * nothing while nothing is showing them, so every meter and visualizer says
 * so while it is on screen and takes it back when it leaves. Components call
 * `watch` from an effect; returning its result lets Svelte undo it.
 */

import { watchAnalysis, type Watching } from '$lib/backend';
import { player } from '$lib/player.svelte';
import { settings } from '$lib/settings.svelte';
import { activeTheme } from '$lib/theme/index.svelte';
import type { ToneTarget } from '$lib/theme/types';

/** How many of each kind are on screen. */
const showing: Record<keyof Watching, number> = { spectrum: 0, meters: 0 };
let sent: Watching | null = null;
let pending = false;

/**
 * Says that one of these is on screen, and gives back the undo. Several can
 * show at once, so they are counted rather than switched.
 */
export function watch(kind: keyof Watching): () => void {
	showing[kind] += 1;
	schedule();
	return () => {
		showing[kind] = Math.max(0, showing[kind] - 1);
		schedule();
	};
}

/** Sends the state once the components of this frame have settled. */
function schedule(): void {
	if (pending) return;
	pending = true;
	queueMicrotask(() => {
		pending = false;
		const now: Watching = { spectrum: showing.spectrum > 0, meters: showing.meters > 0 };
		if (sent && sent.spectrum === now.spectrum && sent.meters === now.meters) return;
		sent = now;
		watchAnalysis(now).catch(() => {
			// The backend is the one that acts on this; a failed report only
			// means the analysis keeps doing what it did.
			sent = null;
		});
	});
}

/** Centre of the shift: sounds around here keep the theme's own hue. */
const MID_HZ = 1200;
/** The span the centroid is read over, in octaves either side of the middle. */
const OCTAVES = 2.5;
/** Weight of each new frame in the running average (about a second at 60 fps). */
const SMOOTHING = 0.02;

/** The running average, kept out of the reactive graph: an effect that read
 * what it writes would keep waking itself. */
let average = 0;
/** What the components read. */
let smoothed = $state(0);

/**
 * Follows the spectral centroid with a slow average, so the colour drifts
 * rather than flickers (SPEC section 9.5). Call once, from the root.
 */
export function followTone(): void {
	$effect(() => {
		const centroid = player.meter.centroidHz;
		if (centroid <= 0) return;
		average = average === 0 ? centroid : average + (centroid - average) * SMOOTHING;
		smoothed = average;
	});
}

/** Whether the listener and the theme both want tone colour. */
export function toneColorOn(): boolean {
	const theme = activeTheme();
	const prefs = settings.value;
	if (!theme?.toneColor.enabled) return false;
	if (prefs && (!prefs.display.toneColor || prefs.playback.powerSave)) return false;
	return true;
}

/**
 * The filter for one tinted element: brighter sound turns the hue up towards
 * the cool end, heavier sound down towards the warm end.
 */
export function toneFilter(target: ToneTarget): string {
	const theme = activeTheme();
	if (!theme || !toneColorOn()) return 'none';
	if (!theme.toneColor.targets.includes(target)) return 'none';
	if (smoothed <= 0) return 'none';
	const octaves = Math.log2(smoothed / MID_HZ);
	const shift = Math.max(-1, Math.min(1, octaves / OCTAVES)) * (theme.toneColor.range / 2);
	return `hue-rotate(${shift.toFixed(1)}deg)`;
}
