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
	| { kind: 'tracks'; ids: number[]; index: number }
	| { kind: 'playlist'; playlistId: number; index: number }
	| { kind: 'queue' };

// Playlists (SPEC section 6) ------------------------------------------------

/** What a smart rule looks at. */
export type RuleField =
	| 'title'
	| 'artist'
	| 'album'
	| 'album_artist'
	| 'genre'
	| 'year'
	| 'rating'
	| 'play_count'
	| 'last_played'
	| 'added_at'
	| 'codec'
	| 'duration'
	| 'folder'
	| 'sample_rate'
	| 'bit_depth'
	| 'bitrate'
	| 'has_cover'
	| 'has_artist';

/** The kinds of value a field holds, which decide the operators it takes. */
export type RuleKind = 'text' | 'number' | 'date' | 'flag';

/** How a rule compares. */
export type RuleOp =
	| 'contains'
	| 'not_contains'
	| 'is'
	| 'is_not'
	| 'starts_with'
	| 'not_starts_with'
	| '='
	| '!='
	| '<'
	| '<='
	| '>'
	| '>='
	| 'between'
	| 'in_last'
	| 'not_in_last'
	| 'before'
	| 'after'
	| 'yes'
	| 'no';

/** What a rule compares with: text, a number, `[a, b]` or `{ days }`. */
export type RuleValue = string | number | [number, number] | { days: number } | null;

export interface Rule {
	field: RuleField;
	op: RuleOp;
	value: RuleValue;
}

export type SortField =
	| 'title'
	| 'artist'
	| 'album'
	| 'year'
	| 'duration'
	| 'rating'
	| 'play_count'
	| 'last_played'
	| 'added_at'
	| 'random';

export interface RuleSort {
	field: SortField;
	descending: boolean;
}

export interface Rules {
	match: 'all' | 'any';
	rules: Rule[];
	sort: RuleSort;
	limit: number | null;
}

export interface Playlist {
	id: number;
	name: string;
	kind: 'manual' | 'smart';
	rules: Rules | null;
	trackCount: number;
	createdAt: number;
	updatedAt: number;
}

/** What came of reading a playlist file. */
export interface PlaylistImport {
	playlistId: number | null;
	name: string;
	found: number;
	missing: string[];
}

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
export type ToneStrength = 'off' | 'subtle' | 'medium' | 'strong';

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

/** How one track list is laid out: its column widths and what is shown. */
export interface ColumnPrefs {
	widths: Record<string, number>;
	hidden: string[];
}

/** Where a key came from. Onsa never tells the interface what it is. */
export type KeySource = 'settings' | 'environment' | 'build' | 'none';

export interface KeyStatus {
	present: boolean;
	source: KeySource;
}

/** What Onsa may ask the internet about a track, and which key it has. */
export interface MetadataPrefs {
	online: boolean;
	acoustid: KeyStatus;
}

