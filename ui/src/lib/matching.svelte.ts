/**
 * Looking tracks up on the internet, as the interface holds it (SPEC §8).
 *
 * The store keeps what was found and what the listener has ticked. It applies
 * nothing: what leaves here is an ordinary list of changes, handed to the same
 * preview and the same apply as a value typed by hand, so a suggestion from a
 * service goes through the folder, the summary and the undo like everything
 * else.
 *
 * Results arrive one track at a time while the run goes, and the whole state
 * can be read back at any moment — after leaving the page and coming back, or
 * after the run was stopped. Nothing found is ever thrown away by the
 * interface.
 */

import {
	EVENTS,
	failureKey,
	matchStart,
	matchState,
	matchStop,
	on,
	tidyApplyCovers,
	tidyPreviewCovers,
	type CoverPick,
	type MatchState,
	type TidyChange,
	type TidyReport,
	type TidySummary,
	type TrackMatch
} from '$lib/backend';
import type { MessageKey } from '$lib/i18n/dictionary';
import {
	changesFrom,
	coversFrom,
	everyTick,
	pickedOf,
	tickedOf,
	tickKey,
	type Covers,
	type Picks,
	type Ticks
} from '$lib/matches';

/** What arrives while a run is going: the state, and a track if one is done. */
interface Tick {
	running: boolean;
	done: number;
	total: number;
	trouble: string | null;
	track: TrackMatch | null;
}

const EMPTY: MatchState = {
	running: false,
	done: 0,
	total: 0,
	trouble: null,
	found: [],
	online: false,
	key: false,
	fingerprinter: false
};

let state = $state<MatchState>(EMPTY);
let failure = $state<MessageKey | null>(null);
let stop: (() => void) | null = null;

/**
 * Which candidate is in use per track, and which of its fields and its cover
 * are ticked. Kept beside the results rather than inside them, so a result
 * that arrives again does not undo a choice already made.
 */
let picks = $state<Picks>({});
let ticks = $state<Ticks>({});
let covers = $state<Covers>({});

export const matching = {
	/** How the run stands, and everything found so far. */
	get state() {
		return state;
	},
	get found() {
		return state.found;
	},
	get running() {
		return state.running;
	},
	get failure() {
		return failure;
	},
	/** Whether a run could even be started. */
	get ready() {
		return state.online && state.key && state.fingerprinter;
	},
	/** Which candidate a track is going by. */
	picked(track: TrackMatch): number | null {
		return pickedOf(track, picks);
	},
	/** Whether one field of one candidate is ticked. */
	ticked(track: TrackMatch, candidate: number, field: string): boolean {
		return tickedOf(track, candidate, field, ticks);
	},
	/** Whether a track's cover is ticked. Never ticked on its own. */
	coverTicked(track: TrackMatch): boolean {
		return covers[track.trackId] ?? false;
	}
};

/** Goes by this candidate, or by none at all. */
export function pick(track: TrackMatch, candidate: number | null): void {
	picks = { ...picks, [track.trackId]: candidate };
}

export function tick(track: TrackMatch, candidate: number, field: string, on: boolean): void {
	ticks = { ...ticks, [tickKey(track.trackId, candidate, field)]: on };
}

export function tickCover(track: TrackMatch, on: boolean): void {
	covers = { ...covers, [track.trackId]: on };
}

/** Ticks or unticks everything that is ticked by default across the list. */
export function tickAll(on: boolean): void {
	ticks = everyTick(state.found, picks, on);
}

/** The changes the ticks add up to, in the shape the Tidy page applies. */
export function chosenChanges(): TidyChange[] {
	return changesFrom(state.found, picks, ticks);
}

/** The covers the ticks add up to. */
export function chosenCovers(): CoverPick[] {
	return coversFrom(state.found, picks, covers);
}

/** Reads the state back, and starts following a run that is going. */
export async function loadMatching(): Promise<void> {
	try {
		state = await matchState();
		failure = null;
	} catch (error) {
		failure = failureKey(error);
	}
	if (stop) return;
	const unlisten = await on<Tick>(EVENTS.matching, (tick) => {
		const found = tick.track ? merge(state.found, tick.track) : state.found;
		state = {
			...state,
			running: tick.running,
			done: tick.done,
			total: tick.total,
			trouble: tick.trouble,
			found
		};
	});
	stop = unlisten;
}

/** Puts a track's result in place, replacing an earlier one for that track. */
function merge(found: TrackMatch[], track: TrackMatch): TrackMatch[] {
	const at = found.findIndex((one) => one.trackId === track.trackId);
	if (at < 0) return [...found, track];
	const next = found.slice();
	next[at] = track;
	return next;
}

/** Stops following a run. What was found stays in the backend. */
export function unfollowMatching(): void {
	stop?.();
	stop = null;
}

/** Starts a run over these tracks. Comes back as soon as it has begun. */
export async function startMatching(tracks: number[]): Promise<void> {
	picks = {};
	ticks = {};
	covers = {};
	try {
		state = await matchStart(tracks);
		failure = null;
	} catch (error) {
		failure = failureKey(error);
	}
}

/** Asks the run to stop. What it has found stays. */
export async function stopMatching(): Promise<void> {
	try {
		state = await matchStop();
		failure = null;
	} catch (error) {
		failure = failureKey(error);
	}
}

export async function previewCovers(): Promise<TidySummary | null> {
	const picked = chosenCovers();
	if (picked.length === 0) return null;
	try {
		return await tidyPreviewCovers(picked.map((one) => one.trackId));
	} catch (error) {
		failure = failureKey(error);
		return null;
	}
}

export async function applyCovers(note: string): Promise<TidyReport | null> {
	const picked = chosenCovers();
	if (picked.length === 0) return null;
	try {
		return await tidyApplyCovers(note, picked);
	} catch (error) {
		failure = failureKey(error);
		return null;
	}
}
