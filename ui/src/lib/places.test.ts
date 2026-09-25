/**
 * When a remembered position still stands for something, and when it does
 * not. Getting this wrong scrolls somebody to a song they never chose.
 */

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { placeApplies, type ListPlace, type Ordering } from './places.ts';

const at = (top: number, sort: ListPlace['sort'], descending = false): ListPlace => ({
	top,
	sort,
	descending
});

const sorted = (sort: Ordering['sort'], descending = false): Ordering => ({
	fixed: false,
	sort,
	descending
});

test('a position taken in this ordering is the one that comes back', () => {
	assert.equal(placeApplies(at(2048, 'title'), sorted('title')), true);
	assert.equal(placeApplies(at(2048, 'artist', true), sorted('artist', true)), true);
});

test('a position from another ordering is not forced onto this one', () => {
	// Row four hundred by title is a different song from row four hundred
	// by year, so the number means nothing here.
	assert.equal(placeApplies(at(2048, 'title'), sorted('year')), false);
	// The same column the other way round is the other end of the list.
	assert.equal(placeApplies(at(2048, 'title'), sorted('title', true)), false);
	assert.equal(placeApplies(at(2048, 'title', true), sorted('title')), false);
});

test('a list with an order of its own keeps its own kind of position', () => {
	const album: Ordering = { fixed: true, sort: 'title', descending: false };
	assert.equal(placeApplies(at(600, 'fixed'), album), true);
	// One taken while the library list was sorted says nothing about an album.
	assert.equal(placeApplies(at(600, 'title'), album), false);
	// And the other way round: an album's position is not the library's.
	assert.equal(placeApplies(at(600, 'fixed'), sorted('title')), false);
});

test('the top of the list is nothing worth restoring', () => {
	assert.equal(placeApplies(null, sorted('title')), false);
	assert.equal(placeApplies(at(0, 'title'), sorted('title')), false);
	assert.equal(placeApplies(at(-10, 'title'), sorted('title')), false);
});
