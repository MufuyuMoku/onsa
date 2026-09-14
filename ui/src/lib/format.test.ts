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

import { clock, fileName } from './format.ts';

test('a Windows path gives up its file name', () => {
	assert.equal(fileName('C:\\Users\\sorar\\Music\\Lilith\\BANG BANG.mp3'), 'BANG BANG.mp3');
	assert.equal(fileName('D:\\音楽\\夜明け.flac'), '夜明け.flac');
});

test('a Linux path gives up its file name', () => {
	assert.equal(fileName('/home/sorar/Music/Lilith/BANG BANG.mp3'), 'BANG BANG.mp3');
	assert.equal(fileName('/home/sorar/音楽/夜明け.flac'), '夜明け.flac');
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
