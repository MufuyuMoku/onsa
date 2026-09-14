/**
 * The column arrangement of each track list, as the interface holds it.
 *
 * The widths live in the display settings, so they come back when Onsa is
 * opened again. A drag changes many pixels a second; only the width the
 * listener lets go of is written down.
 */

import { emptyPrefs, type ColumnKey, type ColumnPrefs } from '$lib/columns';
import { settings, updateDisplay } from '$lib/settings.svelte';

/** Which list is being arranged. One key per place a track list appears. */
export type ListView =
	| 'tracks'
	| 'album'
	| 'artist'
	| 'genre'
	| 'folder'
	| 'playlist'
	| 'search';

/** The arrangement of one list, defaults included. */
export function prefsFor(view: ListView): ColumnPrefs {
	const stored = settings.value?.display.columns?.[view];
	if (!stored) return emptyPrefs();
	return {
		widths: { ...(stored.widths ?? {}) } as ColumnPrefs['widths'],
		hidden: [...(stored.hidden ?? [])] as ColumnKey[]
	};
}

function store(view: ListView, next: ColumnPrefs): void {
	const columns = { ...(settings.value?.display.columns ?? {}) };
	// A list back at its defaults is forgotten rather than stored as empty.
	if (Object.keys(next.widths).length === 0 && next.hidden.length === 0) delete columns[view];
	else columns[view] = { widths: { ...next.widths }, hidden: [...next.hidden] };
	void updateDisplay({ columns });
}

/** Writes down the width a column was let go at. */
export function setWidth(view: ListView, key: ColumnKey, width: number): void {
	const prefs = prefsFor(view);
	store(view, { ...prefs, widths: { ...prefs.widths, [key]: Math.round(width) } });
}

/** Puts a column on the list or takes it off. */
export function setShown(view: ListView, key: ColumnKey, shown: boolean): void {
	const prefs = prefsFor(view);
	const hidden = prefs.hidden.filter((entry) => entry !== key);
	if (!shown) hidden.push(key);
	store(view, { ...prefs, hidden });
}

/** Forgets everything about this list, so it goes back to its defaults. */
export function resetColumns(view: ListView): void {
	store(view, emptyPrefs());
}
