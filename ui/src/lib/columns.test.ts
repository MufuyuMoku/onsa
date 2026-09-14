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
	emptyPrefs,
	shownColumns,
	template,
	overhead,
	titleFloor,
	widthOf
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

test('the title takes the space left over, from its own width upwards', () => {
	const prefs = { ...emptyPrefs(), widths: { title: 240 } };
	assert.equal(
		template(1200, ['title', 'duration'], prefs),
		`${NUMBER_WIDTH}px minmax(240px, 1fr) 56px ${MENU_WIDTH}px`
	);
});

test('a title floor wider than the room gives way rather than spilling', () => {
	const prefs = { ...emptyPrefs(), widths: { title: 400 } };
	const shown = ['title', 'duration'] as const;
	// 300 wide, less the number, the menu, the length and the row's own
	// padding and gaps, leaves this much for the title.
	const left = 300 - NUMBER_WIDTH - MENU_WIDTH - overhead(shown.length) - 56;
	assert.equal(titleFloor(300, [...shown], prefs), left);
	assert.ok(
		template(300, [...shown], prefs).includes(`minmax(${left}px, 1fr)`),
		'the grid asks for no more than there is'
	);
	assert.equal(titleFloor(180, [...shown], prefs), 120, 'and never below its own minimum');
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
			titleFloor(available, shown, prefs);
		const floor = NUMBER_WIDTH + MENU_WIDTH + overhead(2) + 120 + 56;
		assert.ok(
			total <= Math.max(available, floor),
			`${available}px holds ${shown.join(', ')} in ${total}px`
		);
	}
});
