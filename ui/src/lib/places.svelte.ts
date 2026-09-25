/**
 * Where each list was left standing, and asking one to show what is playing.
 *
 * A track list is paged: the interface never holds the whole library, and
 * the rows a listener was looking at are thrown away the moment they leave
 * the page. Coming back to a list that has forgotten where it was is the
 * same as being sent to the top of ten thousand songs.
 *
 * Two layers, on purpose. The position is kept here the instant it changes,
 * so leaving a page and coming back is exact and costs nothing; it is
 * written into the display settings a moment later, so it is still there
 * after Onsa is closed and opened again. Writing every pixel of every
 * scroll to the database would be a write a frame.
 */

import type { ListView } from '$lib/columns.svelte';
import type { ListPlace } from '$lib/places';
import { settings, updateDisplay } from '$lib/settings.svelte';

export { placeApplies, type ListPlace, type Ordering } from '$lib/places';

/** How long after the scrolling stops before it is written down. */
const SETTLE = 700;

const live: Record<string, ListPlace> = $state({});
let pending: ReturnType<typeof setTimeout> | null = null;

/** Where this list was left, from this session or from the last one. */
export function placeFor(view: ListView): ListPlace | null {
	const here = live[view];
	if (here) return here;
	const stored = settings.value?.display.places?.[view];
	return stored ? { ...stored } : null;
}

/**
 * Remembers where a list is now.
 *
 * Called as often as a scroll fires, so the settings write waits until the
 * scrolling has stopped; the in-memory answer is already correct.
 */
export function rememberPlace(view: ListView, place: ListPlace): void {
	live[view] = place;
	if (pending) clearTimeout(pending);
	pending = setTimeout(() => {
		pending = null;
		const places = { ...(settings.value?.display.places ?? {}) };
		// A list back at the top has nothing worth remembering.
		if (place.top <= 0) delete places[view];
		else places[view] = { ...place };
		void updateDisplay({ places });
	}, SETTLE);
}

/** Forgets where a list was, for a position that no longer means anything. */
export function forgetPlace(view: ListView): void {
	delete live[view];
	const places = { ...(settings.value?.display.places ?? {}) };
	if (view in places) {
		delete places[view];
		void updateDisplay({ places });
	}
}

/**
 * The count of times somebody has asked to be shown the playing track.
 *
 * A request rather than a call: the control and the shortcut are in the
 * window, and the list that can scroll is somewhere inside it. The list
 * watches this number and answers when it changes.
 */
let asked = $state(0);

export function showPlaying(): void {
	asked += 1;
}

export function askedToShowPlaying(): number {
	return asked;
}
