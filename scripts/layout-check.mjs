#!/usr/bin/env node
// Checks that every theme lays out cleanly in every layout it offers
// (SPEC M16, criterion 2): nothing overlaps, the page never scrolls sideways,
// and no text is cut off without an ellipsis. It also looks for false
// buttons (criterion 8): a button that does nothing, or something that shows
// the hand of the pointer without being pressable.
//
// It drives a running Onsa through WebView2's debugging port the way a
// listener would: the Appearance page to pick a theme and a layout, the
// sidebar (or its drawer) to move between pages. Nothing outside Onsa's own
// page is touched. Start Onsa with the port open, preferably on a test
// profile with a few tracks playing:
//
//   WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9241 \
//   ONSA_DATA_DIR=<test profile> target/release/onsa.exe
//
// then:  node scripts/layout-check.mjs [--port 9241] [--shots <folder>] [--only <theme>]
//        node scripts/layout-check.mjs --self-test   (plants faults, expects to find them)
//
// The window is sized through the debugging port, so the checks run at the
// sizes below whatever the real window is. Exit code 1 when anything is
// found. Windows only for now: WebKitGTK has no such port.

import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const args = process.argv.slice(2);
const option = (name, fallback) => {
	const at = args.indexOf(name);
	return at >= 0 && args[at + 1] ? args[at + 1] : fallback;
};
const PORT = option('--port', process.env.ONSA_CDP_PORT ?? '9241');
const SHOTS = option('--shots', null);
/** Only the themes whose name contains this, to look again at one. */
const ONLY = option('--only', '').toLowerCase();

/** The window's smallest size (tauri.conf.json) and a common laptop one. */
const SIZES = [
	{ width: 1280, height: 800 },
	{ width: 820, height: 600 }
];
/** The Appearance page's place in the sidebar: eight library pages, then
 *  output, DSP, library, metadata, lyrics, appearance. */
const APPEARANCE_ITEM = 13;
/** Pages each layout is looked at on: by their place in the sidebar. */
const PAGES = [
	{ name: 'now-playing', nowPlaying: true },
	{ name: 'tracks', item: 0 },
	{ name: 'albums', item: 1 },
	{ name: 'appearance', item: APPEARANCE_ITEM }
];

const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

async function connect() {
	const list = await (await fetch(`http://127.0.0.1:${PORT}/json`)).json();
	const page = list.find((target) => target.type === 'page');
	if (!page) throw new Error(`no Onsa page on port ${PORT}`);
	const ws = new WebSocket(page.webSocketDebuggerUrl);
	await new Promise((resolve) => ws.addEventListener('open', resolve, { once: true }));
	let next = 0;
	const pending = new Map();
	ws.addEventListener('message', (event) => {
		const message = JSON.parse(event.data);
		pending.get(message.id)?.(message.result);
		pending.delete(message.id);
	});
	const send = (method, params = {}) =>
		new Promise((resolve) => {
			const id = ++next;
			pending.set(id, resolve);
			ws.send(JSON.stringify({ id, method, params }));
		});
	const evaluate = async (expression) => {
		const answer = await send('Runtime.evaluate', {
			expression,
			returnByValue: true,
			awaitPromise: true,
			userGesture: true
		});
		if (answer?.exceptionDetails) throw new Error(JSON.stringify(answer.exceptionDetails));
		return answer?.result?.value;
	};
	return { ws, send, evaluate };
}

// --- in the page ------------------------------------------------------------

/** Clicks a sidebar entry, through the drawer when the sidebar is hidden. */
const goTo = (item) => `(async () => {
	const entries = () => [...document.querySelectorAll('.sidebar .item:not(.playlist):not(.all)')];
	if (!entries()[${item}]) {
		document.querySelector('.head .icon-btn[aria-expanded]')?.click();
		await new Promise((r) => setTimeout(r, 400));
	}
	const entry = entries()[${item}];
	if (!entry) return false;
	entry.click();
	return true;
})()`;

/** To Now Playing: Ctrl+N toggles it with the track list (SPEC §13). */
const toNowPlaying = `(() => {
	if (document.querySelector('.now')) return true;
	window.dispatchEvent(new KeyboardEvent('keydown', { key: 'n', ctrlKey: true, bubbles: true }));
	return true;
})()`;

/** The chips of the Appearance page's first two sections: themes, layouts. */
const chips = (section) =>
	`[...document.querySelectorAll('.page section')[${section}]?.querySelectorAll('.chip') ?? []]`;