export interface DisplayPrefs {
	/** How far the colour follows the character of the sound. */
	toneColor: ToneStrength;
	/** Column widths and choices, one entry per list already arranged. */
	columns: Record<string, ColumnPrefs>;
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
	auto_eq_empty: 'error.auto_eq_empty',
	offline: 'error.offline',
	busy: 'error.busy',
	download: 'error.download',
	no_folder: 'error.no_folder'
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

// When the library cannot be opened at all ---------------------------------

/** Which kind of trouble the database is in. */
export type TroubleKind = 'fromNewerOnsa' | 'unreadable';

/** What the window is told when the library could not be opened. */
export interface StartupTrouble {
	kind: TroubleKind;
	database: string;
	folder: string;
	said: string;
}

/** Nothing means the library opened, which is the ordinary answer. */
export const startupTrouble = () => call<StartupTrouble | null>('startup_trouble');
export const startupOpenFolder = () => call<void>('startup_open_folder');

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
/** One library folder, and whether it is still where it was put. */
export interface LibraryFolder {
	name: string;
	trackCount: number;
	/** Whether that folder is on the disk right now. */
	present: boolean;
}

export const libraryFolders = () => call<LibraryFolder[]>('library_folders');
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
export const setMini = (mini: boolean) => call<boolean>('window_set_mini', { mini });
export const themeOpenFolder = () => call<void>('theme_open_folder');

/** A theme file that is in the folder but could not be used. */
export interface ThemeTrouble {
	file: string;
	said: string;
}

export const themeTroubles = () => call<ThemeTrouble[]>('theme_troubles');
/** Copies a built-in theme into the theme folder; answers with the path. */
export const themeCopyBuiltin = (id: string, name: string) =>
	call<string>('theme_copy_builtin', { id, name });
export const setVolume = (db: number) => call<void>('player_set_volume', { db });

// Tidying metadata (SPEC section 8) ----------------------------------------

/** The folder a run is held to, and how many tracks it may take. */
export interface TidyScope {
	folder: string | null;
	limit: number;
	defaultLimit: number;
	maxLimit: number;
	tracks: number;
}

/** One field a run would set. */
export interface TidyChange {
	trackId: number;
	field: string;
	value: string | null;
}

/** Why some tracks would be left alone. */
export interface Skip {
	reason: 'outsideFolder' | 'overLimit' | 'noChange' | 'nameTaken';
	count: number;
}

/** What a run would do, before there is a button to press. */
export interface TidySummary {
	tracks: number;
	fields: number;
	filesWritten: number;
	filesMoved: number;
	covers: number;
	skipped: Skip[];
	any: boolean;
}

/** What a run actually did. */
export interface TidyReport {
	batch: number | null;
	changed: number;
	unchanged: number;
	outOfScope: number;
	overLimit: number;
	failed: string[];
}

export interface PlannedMove {
	trackId: number;
	from: string;
	to: string;
	clash: string | null;
}

export interface RenamePlan {
	moves: PlannedMove[];
	summary: TidySummary;
}

/** A run as the history lists it. */
export interface EditBatch {
	id: number;
	note: string;
	scope: string | null;
	createdAt: number;
	undoneAt: number | null;
	steps: number;
}

export interface UndoReport {
	restored: number;
	failed: string[];
}

export const tidyScope = () => call<TidyScope>('tidy_scope');
export const tidySetScope = (folder: string | null, limit: number) =>
	call<TidyScope>('tidy_set_scope', { folder, limit });
export const tidyPickFolder = (title: string) =>
	call<string | null>('tidy_pick_folder', { title });
export const tidyTracks = () => call<Track[]>('tidy_tracks');

/** One thing Onsa would write differently about the tags it already has. */
export interface Suggestion {
	trackId: number;
	path: string;
	field: string;
	current: string | null;
	value: string;
	reasons: string[];
}

export const tidyAuto = () => call<Suggestion[]>('tidy_auto');
export const tidyPreviewEdits = (changes: TidyChange[]) =>
	call<TidySummary>('tidy_preview_edits', { changes });
export const tidyApplyEdits = (note: string, changes: TidyChange[]) =>
	call<TidyReport>('tidy_apply_edits', { note, changes });
export const tidyPreviewWrites = (tracks: number[]) =>
	call<TidySummary>('tidy_preview_writes', { tracks });
export const tidyWrite = (note: string, tracks: number[]) =>
	call<TidyReport>('tidy_write', { note, tracks });
export const tidyRenamePlan = (root: string, pattern: string, tracks: number[]) =>
	call<RenamePlan>('tidy_rename_plan', { root, pattern, tracks });
export const tidyRenameApply = (
	note: string,
	root: string,
	pattern: string,
	tracks: number[]
) => call<TidyReport>('tidy_rename_apply', { note, root, pattern, tracks });
export const editHistory = (limit: number) => call<EditBatch[]>('edit_history', { limit });
export const editUndo = (batch: number) => call<UndoReport>('edit_undo', { batch });

// Looking a track up on the internet (SPEC section 8, section 14) -----------

/** An outside program Onsa runs, and whether it is there (SPEC section 7.1). */
export interface ProgramStatus {
	key: string;
	present: boolean;
	from: 'managed' | 'chosen' | 'system' | null;
	path: string | null;
	version: string | null;
}

/** Where a suggestion came from. */
export type MatchSource = 'acoustId' | 'musicBrainz' | 'fileName';

/** One field a service would change. */
export interface FieldProposal {
	field: string;
	current: string | null;
	suggested: string | null;
	chosen: boolean;
}

/** A picture offered with a suggestion, waiting to be looked at. */
export interface CoverOffer {
	key: string;
	width: number;
	height: number;
	chosen: boolean;
}

/** One answer about one track. */
export interface Proposal {
	trackId: number;
	path: string;
	source: MatchSource;
	confidence: number;
	trusted: boolean;
	fields: FieldProposal[];
	cover: CoverOffer | null;
	releaseGroup: string | null;
}

/** Everything found about one track, and what came of it. */
export interface TrackMatch {
	trackId: number;
	path: string;
	candidates: Proposal[];
	picked: number | null;
	disagree: boolean;
	failure: string | null;
}

/** How a matching run stands, and everything it has found. */
export interface MatchState {
	running: boolean;
	done: number;
	total: number;
	trouble: string | null;
	found: TrackMatch[];
	online: boolean;
	key: boolean;
	fingerprinter: boolean;
}

/** One cover the listener said yes to. */
export interface CoverPick {
	trackId: number;
	key: string;
}

export const programStatus = () => call<ProgramStatus[]>('program_status');
export const programChoose = (program: string, path: string | null) =>
	call<void>('program_choose', { program, path });
export const programPick = (title: string) => call<string | null>('program_pick', { title });

// The downloader (SPEC section 7) ------------------------------------------

/** One program the downloader needs, and what Onsa can do about it. */
export interface BinaryStatus {
	key: string;
	present: boolean;
	/**
	 * Where the copy in use came from. `systemAnyway` is the one worth a
	 * word of its own: ffmpeg found on the system although Onsa was told
	 * not to use the system's programs, because yt-dlp looks for it itself.
	 */
	from: 'managed' | 'chosen' | 'system' | 'systemAnyway' | null;
	path: string | null;
	version: string | null;
	installable: boolean;
	url: string | null;
	aboutBytes: number | null;
	required: boolean;
}

/** How far along fetching a program is. */
export interface FetchProgress {
	key: string;
	done: number;
	total: number | null;
	finished: boolean;
	failed: string | null;
}

/** The programs, and whether a copy already on the system may be used. */
export interface Binaries {
	programs: BinaryStatus[];
	useSystem: boolean;
	/** Whether there is an ffmpeg for yt-dlp to use. */
	canConvert: boolean;
}

export const binariesStatus = () => call<Binaries>('binaries_status');
export const binaryInstall = (program: string) =>
	call<BinaryStatus>('binary_install', { program });
export const binaryStop = () => call<void>('binary_stop');
export const binariesUseSystem = (allowed: boolean) =>
	call<Binaries>('binaries_use_system', { allowed });

/** One thing at a URL, as yt-dlp reports it without fetching anything. */
export interface DownloadEntry {
	url: string;
	title: string;
	seconds: number | null;
	uploader: string | null;
}

/** What is at a URL. */
export interface Probe {
	title: string;
	playlist: boolean;
	entries: DownloadEntry[];
}

/** One row of the download queue. */
export interface QueueItem {
	id: number;
	url: string;
	title: string;
	state: 'waiting' | 'running' | 'done' | 'failed' | 'stopped';
	fraction: number | null;
	speed: number | null;
	eta: number | null;
	file: string | null;
	failed: string | null;
	/** What yt-dlp itself said, kept behind "see the detail". */
	said: string | null;
}

/** The queue as it stands. */
export interface DownloadQueue {
	running: boolean;
	items: QueueItem[];
	folder: string | null;
}

/** What Onsa asks yt-dlp for (SPEC section 7.2). */
export type DownloadFormat = 'original' | 'mp3' | 'flac';

export const downloadProbe = (url: string) => call<Probe>('download_probe', { url });

/** Why a URL could not be looked at, with yt-dlp's own sentence. */
export interface UrlTrouble {
	code: string;
	said: string | null;
}

/** Reads the trouble a refused URL came back with, when that is what it is. */
export function urlTroubleOf(error: unknown): UrlTrouble | null {
	const cause = error instanceof BackendError ? error.cause : error;
	if (typeof cause !== 'object' || cause === null) return null;
	const code = (cause as { code?: unknown }).code;
	if (typeof code !== 'string') return null;
	const said = (cause as { said?: unknown }).said;
	return { code, said: typeof said === 'string' ? said : null };
}
export const downloadQueue = () => call<DownloadQueue>('download_queue');
export const downloadStart = (items: { url: string; title: string }[], format: DownloadFormat) =>
	call<DownloadQueue>('download_start', { items, format });
export const downloadStop = () => call<DownloadQueue>('download_stop');
export const matchState = () => call<MatchState>('match_state');
export const matchStart = (tracks: number[]) => call<MatchState>('match_start', { tracks });
export const matchStop = () => call<MatchState>('match_stop');
export const tidyPreviewCovers = (tracks: number[]) =>
	call<TidySummary>('tidy_preview_covers', { tracks });
export const tidyApplyCovers = (note: string, picks: CoverPick[]) =>
	call<TidyReport>('tidy_apply_covers', { note, picks });

// Playlists ----------------------------------------------------------------

export const playlists = () => call<Playlist[]>('playlist_list');
export const playlistGet = (id: number) => call<Playlist | null>('playlist_get', { id });
export const playlistTracks = (id: number) => call<Track[]>('playlist_tracks', { id });
export const playlistCreate = (name: string, rules: Rules | null = null) =>
	call<number>('playlist_create', { name, rules });
export const playlistRename = (id: number, name: string) =>
	call<void>('playlist_rename', { id, name });
export const playlistSetRules = (id: number, rules: Rules) =>
	call<void>('playlist_set_rules', { id, rules });
export const playlistDelete = (id: number) => call<void>('playlist_delete', { id });
export const playlistDuplicate = (id: number, name: string) =>
	call<number>('playlist_duplicate', { id, name });
export const playlistAdd = (id: number, context: PlayContext) =>
	call<number>('playlist_add', { id, context });
export const playlistRemove = (id: number, position: number) =>
	call<void>('playlist_remove', { id, position });
export const playlistMove = (id: number, from: number, to: number) =>
	call<void>('playlist_move', { id, from, to });
export const playlistFromQueue = (name: string) => call<number>('playlist_from_queue', { name });
export const playlistExport = (id: number, title: string, filterName: string) =>
	call<string | null>('playlist_export', { id, title, filterName });
export const playlistImport = (title: string, filterName: string) =>
	call<PlaylistImport | null>('playlist_import', { title, filterName });
export const smartPreview = (rules: Rules, limit: number) =>
	call<Track[]>('smart_preview', { rules, limit });

// Editing one track's tags (SPEC section 8) --------------------------------

/** One field, as the editor shows it. */
export interface TrackField {
	name: string;
	value: string | null;
	/** Whether the value shown is Onsa's rather than the file's. */
	edited: boolean;
	/** Whether the file itself still says something else. */
	unwritten: boolean;
}

/** What a file says about its own loudness. Read only. */
export interface TrackGain {
	trackGainDb: number | null;
	trackPeak: number | null;
	albumGainDb: number | null;
	albumPeak: number | null;
}

/** One song, as the editor sees it. */
export interface TrackFields {
	trackId: number;
	path: string;
	fields: TrackField[];
	gain: TrackGain;
	anyUnwritten: boolean;
}

/** One value the listener typed. */
export interface TypedField {
	field: string;
	value: string | null;
}

export const trackFields = (trackId: number) => call<TrackFields>('track_fields', { trackId });
export const trackPreview = (trackId: number, fields: TypedField[]) =>
	call<TidySummary>('track_preview', { trackId, fields });
export const trackApply = (note: string, trackId: number, fields: TypedField[]) =>
	call<TidyReport>('track_apply', { note, trackId, fields });
export const trackWrite = (note: string, trackId: number) =>
	call<TidyReport>('track_write', { note, trackId });

// Lyrics (SPEC section 10) -------------------------------------------------

/** Where a song's words came from. */
export type LyricsSource = 'edited' | 'file' | 'tag' | 'lrclib' | 'none';

/** One line of lyrics. `atMs` is null for a line with no time of its own. */
export interface LyricsLine {
	atMs: number | null;
	text: string;
}

/** A song's words, and everything needed to show them. */
export interface Lyrics {
	trackId: number;
	source: LyricsSource;
	synced: boolean;
	lines: LyricsLine[];
	offsetMs: number;
	looking: boolean;
	online: boolean;
	canAsk: boolean;
	beside: boolean;
}

/** What Onsa may do about a song's words. */
export interface LyricsPrefs {
	online: boolean;
	writeBeside: boolean;
}

/** Asks for a song's words, and lets a lookup start if one is worth it. */
export const lyricsFor = (trackId: number) => call<Lyrics>('lyrics_for', { trackId });
/** The same, starting nothing: what to ask after a lookup says it finished. */
export const lyricsState = (trackId: number) => call<Lyrics>('lyrics_state', { trackId });
export const lyricsLookAgain = (trackId: number) => call<Lyrics>('lyrics_look_again', { trackId });
export const lyricsOffset = (trackId: number, offsetMs: number) =>
	call<Lyrics>('lyrics_offset', { trackId, offsetMs });
export const lyricsWriteBeside = (trackId: number) =>
	call<Lyrics>('lyrics_write_beside', { trackId });
export const lyricsSettings = () => call<LyricsPrefs>('lyrics_settings');
export const setLyricsSettings = (online: boolean, writeBeside: boolean) =>
	call<LyricsPrefs>('lyrics_set_settings', { online, writeBeside });

// Settings -----------------------------------------------------------------

export const settingsGet = () => call<Settings>('settings_get');
export const setOutput = (output: OutputPrefs) => call<void>('settings_set_output', { output });
export const setPlayback = (playback: PlaybackPrefs) =>
	call<void>('settings_set_playback', { playback });
export const setDsp = (dsp: DspPrefs) => call<void>('settings_set_dsp', { dsp });
/** What came of asking AcoustID whether it knows the key Onsa would use. */
export type KeyTest = 'works' | 'refused' | 'missing' | 'offline' | 'unreachable';

export const metadataGet = () => call<MetadataPrefs>('metadata_get');
export const acoustidTest = () => call<KeyTest>('acoustid_test');
/** Leave `acoustidKey` out to keep the stored key; empty clears it. */
export const setMetadata = (online: boolean, acoustidKey?: string) =>
	call<MetadataPrefs>('settings_set_metadata', { online, acoustidKey });
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
	libraryChanged: 'library://changed',
	playlists: 'library://playlists',
	windowMode: 'window://mode',
	matching: 'match://progress',
	binary: 'downloads://binary',
	downloads: 'downloads://progress',
	lyrics: 'lyrics://arrived'
} as const;

/** URL of a cover thumbnail, served by the backend's `onsa` protocol. */
export function coverUrl(id: number, size: 128 | 512): string {
	return protocolUrl(`cover/${id}/${size}`);
}

/**
 * URL of a cover a service has offered and nobody has agreed to yet.
 *
 * It is served from the same cache as any other cover, one folder further
 * in, so a picture can be looked at before it is applied.
 */
export function proposedCoverUrl(key: string, size: 128 | 512): string {
	return protocolUrl(`proposed/${key}/${size}`);
}

function protocolUrl(path: string): string {
	const windows = typeof navigator !== 'undefined' && /Windows/i.test(navigator.userAgent);
	return windows ? `http://onsa.localhost/${path}` : `onsa://localhost/${path}`;
}
