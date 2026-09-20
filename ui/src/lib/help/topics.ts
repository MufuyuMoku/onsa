/**
 * What each page is for, and what every control on it does.
 *
 * The help is written page by page, not as one long document: the panel
 * beside a page shows that page's entry and nothing else. Every control a
 * page draws has a line here, including the ones that look obvious — the
 * reader is somebody who has never used a music player that plays files
 * from a disk.
 *
 * The name of a control is the same dictionary key the control itself
 * uses, so the help says the words that are on the screen, in whichever
 * language the window is speaking.
 */

import type { View } from '$lib/app.svelte';
import type { MessageKey } from '$lib/i18n/dictionary';

/** One control, and what it does. */
export interface HelpControl {
	/** The words on the control, as the screen shows them. */
	name: MessageKey;
	/** What pressing it does, in plain words. */
	says: MessageKey;
}

/** The help for one page. */
export interface HelpTopic {
	/** How the page is named in `View`, plus `settings.<section>`. */
	id: string;
	title: MessageKey;
	/** One paragraph: what this page is for. */
	what: MessageKey;
	controls: HelpControl[];
	/** What happens to the listener's own files here, when anything can. */
	files?: MessageKey;
	/** What this page cannot do in this version. */
	limits: MessageKey[];
}

const control = (name: MessageKey, says: MessageKey): HelpControl => ({ name, says });

/** The song list, which six pages draw in the same shape. */
const trackList: HelpControl[] = [
	control('help.name.columns', 'help.says.columns'),
	control('help.name.resize', 'help.says.resize'),
	control('column.choose', 'help.says.chooseColumns'),
	control('help.name.row', 'help.says.row'),
	control('help.name.rowMenu', 'help.says.rowMenu')
];