/** The checks themselves; returns a list of findings. */
const inspect = `(() => {
	const found = [];
	const name = (el) => {
		const text = (el.getAttribute('aria-label') || el.textContent || '').trim().replace(/\\s+/g, ' ');
		const cls = typeof el.className === 'string' && el.className ? '.' + el.className.trim().split(/\\s+/).slice(0, 2).join('.') : '';
		return el.tagName.toLowerCase() + cls + (text ? ' "' + text.slice(0, 40) + '"' : '');
	};
	// What of a thing can be seen: its box, cut to every box round it that
	// clips, and to the window. Rows scrolled out of a list are not seen.
	const seen = (el, rect = el.getBoundingClientRect()) => {
		let left = Math.max(rect.left, 0), top = Math.max(rect.top, 0);
		let right = Math.min(rect.right, innerWidth), bottom = Math.min(rect.bottom, innerHeight);
		for (let at = el.parentElement; at && at !== document.body; at = at.parentElement) {
			const style = getComputedStyle(at);
			if (style.overflowX === 'visible' && style.overflowY === 'visible') continue;
			const box = at.getBoundingClientRect();
			left = Math.max(left, box.left); top = Math.max(top, box.top);
			right = Math.min(right, box.right); bottom = Math.min(bottom, box.bottom);
		}
		return right - left >= 1 && bottom - top >= 1 ? { left, top, right, bottom } : null;
	};
	const shown = (el) => {
		const rect = el.getBoundingClientRect();
		if (rect.width < 1 || rect.height < 1 || !seen(el, rect)) return false;
		for (let at = el; at; at = at.parentElement) {
			const style = getComputedStyle(at);
			if (style.visibility === 'hidden' || style.display === 'none' || Number(style.opacity) === 0) return false;
		}
		return true;
	};
	// Inside a scrolled box a thing can be out of view without being clipped.
	const clippedBy = (el) => {
		for (let at = el.parentElement; at && at !== document.body; at = at.parentElement) {
			const style = getComputedStyle(at);
			const x = style.overflowX, y = style.overflowY;
			if (x === 'visible' && y === 'visible') continue;
			return { box: at, scrolls: /auto|scroll/.test(x + y) };
		}
		return null;
	};

	// 1. The page itself never scrolls sideways.
	const root = document.scrollingElement;
	if (root.scrollWidth > root.clientWidth + 1) found.push('page scrolls sideways by ' + (root.scrollWidth - root.clientWidth) + 'px');
	for (const el of document.querySelectorAll('.shell, .shell > *, .content, .content > *')) {
		const style = getComputedStyle(el);
		if (/auto|scroll/.test(style.overflowX) && el.scrollWidth > el.clientWidth + 1)
			found.push('scrolls sideways: ' + name(el) + ' by ' + (el.scrollWidth - el.clientWidth) + 'px');
	}

	// 2. Text cut off without an ellipsis.
	const texts = [...document.querySelectorAll('body *')].filter((el) =>
		[...el.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()) && shown(el));
	for (const el of texts) {
		const style = getComputedStyle(el);
		const ellipsis = style.textOverflow === 'ellipsis' || Number(style.webkitLineClamp) > 0;
		if (style.overflowX !== 'visible' && el.scrollWidth > el.clientWidth + 1 && !ellipsis)
			found.push('text cut off: ' + name(el));
		const clip = clippedBy(el);
		if (clip && !clip.scrolls) {
			const own = el.getBoundingClientRect(), box = clip.box.getBoundingClientRect();
			if (own.right > box.right + 1 || own.left < box.left - 1 || own.bottom > box.bottom + 1 || own.top < box.top - 1) {
				const inner = getComputedStyle(clip.box);
				if (inner.textOverflow !== 'ellipsis') found.push('text cut off by its box: ' + name(el) + ' in ' + name(clip.box));
			}
		}
	}

	// 3. Something the listener can press is covered by something else.
	const controls = [...document.querySelectorAll('button, a[href], input, select, textarea, [role="slider"], [tabindex="0"]')]
		.filter((el) => shown(el) && !el.closest('[inert]'));
	for (const el of controls) {
		const part = seen(el);
		if (!part) continue;
		const x = (part.left + part.right) / 2, y = (part.top + part.bottom) / 2;
		const top = document.elementFromPoint(x, y);
		if (!top || el === top || el.contains(top) || top.contains(el)) continue;
		if (getComputedStyle(top).pointerEvents === 'none') continue;
		found.push('covered: ' + name(el) + ' under ' + name(top));
	}

	// 4. Two pieces of text over each other.
	const leaves = texts.filter((el) => !el.closest('[aria-hidden="true"]') && ![...el.children].some((c) => texts.includes(c)));
	const boxes = leaves.map((el) => {
		const range = document.createRange();
		range.selectNodeContents(el);
		let rect = range.getBoundingClientRect();
		// What runs past a box that clips it (behind an ellipsis) is not drawn.
		const style = getComputedStyle(el);
		if (style.overflowX !== 'visible' || style.overflowY !== 'visible') {
			const own = el.getBoundingClientRect();
			rect = new DOMRect(Math.max(rect.left, own.left), Math.max(rect.top, own.top),
				Math.min(rect.right, own.right) - Math.max(rect.left, own.left),
				Math.min(rect.bottom, own.bottom) - Math.max(rect.top, own.top));
			if (rect.width < 1 || rect.height < 1) return { el, rect: null };
		}
		return { el, rect: seen(el, rect) };
	}).filter((one) => one.rect);
	for (let i = 0; i < boxes.length; i++) {
		for (let j = i + 1; j < boxes.length; j++) {
			const a = boxes[i], b = boxes[j];
			if (a.el.contains(b.el) || b.el.contains(a.el)) continue;
			const w = Math.min(a.rect.right, b.rect.right) - Math.max(a.rect.left, b.rect.left);
			const h = Math.min(a.rect.bottom, b.rect.bottom) - Math.max(a.rect.top, b.rect.top);
			const at = (r) => '[' + [r.left, r.top, r.right, r.bottom].map(Math.round).join(',') + ']';
			if (w > 2 && h > 2) found.push('text over text: ' + name(a.el) + ' ' + at(a.rect) + ' and ' + name(b.el) + ' ' + at(b.rect));
		}
	}

	// 5. Nothing pretends to be pressable (SPEC §9.1: no false buttons). A
	//    button does something: Svelte 5 keeps the handlers it delegates on
	//    the element under a symbol named events (older ones used __click),
	//    a plain one has onclick, a form's button sends the form. And the
	//    hand of the pointer appears only over something that can be
	//    pressed.
	const pressable = 'button, a[href], input, select, textarea, label, summary, [role="button"], [role="slider"], [role="tab"], [role="option"], [role="menuitem"], [tabindex]';
	for (const el of document.querySelectorAll('button')) {
		if (!shown(el) || el.disabled) continue;
		const events = el[Object.getOwnPropertySymbols(el).find((s) => String(s) === 'Symbol(events)')] ?? {};
		const handled = ['click', 'pointerdown', 'mousedown', 'keydown'].some((kind) => events[kind] || el['__' + kind]);
		const acts = handled || el.onclick ||
			(el.type === 'submit' && el.form) || el.closest('label, summary');
		if (!acts) found.push('button that does nothing: ' + name(el));
	}
	for (const el of document.querySelectorAll('body *')) {
		if (getComputedStyle(el).cursor !== 'pointer') continue;
		if (el.parentElement && getComputedStyle(el.parentElement).cursor === 'pointer') continue;
		if (el.matches(pressable) || !shown(el)) continue;
		found.push('looks pressable but is not: ' + name(el));
	}
	return [...new Set(found)];
})()`;

