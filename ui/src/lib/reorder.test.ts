/**
 * The arithmetic behind dragging a row somewhere else. The pointer handling
 * needs a window and is checked on the running application; this is the part
 * that decides where a row actually lands, which is where the mistakes are.
 */

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { settle } from './reorder.ts';

test('dropping below a row closes the gap it came from', () => {
	// Four rows, the first dragged into the gap after the third.
	assert.equal(settle(0, 3, 4), 2);
	assert.equal(settle(0, 4, 4), 3, 'right to the end');
	assert.equal(settle(1, 4, 4), 3);
});

test('dropping above a row lands on that row', () => {
	assert.equal(settle(3, 0, 4), 0);
	assert.equal(settle(3, 1, 4), 1);
	assert.equal(settle(2, 2, 4), 2, 'the gap above itself is where it is');
});

test('a row let go where it already sits does not move', () => {
	for (let at = 0; at < 4; at++) {
		assert.equal(settle(at, at, 4), at);
		assert.equal(settle(at, at + 1, 4), at, 'the gap just below is the same place');
	}
});

test('a place beyond the list is pulled back into it', () => {
	assert.equal(settle(0, 99, 4), 3);
	assert.equal(settle(3, -5, 4), 0);
});

test('a list of one has nowhere to go', () => {
	assert.equal(settle(0, 1, 1), 0);
	assert.equal(settle(0, 0, 0), 0);
});
