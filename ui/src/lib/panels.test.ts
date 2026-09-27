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
