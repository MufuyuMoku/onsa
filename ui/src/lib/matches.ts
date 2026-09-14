/**
 * What a list of suggestions and a set of ticks add up to (SPEC §8).
 *
 * Kept apart from the store that holds them so it can be tested on its own.
 * This is the piece that decides what actually gets applied, and "nothing was
 * ticked, so nothing happens" is exactly the property worth a test rather
 * than a careful reading.
 */

import type { CoverPick, TidyChange, TrackMatch } from '$lib/backend';

/** Which candidate is in use per track. Absent means "as it arrived". */
export type Picks = Record<number, number | null>;
/** Which fields are ticked, by track, candidate and field. */
export type Ticks = Record<string, boolean>;
/** Which tracks' covers are ticked. */
export type Covers = Record<number, boolean>;

/** The key one field of one candidate of one track is ticked under. */
export function tickKey(trackId: number, candidate: number, field: string): string {
	return `${trackId}:${candidate}:${field}`;
}

/** Which candidate a track is going by: the listener's choice, or its own. */
export function pickedOf(track: TrackMatch, picks: Picks): number | null {
	const chosen = picks[track.trackId];
	return chosen === undefined ? track.picked : chosen;
}

/** Whether one field is ticked: the listener's tick, or the one it arrived with. */
export function tickedOf(
	track: TrackMatch,
	candidate: number,
	field: string,
	ticks: Ticks
): boolean {
	const own = ticks[tickKey(track.trackId, candidate, field)];
	if (own !== undefined) return own;
	return track.candidates[candidate]?.fields.find((one) => one.field === field)?.chosen ?? false;
}

/**
 * The changes the ticks add up to.
 *
 * Only the candidate in use counts, and only its ticked fields. A track
 * nobody has settled contributes nothing, however much was found for it.
 */
export function changesFrom(found: TrackMatch[], picks: Picks, ticks: Ticks): TidyChange[] {
	const changes: TidyChange[] = [];
	for (const track of found) {
		const candidate = pickedOf(track, picks);
		if (candidate === null) continue;
		const proposal = track.candidates[candidate];
		if (!proposal) continue;
		for (const field of proposal.fields) {
			if (!tickedOf(track, candidate, field.field, ticks)) continue;
			changes.push({ trackId: track.trackId, field: field.field, value: field.suggested });
		}
	}
	return changes;
}

/** The covers the ticks add up to. */
export function coversFrom(found: TrackMatch[], picks: Picks, covers: Covers): CoverPick[] {
	const chosen: CoverPick[] = [];
	for (const track of found) {
		if (!covers[track.trackId]) continue;
		const candidate = pickedOf(track, picks);
		if (candidate === null) continue;
		const key = track.candidates[candidate]?.cover?.key;
		if (key) chosen.push({ trackId: track.trackId, key });
	}
	return chosen;
}

/**
 * Every tick set at once, for the button that takes all of them.
 *
 * A field that suggests what the track already says is left alone even here:
 * applying it would be a step in the record with nothing behind it.
 */
export function everyTick(found: TrackMatch[], picks: Picks, on: boolean): Ticks {
	const ticks: Ticks = {};
	for (const track of found) {
		const candidate = pickedOf(track, picks);
		if (candidate === null) continue;
		for (const field of track.candidates[candidate]?.fields ?? []) {
			ticks[tickKey(track.trackId, candidate, field.field)] =
				on && field.current !== field.suggested;
		}
	}
	return ticks;
}
