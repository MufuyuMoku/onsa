/**
 * The commands and events the interface uses (SPEC section 2).
 *
 * The interface is a remote control: everything it knows comes from here, and
 * every failure arrives as a stable code that the dictionaries translate.
 * The shapes mirror `src-tauri/src/dto.rs`, `player.rs` and `settings.rs`.
 */

import { invoke, isTauri } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { MessageKey } from '$lib/i18n/dictionary';
import type { Theme } from '$lib/theme/types';

/** Name, version and starting theme of the running application. */
export interface AppInfo {
	name: string;
	version: string;
	defaultThemeId: string;
}

/** What the window restores at start. */
export interface AppState {
	themeId: string | null;
	locale: string | null;
	hasFolders: boolean;
	logDebug: boolean;
	logDir: string;
	graphicFreqs: number[];
	maxBands: number;
}

export interface Track {
	id: number;
	path: string;
	title: string | null;
	artist: string | null;
	album: string | null;
	albumArtist: string | null;
	trackNumber: number | null;
	discNumber: number | null;
	year: number | null;
	genre: string | null;
	durationMs: number | null;
	albumId: number | null;
	coverId: number | null;
	status: 'ok' | 'missing' | 'failed';
}

export interface Album {
	id: number;
	title: string;
	albumArtist: string;
	year: number | null;
	coverId: number | null;
	trackCount: number;
}

export interface NameCount {
	name: string;
	trackCount: number;
}

export interface SearchResults {
	tracks: Track[];
	albums: Album[];
	artists: NameCount[];
}

export type SortKey = 'title' | 'artist' | 'album' | 'year' | 'duration' | 'dateAdded';

export type PlayContext =
	| { kind: 'library'; sort: SortKey; descending: boolean; index: number }
	| { kind: 'album'; albumId: number; index: number }
	| { kind: 'tracks'; ids: number[]; index: number };

export interface ScanStatus {
	running: boolean;
	seen: number;
	read: number;
	unchanged: number;
	failed: number;
	missing: number;
	elapsedMs: number;
}

export type RgMode = 'off' | 'track' | 'album' | 'auto';
export type EqKind = 'graphic' | 'parametric';
export type FilterType = 'peaking' | 'lowShelf' | 'highShelf' | 'lowPass' | 'highPass' | 'notch';
export type Quality = 'fast' | 'balanced' | 'best';
export type BufferChoice = 'low' | 'normal' | 'large';
export type Curve = 'equalPower' | 'linear';

export interface SignalPath {
	source: { codec: string | null; sampleRate: number; bitDepth: number | null; channels: number } | null;
	resample: { from: number; to: number } | null;
	replaygain: { mode: RgMode; album: boolean; gainDb: number | null };
	eq: { enabled: boolean; kind: EqKind; bands: number; preampDb: number; autoPreamp: boolean };
	limiter: { enabled: boolean };
	output: { name: string; backend: string; sampleRate: number; channels: number } | null;
}

export interface PlayerSnapshot {
	state: 'stopped' | 'playing' | 'paused';
	current: number | null;
	queueLength: number;
	track: Track | null;
	position: number;
	duration: number | null;
	volumeDb: number;
	signal: SignalPath;
	outputError: boolean;
	failedTrack: string | null;
}

export interface Position {
	index: number;
	seconds: number;
}

export interface Meter {
	peakDb: [number, number];
	clip: boolean;
	limiting: boolean;
}

export interface Queue {
	items: Track[];
	current: number | null;
}

export interface OutputPrefs {
	deviceId: string | null;
	sampleRate: number | null;
}

export interface PlaybackPrefs {
	crossfadeSeconds: number;
	curve: Curve;
	skipCrossfadeSeconds: number;
	albumGapless: boolean;
	quality: Quality;
	buffer: BufferChoice;
}

export interface Band {
	kind: FilterType;
	freq: number;
	gainDb: number;
	q: number;
	enabled: boolean;
}

export interface DspPrefs {
	replaygainMode: RgMode;
	replaygainPreampDb: number;
	replaygainFallbackDb: number;
	replaygainPreventClipping: boolean;
	eqEnabled: boolean;
	eqKind: EqKind;
	graphicGains: number[];
	parametric: Band[];
	preampDb: number;
	autoPreamp: boolean;
	limiterEnabled: boolean;
	limiterReleaseMs: number;
	volumeDb: number;
	dither: boolean;
}

export interface Settings {
	output: OutputPrefs;
	playback: PlaybackPrefs;
	dsp: DspPrefs;
}

export interface Device {
	id: string;
	name: string;
	isDefault: boolean;
	sampleRate: number;
	channels: number;
}

export interface EqCurve {
	points: [number, number][];
	autoPreampDb: number;
}

export interface AutoEqImport {
	bands: Band[];
	preampDb: number | null;
	skipped: number;
}

/** A failure the backend reported, already turned into a dictionary key. */
export class BackendError extends Error {
	readonly messageKey: MessageKey;