// --- the run -----------------------------------------------------------------

const onsa = await connect();
const shot = async (file) => {
	if (!SHOTS) return;
	mkdirSync(SHOTS, { recursive: true });
	const picture = await onsa.send('Page.captureScreenshot', { format: 'png' });
	writeFileSync(join(SHOTS, `${file}.png`), Buffer.from(picture.data, 'base64'));
};
const toAppearance = async () => {
	await onsa.evaluate(goTo(APPEARANCE_ITEM));
	await wait(600);
};

await onsa.send('Emulation.setDeviceMetricsOverride', { ...SIZES[0], deviceScaleFactor: 1, mobile: false });
await wait(600);

// --self-test: plant one fault of each kind on the page and make sure each
// is found, so a clean run means something.
if (args.includes('--self-test')) {
	await onsa.evaluate(`(() => {
		const plant = document.createElement('div');
		plant.id = 'layout-check-faults';
		plant.innerHTML =
			'<div style="position:absolute;left:0;top:0;width:300vw;height:1px"></div>' +
			'<div style="position:fixed;left:40px;top:40px;width:60px;overflow:hidden;white-space:nowrap">a sentence far too long for its box</div>' +
			'<span style="position:fixed;left:40px;top:120px">one piece of text</span>' +
			'<span style="position:fixed;left:60px;top:122px">another one</span>' +
			'<button type="button" style="position:fixed;left:40px;top:200px;width:80px;height:30px">pressable</button>' +
			'<div style="position:fixed;left:30px;top:190px;width:120px;height:60px;background:red"></div>' +
			'<button type="button" style="position:fixed;left:300px;top:40px">does nothing</button>' +
			'<div style="position:fixed;left:300px;top:100px;cursor:pointer">a sign</div>';
		document.body.append(plant);
	})()`);
	await wait(200);
	const found = await onsa.evaluate(inspect);
	await onsa.evaluate(`document.getElementById('layout-check-faults')?.remove()`);
	await onsa.send('Emulation.clearDeviceMetricsOverride');
	onsa.ws.close();
	const kinds = ['scrolls sideways', 'text cut off', 'text over text', 'covered', 'button that does nothing', 'looks pressable but is not'];
	let missed = 0;
	for (const kind of kinds) {
		const hit = found.some((one) => one.includes(kind));
		if (!hit) missed += 1;
		console.log(`${hit ? 'found ' : 'MISSED'}  ${kind}`);
	}
	process.exit(missed ? 1 : 0);
}