export const TOPICS: HelpTopic[] = [
	{
		id: 'tracks',
		title: 'nav.tracks',
		what: 'help.tracks.what',
		controls: trackList,
		limits: ['help.tracks.limit']
	},
	{
		id: 'albums',
		title: 'nav.albums',
		what: 'help.albums.what',
		controls: [control('help.name.albumCard', 'help.says.albumCard')],
		limits: []
	},
	{
		id: 'album',
		title: 'help.title.album',
		what: 'help.album.what',
		controls: [
			control('album.back', 'help.says.albumBack'),
			control('album.play', 'help.says.albumPlay'),
			control('playlist.addTo', 'help.says.addTo'),
			...trackList
		],
		limits: []
	},
	{
		id: 'artists',
		title: 'nav.artists',
		what: 'help.artists.what',
		controls: [control('help.name.artistRow', 'help.says.artistRow')],
		limits: []
	},
	{
		id: 'artist',
		title: 'help.title.artist',
		what: 'help.artist.what',
		controls: [
			control('help.name.browseBack', 'help.says.browseBack'),
			control('transport.play', 'help.says.browsePlay'),
			control('playlist.addTo', 'help.says.addTo'),
			...trackList
		],
		limits: []
	},
	{
		id: 'genres',
		title: 'nav.genres',
		what: 'help.genres.what',
		controls: [control('help.name.genreRow', 'help.says.genreRow')],
		limits: ['help.genres.limit']
	},
	{
		id: 'genre',
		title: 'help.title.genre',
		what: 'help.genre.what',
		controls: [
			control('help.name.browseBack', 'help.says.browseBack'),
			control('transport.play', 'help.says.browsePlay'),
			control('playlist.addTo', 'help.says.addTo'),
			...trackList
		],
		limits: []
	},
	{
		id: 'folders',
		title: 'nav.folders',
		what: 'help.folders.what',
		controls: [control('help.name.folderRow', 'help.says.folderRow')],
		limits: []
	},
	{
		id: 'folder',
		title: 'help.title.folder',
		what: 'help.folder.what',
		controls: [
			control('help.name.browseBack', 'help.says.browseBack'),
			control('transport.play', 'help.says.browsePlay'),
			control('playlist.addTo', 'help.says.addTo'),
			...trackList
		],
		limits: []
	},
	{
		id: 'playlists',
		title: 'nav.playlists',
		what: 'help.playlists.what',
		controls: [
			control('playlist.new', 'help.says.playlistNew'),
			control('playlist.newSmart', 'help.says.playlistNewSmart'),
			control('playlist.import', 'help.says.playlistImport'),
			control('help.name.playlistCard', 'help.says.playlistCard')
		],
		files: 'help.playlists.files',
		limits: []
	},
	{
		id: 'playlist',
		title: 'help.title.playlist',
		what: 'help.playlist.what',
		controls: [
			control('help.name.playlistBack', 'help.says.playlistBack'),
			control('playlist.play', 'help.says.playlistPlay'),
			control('queue.addToEnd', 'help.says.playlistQueue'),
			control('playlist.more', 'help.says.playlistMore'),
			control('playlist.editRules', 'help.says.playlistRules'),
			control('help.name.playlistRow', 'help.says.playlistRow'),
			...trackList
		],
		files: 'help.playlist.files',
		limits: []
	},
	{
		id: 'search',
		title: 'help.title.search',
		what: 'help.search.what',
		controls: [
			control('help.name.searchArtist', 'help.says.searchArtist'),
			control('help.name.searchAlbum', 'help.says.searchAlbum'),
			...trackList
		],
		limits: ['help.search.limit']
	},
	{
		id: 'tidy',
		title: 'nav.tidy',
		what: 'help.tidy.what',
		controls: [
			control('help.name.tidyScope', 'help.says.tidyScope'),
			control('tidy.pickFolder', 'help.says.tidyPick'),
			control('tidy.clearFolder', 'help.says.tidyClear'),
			control('tidy.limit', 'help.says.tidyLimit'),
			control('tidy.jobEdit', 'help.says.tidyEdit'),
			control('tidy.jobWrite', 'help.says.tidyWrite'),
			control('tidy.jobRename', 'help.says.tidyRename'),
			control('tidy.jobAuto', 'help.says.tidyAuto'),
			control('tidy.jobMatch', 'help.says.tidyMatch'),
			control('rules.field', 'help.says.tidyField'),
			control('tidy.valueEmpty', 'help.says.tidyValue'),
			control('tidy.pattern', 'help.says.tidyPattern'),
			control('tidy.renameRoot', 'help.says.tidyRoot'),
			control('tidy.look', 'help.says.tidyLook'),
			control('tidy.apply', 'help.says.tidyApply'),
			control('tidy.undo', 'help.says.tidyUndo')
		],
		files: 'help.tidy.files',
		limits: ['help.tidy.limit']
	},
	{
		id: 'downloads',
		title: 'nav.downloads',
		what: 'help.downloads.what',
		controls: [
			control('downloads.useSystem', 'help.says.useSystem'),
			control('help.name.programRow', 'help.says.programRow'),
			control('downloads.agree', 'help.says.agree'),
			control('downloads.update', 'help.says.update'),
			control('downloads.urlPlaceholder', 'help.says.urlField'),
			control('downloads.look', 'help.says.look'),
			control('downloads.format', 'help.says.format'),
			control('downloads.fetchCount', 'help.says.fetch'),
			control('downloads.stop', 'help.says.stop'),
			control('downloads.stopAll', 'help.says.stopAll')
		],
		files: 'help.downloads.files',
		limits: ['help.downloads.limitOpus', 'help.downloads.limitOne']
	},
	{
		id: 'nowPlaying',
		title: 'nowPlaying.open',
		what: 'help.nowPlaying.what',
		controls: [
			control('nowPlaying.close', 'help.says.nowPlayingClose'),
			control('transport.seek', 'help.says.seek'),
			control('help.name.nowPlayingLyrics', 'help.says.nowPlayingLyrics')
		],
		limits: []
	},
	{
		id: 'settings.output',
		title: 'settings.output',
		what: 'help.output.what',
		controls: [
			control('output.device', 'help.says.device'),
			control('output.sampleRate', 'help.says.sampleRate'),
			control('output.matchSource', 'help.says.matchSource'),
			control('output.quality', 'help.says.quality'),
			control('output.buffer', 'help.says.buffer'),
			control('output.dither', 'help.says.dither'),
			control('output.powerSave', 'help.says.powerSave'),
			control('crossfade.duration', 'help.says.crossfade'),
			control('crossfade.curve', 'help.says.curve'),
			control('crossfade.albumGapless', 'help.says.albumGapless'),
			control('crossfade.skip', 'help.says.skipFade'),
			control('replaygain.mode', 'help.says.rgMode'),
			control('replaygain.preamp', 'help.says.rgPreamp'),
			control('replaygain.fallback', 'help.says.rgFallback'),
			control('replaygain.preventClipping', 'help.says.rgClip')
		],
		limits: []
	},
	{
		id: 'settings.dsp',
		title: 'settings.dsp',
		what: 'help.dsp.what',
		controls: [
			control('eq.enabled', 'help.says.eqOn'),
			control('eq.graphic', 'help.says.eqGraphic'),
			control('eq.parametric', 'help.says.eqParametric'),
			control('help.name.eqBand', 'help.says.eqBand'),
			control('eq.flat', 'help.says.eqFlat'),
			control('eq.import', 'help.says.eqImport'),
			control('eq.export', 'help.says.eqExport'),
			control('eq.addBand', 'help.says.eqAdd'),
			control('eq.remove', 'help.says.eqRemove'),
			control('preamp.auto', 'help.says.preampAuto'),
			control('preamp.manual', 'help.says.preampManual'),
			control('limiter.enabled', 'help.says.limiterOn'),
			control('limiter.release', 'help.says.limiterRelease')
		],
		limits: []
	},
	{
		id: 'settings.library',
		title: 'settings.library',
		what: 'help.library.what',
		controls: [
			control('librarySettings.add', 'help.says.addFolder'),
			control('librarySettings.rescan', 'help.says.rescan'),
			control('help.name.folderList', 'help.says.folderList')
		],
		files: 'help.library.files',
		limits: ['librarySettings.cannotRemove']
	},
	{
		id: 'settings.metadata',
		title: 'settings.metadata',
		what: 'help.metadata.what',
		controls: [
			control('metadata.online', 'help.says.metaOnline'),
			control('metadata.key', 'help.says.metaKey'),
			control('metadata.keySave', 'help.says.metaKeySave'),
			control('metadata.keyClear', 'help.says.metaKeyClear'),
			control('metadata.keyTry', 'help.says.metaKeyTry'),
			control('programs.choose', 'help.says.metaChoose'),
			control('programs.forget', 'help.says.metaForget'),
			control('programs.refresh', 'help.says.metaRefresh')
		],
		limits: []
	},
	{
		id: 'settings.lyrics',
		title: 'settings.lyrics',
		what: 'help.lyricsSettings.what',
		controls: [
			control('lyrics.onlineLabel', 'help.says.lyricsOnline'),
			control('lyrics.writeBesideLabel', 'help.says.lyricsBeside')
		],
		files: 'help.lyricsSettings.files',
		limits: []
	},
	{
		id: 'settings.appearance',
		title: 'settings.appearance',
		what: 'help.appearance.what',
		controls: [
			control('theme.label', 'help.says.themePick'),
			control('toneColor.label', 'help.says.tone'),
			control('theme.openFolder', 'help.says.themeFolder'),
			control('theme.copy', 'help.says.themeCopy'),
			control('language.label', 'help.says.language')
		],
		limits: []
	},
	{
		id: 'settings.about',
		title: 'settings.about',
		what: 'help.about.what',
		controls: [
			control('about.debug', 'help.says.debug'),
			control('about.openLogs', 'help.says.openLogs'),
			control('tray.closeToTray', 'help.says.closeToTray')
		],
		limits: []
	},
	{
		id: 'header',
		title: 'help.title.header',
		what: 'help.header.what',
		controls: [
			control('search.placeholder', 'help.says.search'),
			control('search.clear', 'help.says.searchClear'),
			control('queue.title', 'help.says.queueButton'),
			control('help.name.menuButton', 'help.says.menuButton'),
			control('help.open', 'help.says.helpButton')
		],
		limits: []
	},
	{
		id: 'transport',
		title: 'help.title.transport',
		what: 'help.transport.what',
		controls: [
			control('transport.seek', 'help.says.seek'),
			control('transport.previous', 'help.says.previous'),
			control('transport.play', 'help.says.playPause'),
			control('transport.next', 'help.says.next'),
			control('transport.volume', 'help.says.volume'),
			control('nowPlaying.open', 'help.says.openNowPlaying'),
			control('help.name.sleep', 'help.says.sleep'),
			control('mini.open', 'help.says.mini'),
			control('help.name.signal', 'help.says.signal')
		],
		limits: []
	},
	{
		id: 'panel',
		title: 'help.title.panel',
		what: 'help.panel.what',
		controls: [
			control('queue.title', 'help.says.queueTab'),
			control('lyrics.title', 'help.says.lyricsTab'),
			control('help.name.queueRow', 'help.says.queueRow'),
			control('queue.shuffle', 'help.says.shuffle'),
			control('queue.repeat', 'help.says.repeat'),
			control('queue.clear', 'help.says.clearQueue'),
			control('playlist.saveQueue', 'help.says.saveQueue'),
			control('queue.remove', 'help.says.removeFromQueue'),
			control('lyrics.earlier', 'help.says.lyricsEarlier'),
			control('lyrics.later', 'help.says.lyricsLater'),
			control('lyrics.resetOffset', 'help.says.lyricsReset'),
			control('lyrics.saveBeside', 'help.says.lyricsSave'),
			control('lyrics.lookAgain', 'help.says.lyricsAgain')
		],
		files: 'help.panel.files',
		limits: []
	},
	{
		id: 'help',
		title: 'nav.help',
		what: 'help.help.what',
		controls: [
			control('help.name.contents', 'help.says.contents'),
			control('help.name.contentsLinks', 'help.says.contentsLinks'),
			control('help.theme.copy', 'help.says.themeExample'),
			control('help.firstRun', 'help.says.firstRunAgain')
		],
		limits: []
	}
];

const byId = new Map(TOPICS.map((topic) => [topic.id, topic]));

/** The help for one page, by the name the view uses. */
export const topic = (id: string): HelpTopic | undefined => byId.get(id);

/** Which help belongs to what the window is showing. */
export function topicFor(view: View, searching: boolean): HelpTopic | undefined {
	if (searching) return byId.get('search');
	if (view.kind === 'settings') return byId.get(`settings.${view.section}`);
	return byId.get(view.kind);
}

/** The pages the help page lists, in the order the navigation has them. */
export const CONTENTS: string[] = TOPICS.map((entry) => entry.id);
