/**
 * Working out which line of a song is being sung (SPEC section 10).
 *
 * Kept apart from the store so it can be tested on its own, without a
 * window or a player.
 */

import type { Lyrics } from '$lib/backend';

/**
 * Which line belongs to a moment, counting the offset.
 *
 * The same rule the backend keeps: a positive offset holds the words back.
 * Before the first line there is no answer, which is how a long
 * introduction stays unlit rather than lighting the first line early.
 */
export function lineAt(current: Lyrics | null, positionSeconds: number): number {
	if (!current) return -1;
	const want = positionSeconds * 1000 - current.offsetMs;
	let found = -1;
	for (let index = 0; index < current.lines.length; index += 1) {
		const at = current.lines[index]?.atMs;
		if (at === null || at === undefined) continue;
		if (at <= want) found = index;
		else break;
	}
	return found;
}