/** The theme and the layout the listener had, to give back at the end. */
const PRESET_ORDER = ['classic', 'gallery', 'fullscreen'];
const remember = async () => {
	await toAppearance();
	const theme = await onsa.evaluate(`${chips(0)}.findIndex((c) => c.getAttribute('aria-pressed') === 'true')`);
	const layout = await onsa.evaluate(
		`window.__TAURI_INTERNALS__.invoke('settings_get').then((s) => s?.display?.layout ?? null, () => null)`
	);
	return { theme, layout };
};
const giveBack = async ({ theme, layout }) => {
	await toAppearance();
	if (theme >= 0) await onsa.evaluate(`${chips(0)}[${theme}]?.click()`);
	await wait(600);
	if (layout) {
		const own = (await onsa.evaluate(`${chips(1)}.length`)) > PRESET_ORDER.length ? 1 : 0;
		const at = layout === 'signature' ? 0 : PRESET_ORDER.indexOf(layout) + own;
		await onsa.evaluate(`${chips(1)}[${at}]?.click()`);
		await wait(400);
	}
};

const before = await remember();
const themes = await onsa.evaluate(`JSON.stringify(${chips(0)}.map((c) => c.textContent.trim()))`);
let findings = 0;
let looked = 0;
for (const [themeAt, theme] of JSON.parse(themes).entries()) {
	if (ONLY && !theme.toLowerCase().includes(ONLY)) continue;
	await toAppearance();
	await onsa.evaluate(`${chips(0)}[${themeAt}]?.click()`);
	await wait(800);
	const layouts = JSON.parse(await onsa.evaluate(`JSON.stringify(${chips(1)}.map((c) => c.textContent.trim()))`));
	for (const [layoutAt, layout] of layouts.entries()) {
		for (const size of SIZES) {
			await onsa.send('Emulation.setDeviceMetricsOverride', { ...size, deviceScaleFactor: 1, mobile: false });
			await wait(300);
			await toAppearance();
			await onsa.evaluate(`${chips(1)}[${layoutAt}]?.click()`);
			await wait(700);
			for (const page of PAGES) {
				await onsa.evaluate(page.nowPlaying ? toNowPlaying : goTo(page.item));
				await wait(1200);
				const where = `${theme} · ${layout} · ${page.name} · ${size.width}×${size.height}`;
				const found = await onsa.evaluate(inspect);
				looked += 1;
				if (found.length) {
					findings += found.length;
					console.log(`\n${where}`);
					for (const one of found) console.log(`  - ${one}`);
				} else {
					console.log(`ok  ${where}`);
				}
				const slug = (s) => s.toLowerCase().normalize('NFKD').replace(/[^a-z0-9]+/g, '-');
				await shot(`${slug(theme)}_${slug(layout)}_${page.name}_${size.width}`);
			}
		}
	}
}
await onsa.send('Emulation.setDeviceMetricsOverride', { ...SIZES[0], deviceScaleFactor: 1, mobile: false });
await giveBack(before);
await onsa.send('Emulation.clearDeviceMetricsOverride');
onsa.ws.close();
console.log(`\n${looked} views looked at, ${findings} findings.`);
process.exit(findings ? 1 : 0);
