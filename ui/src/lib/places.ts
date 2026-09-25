/**
 * The rule for a remembered list position, kept apart from the state that
 * holds it so it can be read and tested on its own.
 */

import type { SortKey } from '$lib/backend';

/** Where a list was left, and the ordering that position belonged to. */
export interface ListPlace {
	top: number;
	/**
	 * The ordering in force then, or `fixed` for a list with an order of its
	 * own — an album, a playlist — which cannot be re-sorted.
	 */
	sort: SortKey | 'fixed';
	descending: boolean;
}

/** The ordering a list is in right now. */
export interface Ordering {
	/** Whether this list has an order of its own and cannot be re-sorted. */
	fixed: boolean;
	sort: SortKey;
	descending: boolean;
}

/**
 * Whether a position kept earlier still means anything.
 *
 * Row four hundred by title is a different song from row four hundred by
 * year, so a position is only restored under the ordering it was taken in.
 * Anything else is not a position at all, and forcing it would scroll
 * somebody to a song they were not looking at.
 */
export function placeApplies(place: ListPlace | null, now: Ordering): boolean {
	if (!place || !(place.top > 0)) return false;
	if (now.fixed) return place.sort === 'fixed';
	return place.sort === now.sort && place.descending === now.descending;
}
