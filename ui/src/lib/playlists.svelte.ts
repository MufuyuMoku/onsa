/**
 * Playlist state shared by the views (SPEC section 6).
 *
 * The backend owns the playlists; this keeps the list the sidebar and the
 * views read, and reloads it whenever the backend says it changed. Smart
 * playlists are never cached as tracks: they are asked for again every time,
 * so they follow the library.
 */

import {
	EVENTS,
	on,
	playlists as listPlaylists,
	playlistAdd,
	playlistCreate,
	playlistDelete,
	playlistDuplicate,
	playlistFromQueue,
	playlistMove,
	playlistRemove,
	playlistRename,
	playlistSetRules,
	type PlayContext,
	type Playlist,
	type Rules
} from '$lib/backend';

let all = $state<Playlist[]>([]);
/** Changes whenever a playlist's contents changed, so views reload. */
let version = $state(0);

export const playlists = {
	/** Every playlist, newest change first as the backend orders them. */
	get all() {
		return all;
	},
	get version() {
		return version;
	}
};

/** A rule set a new smart playlist starts from: the whole library, by title. */
export function emptyRules(): Rules {
	return { match: 'all', rules: [], sort: { field: 'title', descending: false }, limit: null };
}

async function reload(): Promise<void> {
	try {
		all = await listPlaylists();
		version += 1;
	} catch {
		// A list that cannot be read leaves the last one standing.
	}
}

/** Reads the playlists and follows the backend's events. */
export async function initPlaylists(): Promise<void> {
	await reload();
	await on<null>(EVENTS.playlists, () => void reload());
	// A smart playlist changes when the library does, without anyone
	// touching the playlist itself (SPEC section 6.3).
	await on<null>(EVENTS.libraryChanged, () => void reload());
}

/** One playlist by id, as far as the list knows it. */
export function playlistById(id: number): Playlist | undefined {
	return all.find((playlist) => playlist.id === id);
}

export async function createPlaylist(name: string, rules: Rules | null = null): Promise<number> {
	const id = await playlistCreate(name, rules);
	await reload();
	return id;
}

export async function renamePlaylist(id: number, name: string): Promise<void> {
	await playlistRename(id, name);
	await reload();
}

export async function setRules(id: number, rules: Rules): Promise<void> {
	await playlistSetRules(id, rules);
	await reload();
}

export async function deletePlaylist(id: number): Promise<void> {
	await playlistDelete(id);
	await reload();
}

export async function duplicatePlaylist(id: number, name: string): Promise<number> {
	const copy = await playlistDuplicate(id, name);
	await reload();
	return copy;
}

export async function addToPlaylist(id: number, context: PlayContext): Promise<number> {
	const added = await playlistAdd(id, context);
	await reload();
	return added;
}

export async function removeFromPlaylist(id: number, position: number): Promise<void> {
	await playlistRemove(id, position);
	await reload();
}

export async function moveInPlaylist(id: number, from: number, to: number): Promise<void> {
	await playlistMove(id, from, to);
	await reload();
}

export async function saveQueueAsPlaylist(name: string): Promise<number> {
	const id = await playlistFromQueue(name);
	await reload();
	return id;
}
