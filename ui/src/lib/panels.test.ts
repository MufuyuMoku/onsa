/**
 * What is stored about the panels, and what is not. A panel that comes back
 * showing after Onsa is closed and opened again is the whole point of M14;
 * a map that fills up with copies of the defaults is how that stops being
 * true the next time the defaults change.
 */

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { needsAnalysis, panelShown, shownPanels, withPanel, withPanels } from './panels.ts';

test('a panel nobody has decided about shows', () => {
	assert.equal(panelShown(undefined, 'lyrics'), true);
	assert.equal(panelShown({}, 'visualizer'), true);
});

test('a panel that was turned off stays off', () => {
	const stored = withPanel(undefined, 'lyrics', false);
	assert.deepEqual(stored, { lyrics: false });
	assert.equal(panelShown(stored, 'lyrics'), false);
	assert.equal(panelShown(stored, 'cover'), true, 'and the others are untouched');
});

test('a panel put back the way it started leaves the record', () => {
	const off = withPanel(undefined, 'cover', false);
	const on = withPanel(off, 'cover', true);
	assert.deepEqual(on, {}, 'what is stored is a record of choices, not of defaults');
});

test('several panels can be decided at once, which is what a layout is', () => {
	const stored = withPanels({ lyrics: false }, { cover: false, lyrics: true, details: false });
	assert.deepEqual(stored, { cover: false, details: false });
});

test('the panels showing come back in the order they appear', () => {
	assert.deepEqual(shownPanels({}), ['cover', 'details', 'lyrics', 'visualizer']);
	assert.deepEqual(shownPanels({ details: false, cover: false }), ['lyrics', 'visualizer']);
});

test('hiding the visualizer is what stops the analysis', () => {
	assert.equal(needsAnalysis({}), true);
	assert.equal(needsAnalysis({ lyrics: false }), true, 'the other panels cost no sound');
	assert.equal(needsAnalysis({ visualizer: false }), false);
});

test('a layout may start with fewer panels, and the choices made in it are its own', () => {
	const immersive = { cover: true, lyrics: true, details: false, visualizer: false };
	// Nobody decided anything in it yet: the layout's word stands.
	assert.equal(panelShown(undefined, 'visualizer', immersive), false);
	assert.equal(panelShown(undefined, 'lyrics', immersive), true);
	// Turned on there, it is recorded against the layout's own start.
	const on = withPanel(undefined, 'visualizer', true, immersive);
	assert.deepEqual(on, { visualizer: true });
	assert.equal(panelShown(on, 'visualizer', immersive), true);
	// And turned back off, the record empties again.
	assert.deepEqual(withPanel(on, 'visualizer', false, immersive), {});
});
