/**
 * The words of whatever is playing (SPEC section 10).
 *
 * The backend answers at once with whatever it has to hand, and says
 * whether it has gone off to ask a service as well. When that answer
 * arrives it sends an event, and this asks again. Nothing here waits for
 * the network, so a service that is slow or gone changes nothing about the
 * window except that the words turn up later or not at all.
 */

import {
	EVENTS,
	lyricsFor,
	lyricsLookAgain,
	lyricsOffset,
	lyricsState,
	lyricsWriteBeside,
	on,
	type Lyrics
} from '$lib/backend';
import { player } from '$lib/player.svelte';

export { lineAt } from '$lib/lyrics';

/** How far the ± buttons move the words, in milliseconds. */
export const NUDGE_MS = 250;

let words = $state<Lyrics | null>(null);
let busy = $state(false);
/**
 * Which read is the newest.
 *
 * Two reads can be in the air at once — a track change and an event, say —
 * and the slower one must not land on top of the newer one.
 */
let newest = 0;

export const lyrics = {
	/** The words for the track playing, or null when there are none yet. */
	get words() {
		return words;
	},
	/** Whether the listener asked for something and it has not answered. */
	get busy() {
		return busy;
	}
};

async function read(trackId: number, fresh: boolean): Promise<void> {
	const mine = ++newest;
	try {
		const answer = fresh ? await lyricsFor(trackId) : await lyricsState(trackId);
		if (mine !== newest) return;
		words = answer;
	} catch {
		if (mine !== newest) return;
		words = null;
	}
}

/** Follows the playing track. Call once, from the root. */
export function followLyrics(): void {
	$effect(() => {
		const id = player.snapshot?.track?.id ?? null;
		if (id === null) {
			newest += 1;
			words = null;
			return;
		}
		void read(id, true);
	});

	$effect(() => {
		let stop: (() => void) | undefined;
		let gone = false;
		on<number>(EVENTS.lyrics, (trackId) => {
			// A lookup for a track that has since been left behind is of no
			// use to the window; it is still kept in the library.
			if (trackId === (player.snapshot?.track?.id ?? null)) void read(trackId, false);
		}).then((off) => {
			if (gone) off();
			else stop = off;
		});
		return () => {
			gone = true;
			stop?.();
		};
	});
}

/** Moves the words against the song, and remembers it. */
export async function nudge(byMs: number): Promise<void> {
	const current = words;
	if (!current) return;
	try {
		words = await lyricsOffset(current.trackId, current.offsetMs + byMs);
	} catch {
		// An offset that cannot be stored leaves the words where they are.
	}
}

/** Puts the words back where the file has them. */
export async function resetOffset(): Promise<void> {
	const current = words;
	if (!current || current.offsetMs === 0) return;
	try {
		words = await lyricsOffset(current.trackId, 0);
	} catch {
		// As above.
	}
}

/** Asks the service again, forgetting what it said last time. */
export async function lookAgain(): Promise<void> {
	const current = words;
	if (!current) return;
	busy = true;
	try {
		words = await lyricsLookAgain(current.trackId);
	} catch {
		// Refused because the internet is off, most likely; the panel
		// already says so.
	} finally {
		busy = false;
	}
}

/** Writes the words beside the song as a `.lrc`. */
export async function saveBeside(): Promise<void> {
	const current = words;
	if (!current) return;
	busy = true;
	try {
		words = await lyricsWriteBeside(current.trackId);
	} catch {
		// A folder that will not take the file leaves the words as they are.
	} finally {
		busy = false;
	}
}
