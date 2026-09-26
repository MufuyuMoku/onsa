/**
 * Which columns fit, and how wide they are drawn. This is where a width the
 * listener chose has to meet the narrowing rules without either one winning
 * outright.
 */

import assert from 'node:assert/strict';
import { test } from 'node:test';

import {
	COLUMNS,
	MENU_WIDTH,
	NUMBER_WIDTH,
	column,
	emptyPrefs,
	shownColumns,
	spreadWidths,
	template,
	overhead,
	titleWidth,
	widthOf,
	type ColumnKey
} from './columns.ts';

const all = () => true;
const noAlbum = (key: string) => key !== 'album';

test('a wide list shows every column', () => {
	assert.deepEqual(shownColumns(1200, emptyPrefs(), all), [
		'title',
		'artist',
		'album',
		'year',
		'duration'
	]);
});

test('columns leave in the order the specification gives: year, album, artist', () => {
	const prefs = emptyPrefs();
	const widest = COLUMNS.reduce((sum, c) => sum + (c.key === 'title' ? c.min : c.width), 0);
	const full = NUMBER_WIDTH + MENU_WIDTH + overhead(COLUMNS.length) + widest;
	assert.ok(shownColumns(full, prefs, all).includes('year'), 'exactly enough room keeps them all');
	assert.ok(!shownColumns(full - 1, prefs, all).includes('year'), 'the year goes first');
	const tight = shownColumns(300, prefs, all);
	assert.deepEqual(tight, ['title', 'duration'], 'the title and the length always stay');
});

test('a column the listener took off does not come back when there is room', () => {
	const prefs = { ...emptyPrefs(), hidden: ['artist' as const] };
	assert.deepEqual(shownColumns(1400, prefs, all), ['title', 'album', 'year', 'duration']);
});

test('a list that has no album column of its own never shows one', () => {
	assert.deepEqual(shownColumns(1400, emptyPrefs(), noAlbum), [
		'title',
		'artist',
		'year',
		'duration'
	]);
});

test('a width the listener dragged is what gets drawn', () => {
	const prefs = { ...emptyPrefs(), widths: { artist: 300 } };
	assert.equal(widthOf('artist', prefs), 300);
	assert.ok(template(1200, ['title', 'artist'], prefs).includes('300px'));
});

test('a width below the column minimum is pulled back up to it', () => {
	const prefs = { ...emptyPrefs(), widths: { artist: 10 } };
	assert.equal(widthOf('artist', prefs), 90);
	const broken = { ...emptyPrefs(), widths: { artist: Number.NaN } };
	assert.equal(widthOf('artist', broken), 170, 'and nonsense falls back to the default');
});

test('a column widened past what fits pushes another one out', () => {
	const roomy = shownColumns(900, emptyPrefs(), all);
	assert.ok(roomy.includes('year'));
	const wide = { ...emptyPrefs(), widths: { artist: 420, album: 420 } };
	const cramped = shownColumns(900, wide, all);
	assert.ok(!cramped.includes('year'), 'the widths are honoured, so the year steps out');
	assert.ok(cramped.includes('title') && cramped.includes('duration'));
});

test('the title is the width it was given, and the slack goes to the end', () => {
	// What makes a divider follow the pointer: the column being dragged is
	// the one that changes, and the room nobody claimed sits at the right
	// end rather than inside the title.
	const prefs = { ...emptyPrefs(), widths: { title: 240 } };
	assert.equal(
		template(1200, ['title', 'duration'], prefs),
		`${NUMBER_WIDTH}px 240px 56px minmax(${MENU_WIDTH}px, 1fr)`
	);
});

test('a title wider than the room gives way rather than spilling', () => {
	const prefs = { ...emptyPrefs(), widths: { title: 400 } };
	const shown = ['title', 'duration'] as const;
	// A width worked out from the parts rather than guessed, so that this
	// stays a test about the title giving way and not about the numbers
	// the strips at either end happen to have today.
	const left = column('title').min + 16;
	const room = NUMBER_WIDTH + MENU_WIDTH + overhead(shown.length) + column('duration').width + left;
	assert.equal(titleWidth(room, [...shown], prefs), left);
	assert.ok(
		template(room, [...shown], prefs).includes(`${left}px`),
		'the grid asks for no more than there is'
	);
	assert.equal(titleWidth(180, [...shown], prefs), 120, 'and never below its own minimum');
});

