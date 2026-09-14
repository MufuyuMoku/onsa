/**
 * What the ticks on a list of suggestions add up to.
 *
 * The property these tests are really about is the one the whole approval
 * flow rests on: what was not ticked does not happen, and a track whose
 * sources disagree does nothing at all until somebody settles it.
 */

import assert from 'node:assert/strict';
import { test } from 'node:test';

import type { Proposal, TrackMatch } from './backend.ts';
import { changesFrom, coversFrom, everyTick, tickKey } from './matches.ts';

function candidate(
	source: Proposal['source'],
	confidence: number,
	fields: [string, string | null, string | null][],
	cover?: string
): Proposal {
	return {
		trackId: 1,
		path: 'C:\\music\\a.mp3',
		source,
		confidence,
		trusted: confidence >= 0.85,
		fields: fields.map(([field, current, suggested]) => ({
			field,
			current,
			suggested,
			chosen: confidence >= 0.85 && current !== suggested
		})),
		cover: cover ? { key: cover, width: 500, height: 500, chosen: false } : null,
		releaseGroup: null
	};
}

function track(id: number, candidates: Proposal[], picked: number | null, disagree = false): TrackMatch {
	return {
		trackId: id,
		path: `C:\\music\\${id}.mp3`,
		candidates: candidates.map((one) => ({ ...one, trackId: id })),
		picked,
		disagree,
		failure: null
	};
}

const sure = () =>
	track(
		1,
		[
			candidate('acoustId', 0.97, [
				['title', null, 'Lampu Kota'],
				['artist', null, 'Lilith']
			])
		],
		0
	);

test('a sure match arrives ticked and comes out as changes', () => {
	const changes = changesFrom([sure()], {}, {});
	assert.deepEqual(changes, [
		{ trackId: 1, field: 'title', value: 'Lampu Kota' },
		{ trackId: 1, field: 'artist', value: 'Lilith' }
	]);
});

test('a track whose sources disagree does nothing until one is picked', () => {
	const split = track(
		2,
		[
			candidate('acoustId', 0.99, [['title', null, 'Lampu Kota']]),
			candidate('fileName', 0.6, [['title', null, 'Sore']])
		],
		null,
		true
	);
	assert.deepEqual(changesFrom([split], {}, {}), [], 'nothing at all');

	// Picking the second one, and ticking its field by hand.
	const picks = { 2: 1 };
	const ticks = { [tickKey(2, 1, 'title')]: true };
	assert.deepEqual(changesFrom([split], picks, ticks), [
		{ trackId: 2, field: 'title', value: 'Sore' }
	]);
});

test('unticking a field of a sure match takes it out again', () => {
	const ticks = { [tickKey(1, 0, 'artist')]: false };
	assert.deepEqual(changesFrom([sure()], {}, ticks), [
		{ trackId: 1, field: 'title', value: 'Lampu Kota' }
	]);
});

test('choosing to use nothing leaves the track alone', () => {
	assert.deepEqual(changesFrom([sure()], { 1: null }, {}), []);
});

test('a cover only counts once somebody has ticked it', () => {
	const withCover = track(
		3,
		[candidate('acoustId', 0.95, [['title', null, 'Sore']], 'abc123')],
		0
	);
	assert.deepEqual(coversFrom([withCover], {}, {}), [], 'a picture waits to be looked at');
	assert.deepEqual(coversFrom([withCover], {}, { 3: true }), [{ trackId: 3, key: 'abc123' }]);
	// Ticked, but then the listener decided to use no candidate at all.
	assert.deepEqual(coversFrom([withCover], { 3: null }, { 3: true }), []);
});

test('taking everything leaves alone what would change nothing', () => {
	const partly = track(
		4,
		[
			candidate('musicBrainz', 0.9, [
				['title', 'Sore', 'Sore'],
				['album', null, 'Kota Sunyi']
			])
		],
		0
	);
	const ticks = everyTick([partly], {}, true);
	assert.equal(ticks[tickKey(4, 0, 'title')], false, 'it already says that');
	assert.equal(ticks[tickKey(4, 0, 'album')], true);
	assert.deepEqual(changesFrom([partly], {}, ticks), [
		{ trackId: 4, field: 'album', value: 'Kota Sunyi' }
	]);

	const none = everyTick([partly], {}, false);
	assert.deepEqual(changesFrom([partly], {}, none), []);
});

test('a track nothing was found for contributes nothing', () => {
	const nothing: TrackMatch = {
		trackId: 5,
		path: 'C:\\music\\5.mp3',
		candidates: [],
		picked: null,
		disagree: false,
		failure: 'noFingerprinter'
	};
	assert.deepEqual(changesFrom([nothing], {}, {}), []);
	assert.deepEqual(coversFrom([nothing], {}, { 5: true }), []);
	assert.deepEqual(everyTick([nothing], {}, true), {});
});
