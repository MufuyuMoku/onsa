/**
 * Library state shared by the views: scan progress, a version that changes
 * whenever lists should reload, sorting and the search text.
 */

import { EVENTS, on, scanStatus, type ScanStatus, type SortKey } from '$lib/backend';
import { t } from '$lib/i18n/index.svelte';

/** While scanning, lists reload at most this often. */
const RELOAD_WHILE_SCANNING_MS = 1500;

let scan = $state<ScanStatus>({
	running: false,
	seen: 0,
	read: 0,
	unchanged: 0,
	failed: 0,
	missing: 0,
	elapsedMs: 0
});
let version = $state(0);
let sort = $state<SortKey>('artist');
let descending = $state(false);
let query = $state('');
let lastReload = 0;

export const library = {
	get scan() {
		return scan;
	},
	/** Changes whenever the library contents changed. */
	get version() {
		return version;
	},
	get sort() {
		return sort;
	},
	get descending() {
		return descending;
	},
	get query() {
		return query;
	}
};

function reload(): void {
	lastReload = Date.now();
	version += 1;
}

/** Reads the scan state and follows the library events. */
export async function initLibrary(): Promise<void> {
	scan = await scanStatus();
	await on<ScanStatus>(EVENTS.scan, (next) => {
		scan = next;
		// New tracks show up during a long first scan, not only at its end.
		if (next.running && Date.now() - lastReload > RELOAD_WHILE_SCANNING_MS) reload();
	});
	await on<null>(EVENTS.libraryChanged, reload);
}

/** Sorts by a column; the same column again flips the direction. */
export function sortBy(key: SortKey): void {
	if (sort === key) {
		descending = !descending;
	} else {
		sort = key;
		descending = false;
	}
}

/** Sets the search text. */
export function setQuery(next: string): void {
	query = next;
}

/** The scan in one line: running, finished, or not run yet. */
export function scanSummary(): string {
	if (scan.running) return t('scan.running', { seen: scan.seen, read: scan.read });
	if (scan.seen === 0 && scan.elapsedMs === 0) return t('scan.idle');
	const done = t('scan.done', {
		seen: scan.seen,
		// A rescan of an unchanged library takes milliseconds; show that.
		seconds: (scan.elapsedMs / 1000).toFixed(scan.elapsedMs < 1000 ? 2 : 1)
	});
	return scan.failed > 0 ? `${done} · ${t('scan.failed', { n: scan.failed })}` : done;
}