test('the whole grid fits the width it was given', () => {
	for (const available of [1400, 1000, 820, 686, 500]) {
		const prefs = { ...emptyPrefs(), widths: { artist: 300, album: 300 } };
		const shown = shownColumns(available, prefs, () => true);
		const fixed = shown
			.filter((key) => key !== 'title')
			.reduce((total, key) => total + widthOf(key, prefs), 0);
		const total =
			NUMBER_WIDTH +
			MENU_WIDTH +
			overhead(shown.length) +
			fixed +
			titleWidth(available, shown, prefs);
		const floor = NUMBER_WIDTH + MENU_WIDTH + overhead(2) + 120 + 56;
		assert.ok(
			total <= Math.max(available, floor),
			`${available}px holds ${shown.join(', ')} in ${total}px`
		);
	}
});

test('a column dragged wider takes its room from the ones after it', () => {
	// The dividers were reported as not working: the one under the pointer
	// stayed put because the title, to its left, absorbed everything.
	const prefs = emptyPrefs();
	const shown: ColumnKey[] = ['title', 'artist', 'album', 'year', 'duration'];
	const available =
		NUMBER_WIDTH +
		MENU_WIDTH +
		overhead(shown.length) +
		shown.reduce((total, key) => total + column(key).width, 0);
	const next = spreadWidths(available, shown, prefs, 'artist', column('artist').width + 60);

	assert.equal(next.artist, column('artist').width + 60, 'it follows the pointer');
	assert.equal(next.title, undefined, 'nothing before it moves');
	assert.equal(next.album, column('album').width - 60, 'the one after it gives way');
	assert.equal(next.year, undefined, 'and only as far as it had to');
});

test('a drag stops rather than pushing a column off the row', () => {
	const prefs = emptyPrefs();
	const shown: ColumnKey[] = ['title', 'artist', 'duration'];
	const available =
		NUMBER_WIDTH +
		MENU_WIDTH +
		overhead(shown.length) +
		shown.reduce((total, key) => total + column(key).width, 0);
	// Far more than the row can give. The only column after this one is
	// the length, which has no divider and never gives way, and the title
	// is before it — so there is nothing to take at all.
	const next = spreadWidths(available, shown, prefs, 'artist', 2000);

	assert.equal(next.artist, column('artist').width, 'it stopped where the room ran out');
	assert.equal(next.title, undefined, 'the title is before it and does not move');
	assert.equal(next.duration, undefined, 'a column with no divider never gives way');
});

test('the room nobody is using is spent before anything gives way', () => {
	const prefs = emptyPrefs();
	const shown: ColumnKey[] = ['title', 'artist', 'duration'];
	const slack = 80;
	const available =
		NUMBER_WIDTH +
		MENU_WIDTH +
		overhead(shown.length) +
		shown.reduce((total, key) => total + column(key).width, 0) +
		slack;
	const next = spreadWidths(available, shown, prefs, 'artist', column('artist').width + slack);

	assert.equal(next.artist, column('artist').width + slack);
	assert.deepEqual(Object.keys(next), ['artist'], 'nothing else had to move');
});

test('a column dragged narrower only changes itself', () => {
	const prefs = emptyPrefs();
	const shown: ColumnKey[] = ['title', 'artist', 'album', 'duration'];
	const next = spreadWidths(1400, shown, prefs, 'album', column('album').width - 40);
	assert.deepEqual(next, { album: column('album').width - 40 });
});

test('a column is never dragged below its own smallest', () => {
	const prefs = emptyPrefs();
	const shown: ColumnKey[] = ['title', 'artist', 'duration'];
	const next = spreadWidths(1400, shown, prefs, 'artist', 10);
	assert.equal(next.artist, column('artist').min);
});
