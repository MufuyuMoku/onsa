/**
 * The columns of a track list: how wide they are, which of them are shown,
 * and which have to give way when the window is narrow (SPEC section 9.2).
 *
 * Two rules meet here. The listener may drag a column to the width they
 * want, and that width is kept per list and remembered; but a window too
 * narrow to hold them all still drops columns in the same order as before,
 * because a squeezed column ends up on top of its neighbour rather than
 * merely small.
 */

import type { SortKey } from '$lib/backend';
import type { MessageKey } from '$lib/i18n/dictionary';

/** A column of the track list. `title` and `duration` never leave. */
export type ColumnKey = SortKey;

export interface Column {
	key: ColumnKey;
	label: MessageKey;
	/** Width in CSS pixels when nobody has said otherwise. */
	width: number;
	/** Smallest width a drag may leave it at. */
	min: number;
	/** Whether the listener may take it off the list. */
	optional: boolean;
	/** Whether a drag may change its width. */
	sizable: boolean;
	/** Lower goes first when the window runs out of room. */
	priority: number;
}

/** The number column, which is never one of the arrangeable ones. */
export const NUMBER_WIDTH = 44;
/** The strip at the right end that holds the column menu. */
export const MENU_WIDTH = 22;
/** The gap between two cells of a row, matching the stylesheet. */
export const GAP = 10;
/** The padding at each end of a row, matching the stylesheet. */
export const EDGE = 14;

/**
 * What a row spends on itself rather than on its columns: the padding at
 * both ends and the gaps between the cells. Fixed column widths do not
 * absorb these the way the old flexible ones did, so they are counted.
 */
export function overhead(shown: number): number {
	const cells = shown + 2; // the number at the front, the menu at the end
	return EDGE * 2 + GAP * Math.max(0, cells - 1);
}

export const COLUMNS: Column[] = [
	{
		key: 'title',
		label: 'column.title',
		width: 260,
		min: 120,
		optional: false,
		sizable: true,
		priority: 99
	},
	{
		key: 'artist',
		label: 'column.artist',
		width: 170,
		min: 90,
		optional: true,
		sizable: true,
		priority: 3
	},
	{
		key: 'album',
		label: 'column.album',
		width: 190,
		min: 90,
		optional: true,
		sizable: true,
		priority: 2
	},
	{
		key: 'year',
		label: 'column.year',
		width: 56,
		min: 44,
		optional: true,
		sizable: true,
		priority: 1
	},
	{
		key: 'duration',
		label: 'column.duration',
		width: 56,
		min: 48,
		optional: false,
		sizable: false,
		priority: 99
	}
];

export const column = (key: ColumnKey): Column =>
	COLUMNS.find((entry) => entry.key === key) ?? COLUMNS[0]!;

/** What one list remembers: the widths that were dragged, and what is off. */
export interface ColumnPrefs {
	widths: Partial<Record<ColumnKey, number>>;
	hidden: ColumnKey[];
}

export const emptyPrefs = (): ColumnPrefs => ({ widths: {}, hidden: [] });

/** The width to draw a column at: what was dragged, or the default. */
export function widthOf(key: ColumnKey, prefs: ColumnPrefs): number {
	const wanted = prefs.widths[key];
	const fallback = column(key);
	if (typeof wanted !== 'number' || !Number.isFinite(wanted)) return fallback.width;
	return Math.max(fallback.min, Math.round(wanted));
}

/**
 * Which columns actually get drawn, in order.
 *
 * Everything the listener kept is offered first; then, while the row is
 * wider than the space there is, the lowest-priority column steps out. The
 * title and the length always stay, and the title is allowed to shrink to
 * its own minimum before anything else is dropped.
 */
export function shownColumns(
	available: number,
	prefs: ColumnPrefs,
	allowed: (key: ColumnKey) => boolean
): ColumnKey[] {
	const kept = COLUMNS.filter(
		(entry) => allowed(entry.key) && !(entry.optional && prefs.hidden.includes(entry.key))
	);
	const shown = kept.map((entry) => entry.key);
	// A column is only dropped once even a title squeezed to its own
	// minimum would not fit: the title gives way before a column leaves.
	const room = () =>
		NUMBER_WIDTH +
		MENU_WIDTH +
		overhead(shown.length) +
		shown.reduce(
			(total, key) => total + (key === 'title' ? column('title').min : widthOf(key, prefs)),
			0
		);
	while (room() > available) {
		let weakest: ColumnKey | null = null;
		for (const key of shown) {
			const entry = column(key);
			if (!entry.optional) continue;
			if (weakest === null || entry.priority < column(weakest).priority) weakest = key;
		}
		if (weakest === null) break;
		shown.splice(shown.indexOf(weakest), 1);
	}
	return shown;
}

/**
 * How wide the title column starts out, given what is left after the others.
 *
 * The title takes the space nobody else wants, so its width is a floor
 * rather than a size. That floor is what the listener dragged it to, unless
 * there is less room than that, in which case it gives way — down to its own
 * minimum and no further.
 */
export function titleFloor(available: number, shown: ColumnKey[], prefs: ColumnPrefs): number {
	const others = shown
		.filter((key) => key !== 'title')
		.reduce((total, key) => total + widthOf(key, prefs), 0);
	const left = available - NUMBER_WIDTH - MENU_WIDTH - overhead(shown.length) - others;
	return Math.max(column('title').min, Math.min(widthOf('title', prefs), Math.round(left)));
}

/**
 * The CSS grid the head and every row share.
 *
 * The title takes what is left over, so a wide window is not left with a gap
 * on the right; the width the listener dragged it to is its smallest, as far
 * as the room allows.
 */
export function template(available: number, shown: ColumnKey[], prefs: ColumnPrefs): string {
	const floor = titleFloor(available, shown, prefs);
	const parts = [`${NUMBER_WIDTH}px`];
	for (const key of shown) {
		parts.push(key === 'title' ? `minmax(${floor}px, 1fr)` : `${widthOf(key, prefs)}px`);
	}
	parts.push(`${MENU_WIDTH}px`);
	return parts.join(' ');
}
