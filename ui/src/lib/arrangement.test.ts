import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
	arrangementFor,
	CLASSIC,
	effectiveChoice,
	FULLSCREEN,
	GALLERY,
	isChoice,
	PRESETS
} from './arrangement.ts';
import type { Arrangement, Theme } from './theme/types.ts';

/** A theme as far as the layout cares: whether it brings its own. */
function wearing(signature: Arrangement | null): Theme {
	return {
		signature: signature ? { id: 'own', name: { id: 'Milik', en: 'Own' }, arrangement: signature } : null
	} as Theme;
}

const OWN: Arrangement = { ...CLASSIC, nowPlaying: 'instrument', position: 'dial', plates: true };

test('a theme with a layout of its own is worn in it until somebody chooses otherwise', () => {
	assert.equal(effectiveChoice(null, wearing(OWN)), 'signature');
	assert.deepEqual(arrangementFor(null, wearing(OWN)), OWN);
	assert.deepEqual(arrangementFor('signature', wearing(OWN)), OWN);
});

test('every standard preset can be worn with every theme', () => {
	for (const theme of [wearing(OWN), wearing(null), null]) {
		assert.deepEqual(arrangementFor('classic', theme), CLASSIC);
		assert.deepEqual(arrangementFor('gallery', theme), GALLERY);
		assert.deepEqual(arrangementFor('fullscreen', theme), FULLSCREEN);
	}
});

test('a theme without a layout of its own falls back to Klasik, never to nothing', () => {
	assert.equal(effectiveChoice('signature', wearing(null)), 'classic');
	assert.deepEqual(arrangementFor('signature', wearing(null)), CLASSIC);
	assert.deepEqual(arrangementFor('something-old', wearing(null)), CLASSIC);
	assert.deepEqual(arrangementFor(undefined, null), CLASSIC);
});

test('in a standard preset a theme keeps its look but not its arrangement', () => {
	// The arrangement comes from the preset alone; nothing of the theme's
	// own layout leaks into it.
	assert.notDeepEqual(arrangementFor('classic', wearing(OWN)), OWN);
	assert.equal(arrangementFor('classic', wearing(OWN)).position, 'bar');
});

test('the presets are what the specification describes', () => {
	assert.deepEqual([...PRESETS], ['classic', 'gallery', 'fullscreen']);
	// Klasik is the window as Onsa has always drawn it.
	assert.equal(CLASSIC.sidebar, 'shown');
	assert.equal(CLASSIC.transport, 'full');
	assert.equal(CLASSIC.signal, true);
	// Galeri: a grid of covers and one player bar.
	assert.equal(GALLERY.libraryHome, 'albums');
	assert.equal(GALLERY.transport, 'bar');
	// Layar penuh: the picture large with the words, the controls minimal.
	assert.equal(FULLSCREEN.nowPlaying, 'immersive');
	assert.equal(FULLSCREEN.transport, 'minimal');
	assert.equal(FULLSCREEN.panels.lyrics, true);
	assert.equal(FULLSCREEN.panels.cover, true);
});

test('only the known choices are choices', () => {
	for (const word of ['classic', 'gallery', 'fullscreen', 'signature']) assert.ok(isChoice(word));
	for (const word of ['kokpit-tua', '', null, 3]) assert.ok(!isChoice(word));
});