	constructor(messageKey: MessageKey, cause?: unknown) {
		super(messageKey, { cause });
		this.name = 'BackendError';
		this.messageKey = messageKey;
	}
}

/** Whether the interface is running inside the Onsa window. */
export function hasBackend(): boolean {
	return isTauri();
}

/** The error codes the backend can send, mapped to dictionary keys. */
const ERROR_KEYS: Record<string, MessageKey> = {
	theme_not_found: 'error.theme_not_found',
	library: 'error.library',
	engine: 'error.engine',
	no_output: 'error.no_output',
	dialog: 'error.dialog',
	io: 'error.io',
	auto_eq_empty: 'error.auto_eq_empty'
};

async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	if (!isTauri()) throw new BackendError('error.backend');
	try {
		return await invoke<T>(command, args);
	} catch (cause) {
		throw new BackendError(errorKey(cause), cause);
	}
}

function errorKey(cause: unknown): MessageKey {
	if (typeof cause === 'object' && cause !== null && 'code' in cause) {
		const code = (cause as { code: unknown }).code;
		if (typeof code === 'string' && code in ERROR_KEYS) return ERROR_KEYS[code]!;
	}
	return 'error.backend';
}

/** Turns anything thrown by a call into a dictionary key. */
export function failureKey(error: unknown): MessageKey {
	return error instanceof BackendError ? error.messageKey : 'error.backend';
}

// Application --------------------------------------------------------------

export const appInfo = () => call<AppInfo>('app_info');
export const appState = () => call<AppState>('app_state');
export const themeList = () => call<Theme[]>('theme_list');
export const themeGet = (id: string) => call<Theme>('theme_get', { id });
export const setTheme = (id: string) => call<void>('set_theme', { id });
export const setLocaleSetting = (locale: string) => call<void>('set_locale', { locale });
export const logOpenFolder = () => call<void>('log_open_folder');
export const logSetDebug = (enabled: boolean) => call<void>('log_set_debug', { enabled });

// Library ------------------------------------------------------------------

export const pickFolder = (title: string) => call<string | null>('pick_folder', { title });
export const addFolder = (path: string) => call<void>('library_add_folder', { path });
export const rescan = () => call<void>('library_rescan');
export const scanStatus = () => call<ScanStatus>('library_scan_status');
export const libraryFolders = () => call<NameCount[]>('library_folders');
export const trackCount = () => call<number>('library_track_count');
export const tracksPage = (sort: SortKey, descending: boolean, offset: number, limit: number) =>
	call<Track[]>('library_tracks', { sort, descending, offset, limit });
export const albumCount = () => call<number>('library_album_count');
export const albumsPage = (offset: number, limit: number) =>
	call<Album[]>('library_albums', { offset, limit });
export const albumTracks = (albumId: number) => call<Track[]>('library_album_tracks', { albumId });
export const search = (query: string, limit: number) =>
	call<SearchResults>('library_search', { query, limit });

// Player -------------------------------------------------------------------

export const play = (context: PlayContext) => call<void>('player_play', { context });
export const togglePlay = () => call<void>('player_toggle');
export const nextTrack = () => call<void>('player_next');
export const previousTrack = () => call<void>('player_previous');
export const seek = (seconds: number) => call<void>('player_seek', { seconds });
export const jump = (index: number) => call<void>('player_jump', { index });
export const playerSnapshot = () => call<PlayerSnapshot>('player_snapshot');
export const playerQueue = () => call<Queue>('player_queue');
export const setVolume = (db: number) => call<void>('player_set_volume', { db });

// Settings -----------------------------------------------------------------

export const settingsGet = () => call<Settings>('settings_get');
export const setOutput = (output: OutputPrefs) => call<void>('settings_set_output', { output });
export const setPlayback = (playback: PlaybackPrefs) =>
	call<void>('settings_set_playback', { playback });
export const setDsp = (dsp: DspPrefs) => call<void>('settings_set_dsp', { dsp });
export const outputDevices = () => call<Device[]>('output_devices');
export const eqCurve = (dsp: DspPrefs) => call<EqCurve>('eq_curve', { dsp });
export const autoEqImport = (title: string, filterName: string) =>
	call<AutoEqImport | null>('autoeq_import', { title, filterName });
export const autoEqExport = (title: string, filterName: string) =>
	call<boolean>('autoeq_export', { title, filterName });

// Events -------------------------------------------------------------------

/** Listens to a backend event; resolves to the function that stops it. */
export function on<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
	if (!isTauri()) return Promise.resolve(() => {});
	return listen<T>(event, (message) => handler(message.payload));
}

export const EVENTS = {
	playerState: 'player://state',
	playerPosition: 'player://position',
	playerMeter: 'player://meter',
	scan: 'library://scan',
	libraryChanged: 'library://changed'
} as const;

/** URL of a cover thumbnail, served by the backend's `onsa` protocol. */
export function coverUrl(id: number, size: 128 | 512): string {
	const windows = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent);
	return windows
		? `http://onsa.localhost/cover/${id}/${size}`
		: `onsa://localhost/cover/${id}/${size}`;
}
