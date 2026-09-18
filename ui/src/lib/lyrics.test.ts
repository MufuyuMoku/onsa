/**
 * Which line the window lights while a song plays.
 *
 * The property worth testing is that the line lit is the one being sung:
 * never the one about to be, never one the file gave no time to, and that
 * an offset the listener tuned moves them all together.
 */

import assert from 'node:assert/strict';
import { test } from 'node:test';

import type { Lyrics } from './backend.ts';
import { lineAt } from './lyrics.ts';

function words(lines: [number | null, string][], offsetMs = 0): Lyrics {
	return {
		trackId: 1,
		source: 'file',
		synced: lines.some(([at]) => at !== null),
		lines: lines.map(([atMs, text]) => ({ atMs, text })),
		offsetMs,
		looking: false,
		online: false,
		canAsk: false,
		beside: true
	};
}

const three = words([
	[1_000, 'satu'],
	[2_000, 'dua'],
	[3_000, 'tiga']
]);

test('nothing is lit before the first line', () => {
	assert.equal(lineAt(three, 0), -1);
	assert.equal(lineAt(three, 0.999), -1);
});

test('the line being sung is the last one that has begun', () => {
	assert.equal(lineAt(three, 1), 0);
	assert.equal(lineAt(three, 2.5), 1);
	assert.equal(lineAt(three, 900), 2);
});

test('an offset moves every line together', () => {
	const late = words(
		[
			[1_000, 'satu'],
			[2_000, 'dua']
		],
		500
	);
	// Held back half a second: at two seconds the second line has not
	// arrived yet.
	assert.equal(lineAt(late, 2), 0);
	assert.equal(lineAt(late, 2.5), 1);

	const early = words(
		[
			[1_000, 'satu'],
			[2_000, 'dua']
		],
		-500
	);
	assert.equal(lineAt(early, 1.6), 1);
});

test('a line with no time of its own is never the one being sung', () => {
	const mixed = words([
		[1_000, 'satu'],
		[null, '(instrumental)'],
		[3_000, 'tiga']
	]);
	assert.equal(lineAt(mixed, 2), 0);
	assert.equal(lineAt(mixed, 3.5), 2);
});

test('words without any timing light nothing at all', () => {
	const plain = words([
		[null, 'satu'],
		[null, 'dua']
	]);
	assert.equal(lineAt(plain, 30), -1);
	assert.equal(lineAt(null, 30), -1);
});
