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
	mini: boolean;
	closeToTray: boolean;
}

/** The words the tray menu shows. */
export interface TrayLabels {
	show: string;
	play: string;
	previous: string;
	next: string;
	quit: string;
}

/** When the sleep timer fires and what it then does (SPEC section 3.5). */
export type SleepWhen =
	| { kind: 'minutes'; minutes: number }
	| { kind: 'endOfTrack' }
	| { kind: 'tracks'; tracks: number };

export interface SleepPlan {
	when: SleepWhen;
	action: 'pause' | 'stop' | 'quit';
	fadeSeconds: number;
}

export interface SleepStatus {
	armed: boolean;
	secondsLeft: number | null;
	tracksLeft: number | null;
	plan: SleepPlan | null;
	fading: boolean;
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
	| { kind: 'artist'; name: string; index: number }
	| { kind: 'genre'; name: string; index: number }
	| { kind: 'folder'; path: string; index: number }
	| { kind: 'tracks'; ids: number[]; index: number };

/** Where added tracks join the queue. */
export type QueuePlace = 'next' | 'end';

/** What happens when a track ends. */
export type RepeatKind = 'off' | 'all' | 'one';

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
	/** Where the current entry sits, for scrolling to it. */
	current: number | null;
	/** Which entry is playing; positions change, this does not. */
	currentId: number | null;
	queueLength: number;
	track: Track | null;
	position: number;
	duration: number | null;
	volumeDb: number;
	shuffle: boolean;
	repeat: RepeatKind;
	signal: SignalPath;
	outputError: boolean;
	failedTrack: string | null;
}

export interface Position {
	index: number;
	seconds: number;
}

/** One analysis frame: the meters, and the spectrum when one is showing. */
export interface Meter {
	peakDb: [number, number];
	clip: boolean;
	limiting: boolean;
	/** Level per band, 0 to 255; empty while no visualizer is on screen. */
	bands: number[];
	/** Held peak per band, on the same scale. */
	peaks: number[];
	/** Spectral centroid in Hz, which tone colour follows. */
	centroidHz: number;
}

/** What the interface is showing that needs the analysis tap. */
export interface Watching {
	spectrum: boolean;
	meters: boolean;
}

/** One queue entry: the track, and what tells this entry apart from another
 * entry of the same track. */
export interface QueueEntry {
	entryId: number;
	track: Track;
}

export interface Queue {
	items: QueueEntry[];
	/** Id of the entry that is playing. */
	current: number | null;
}

export interface OutputPrefs {
	deviceId: string | null;
	sampleRate: number | null;
	matchSource: boolean;
}

export interface PlaybackPrefs {
	crossfadeSeconds: number;
	curve: Curve;
	skipCrossfadeSeconds: number;
	albumGapless: boolean;
	quality: Quality;
	buffer: BufferChoice;
	repeat: RepeatKind;
	/** The largest buffer, the cheapest resampler, and less of everything
	 * that only feeds the eye. */
	powerSave: boolean;
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

export interface DisplayPrefs {
	/** Whether the hue follows the character of the sound. */
	toneColor: boolean;
}

export interface Settings {
	output: OutputPrefs;
	playback: PlaybackPrefs;
	dsp: DspPrefs;
	display: DisplayPrefs;
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
export const artistCount = () => call<number>('library_artist_count');
export const artistsPage = (offset: number, limit: number) =>
	call<NameCount[]>('library_artists', { offset, limit });
export const artistTracks = (name: string) => call<Track[]>('library_artist_tracks', { name });
export const genreCount = () => call<number>('library_genre_count');
export const genresPage = (offset: number, limit: number) =>
	call<NameCount[]>('library_genres', { offset, limit });
export const genreTracks = (name: string) => call<Track[]>('library_genre_tracks', { name });
export const folderCount = () => call<number>('library_folder_count');
export const directoriesPage = (offset: number, limit: number) =>
	call<NameCount[]>('library_directories', { offset, limit });
export const folderTracks = (path: string) => call<Track[]>('library_folder_tracks', { path });

// Player -------------------------------------------------------------------

export const play = (context: PlayContext) => call<void>('player_play', { context });
export const togglePlay = () => call<void>('player_toggle');
export const nextTrack = () => call<void>('player_next');
export const previousTrack = () => call<void>('player_previous');
export const seek = (seconds: number) => call<void>('player_seek', { seconds });
export const jump = (entry: number) => call<void>('player_jump', { entry });
export const playerSnapshot = () => call<PlayerSnapshot>('player_snapshot');
export const playerQueue = () => call<Queue>('player_queue');
export const enqueue = (context: PlayContext, place: QueuePlace) =>
	call<void>('player_enqueue', { context, place });
export const removeFromQueue = (entry: number) => call<void>('player_remove', { entry });
export const moveInQueue = (entry: number, to: number) =>
	call<void>('player_move', { entry, to });
export const clearQueue = () => call<void>('player_clear');
export const setShuffle = (shuffle: boolean) => call<void>('player_shuffle', { shuffle });
export const sleepArm = (plan: SleepPlan) => call<void>('sleep_arm', { plan });
export const sleepCancel = () => call<void>('sleep_cancel');
export const sleepStatus = () => call<SleepStatus>('sleep_status');
export const traySetup = (labels: TrayLabels) => call<void>('tray_setup', { labels });
export const setCloseToTray = (enabled: boolean) => call<void>('set_close_to_tray', { enabled });
export const setMini = (mini: boolean) => call<void>('window_set_mini', { mini });
export const themeOpenFolder = () => call<void>('theme_open_folder');
export const setVolume = (db: number) => call<void>('player_set_volume', { db });

// Settings -----------------------------------------------------------------

export const settingsGet = () => call<Settings>('settings_get');
export const setOutput = (output: OutputPrefs) => call<void>('settings_set_output', { output });
export const setPlayback = (playback: PlaybackPrefs) =>
	call<void>('settings_set_playback', { playback });
export const setDsp = (dsp: DspPrefs) => call<void>('settings_set_dsp', { dsp });
export const setDisplay = (display: DisplayPrefs) =>
	call<void>('settings_set_display', { display });
export const watchAnalysis = (watching: Watching) => call<void>('player_watch', { watching });
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
	playerQueue: 'player://queue',
	sleep: 'player://sleep',
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
