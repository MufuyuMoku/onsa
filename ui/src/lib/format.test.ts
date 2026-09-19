/**
 * The formatting helpers, and in particular the one that takes a file name
 * out of a path.
 *
 * That one has now been written wrong twice, both times by losing a
 * backslash on its way into the file, and both times it looked right until
 * a Windows path went through it. It is worth a test of its own.
 */

import assert from 'node:assert/strict';
import { test } from 'node:test';

import { clock, fileName, relativeTo } from './format.ts';

test('a Windows path gives up its file name', () => {
	assert.equal(fileName('C:\\Users\\rian\\Music\\Lilith\\BANG BANG.mp3'), 'BANG BANG.mp3');
	assert.equal(fileName('D:\\音楽\\夜明け.flac'), '夜明け.flac');
});

test('a Linux path gives up its file name', () => {
	assert.equal(fileName('/home/rian/Music/Lilith/BANG BANG.mp3'), 'BANG BANG.mp3');
	assert.equal(fileName('/home/rian/音楽/夜明け.flac'), '夜明け.flac');
});

test('a path with both kinds of separator still ends at the last one', () => {
	assert.equal(fileName('C:\\Music/pop\\02 Lampu Kota.wav'), '02 Lampu Kota.wav');
});

test('something that is already a file name is left alone', () => {
	assert.equal(fileName('02 Lampu Kota.wav'), '02 Lampu Kota.wav');
	assert.equal(fileName(''), '');
});

test('a clock reads as minutes and seconds, and says nothing when it cannot', () => {
	assert.equal(clock(0), '0:00');
	assert.equal(clock(61), '1:01');
	assert.equal(clock(3661), '1:01:01');
	assert.equal(clock(null), '--:--');
});

test('a path inside a folder is shown from that folder', () => {
	assert.equal(relativeTo(String.raw`C:\Music\pop\a.flac`, String.raw`C:\Music`), String.raw`pop\a.flac`);
	assert.equal(relativeTo('/music/pop/a.flac', '/music'), 'pop/a.flac');
	// A root written with a trailing separator is still the same root.
	assert.equal(relativeTo(String.raw`C:\Music\a.flac`, 'C:\\Music\\'), 'a.flac');
});

test('a path outside the folder is left whole', () => {
	assert.equal(relativeTo('/other/a.flac', '/music'), '/other/a.flac');
	assert.equal(relativeTo('/music', '/music'), '/music', 'the folder itself');
	assert.equal(relativeTo('/music/a.flac', ''), '/music/a.flac', 'with no root at all');
});
