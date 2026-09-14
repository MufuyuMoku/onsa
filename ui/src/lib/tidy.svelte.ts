/**
 * Tidying metadata, as the interface holds it (SPEC section 8).
 *
 * Nothing here applies anything on its own. Every run is summarised first
 * and the summary is what the apply button sits under; the store keeps the
 * summary and the report side by side so the interface can show what was
 * about to happen and then what did.
 */

import {
	editHistory,
	editUndo,
	failureKey,
	tidyApplyEdits,
	tidyPickFolder,
	tidyPreviewEdits,
	tidyPreviewWrites,
	tidyRenameApply,
	tidyRenamePlan,
	tidyScope,
	tidySetScope,
	tidyTracks,
	tidyWrite,
	type EditBatch,
	type RenamePlan,
	type TidyChange,
	type TidyReport,
	type TidyScope,
	type TidySummary,
	type Track
} from '$lib/backend';
import type { MessageKey } from '$lib/i18n/dictionary';
import { t } from '$lib/i18n/index.svelte';
import { library } from '$lib/library.svelte';

let current = $state<TidyScope | null>(null);
let tracks = $state<Track[]>([]);
let failure = $state<MessageKey | null>(null);
let busy = $state(false);

export const scope = {
	/** The folder a run is held to, once the backend has said. */
	get value() {
		return current;
	},
	/** The tracks inside it. */
	get tracks() {
		return tracks;
	},
	/** The last failure, if any. */
	get failure() {
		return failure;
	},
	/** Whether a run is under way. */
	get busy() {
		return busy;
	}
};

async function guard<T>(work: () => Promise<T>): Promise<T | null> {
	busy = true;
	try {
		const answer = await work();
		failure = null;
		return answer;
	} catch (error) {
		failure = failureKey(error);
		return null;
	} finally {
		busy = false;
	}
}

/** Reads the scope and the tracks inside it. */
export async function loadTidy(): Promise<void> {
	await guard(async () => {
		current = await tidyScope();
		tracks = await tidyTracks();
	});
}

export async function pickScopeFolder(): Promise<void> {
	const picked = await guard(() => tidyPickFolder(t('tidy.pickFolderTitle')));
	if (!picked) return;
	await guard(async () => {
		current = await tidySetScope(picked, current?.limit ?? 50);
		tracks = await tidyTracks();
	});
}

export async function clearScope(): Promise<void> {
	await guard(async () => {
		current = await tidySetScope(null, current?.limit ?? 50);
		tracks = await tidyTracks();
	});
}

export async function setScopeLimit(limit: number): Promise<void> {
	await guard(async () => {
		current = await tidySetScope(current?.folder ?? null, limit);
	});
}

export const previewEdits = (changes: TidyChange[]): Promise<TidySummary | null> =>
	guard(() => tidyPreviewEdits(changes));

export async function applyEdits(
	note: string,
	changes: TidyChange[]
): Promise<TidyReport | null> {
	const report = await guard(() => tidyApplyEdits(note, changes));
	if (report) await refresh();
	return report;
}

export const previewWrites = (ids: number[]): Promise<TidySummary | null> =>
	guard(() => tidyPreviewWrites(ids));

export async function writeToFiles(note: string, ids: number[]): Promise<TidyReport | null> {
	const report = await guard(() => tidyWrite(note, ids));
	if (report) await refresh();
	return report;
}

export const planRename = (
	root: string,
	pattern: string,
	ids: number[]
): Promise<RenamePlan | null> => guard(() => tidyRenamePlan(root, pattern, ids));

export async function applyRename(
	note: string,
	root: string,
	pattern: string,
	ids: number[]
): Promise<TidyReport | null> {
	const report = await guard(() => tidyRenameApply(note, root, pattern, ids));
	if (report) await refresh();
	return report;
}

export const history = (limit = 20): Promise<EditBatch[] | null> => guard(() => editHistory(limit));

export async function undo(batch: number) {
	const report = await guard(() => editUndo(batch));
	if (report) await refresh();
	return report;
}

/** Reads the tracks again after something changed them. */
async function refresh(): Promise<void> {
	try {
		current = await tidyScope();
		tracks = await tidyTracks();
	} catch {
		// A list that cannot be read leaves the last one standing.
	}
	void library.version;
}
