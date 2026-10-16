#!/usr/bin/env node
// Checks what moves for its look alone (SPEC M16, criterion 5): in every
// theme's own layout, every animation stops in power saving and under
// reduced motion, and nothing changes faster than once a second.
//
// Like layout-check.mjs it drives a running Onsa through WebView2's
// debugging port and touches nothing outside Onsa's page; start the same
// test build the same way (see the top of layout-check.mjs), with a track
// playing so that whatever turns while a song plays
// is turning. Power saving is switched on and back off through the Output
// page's own checkbox; reduced motion is emulated by the debugging port.
//
//   node scripts/motion-check.mjs [--port 9241]

const args = process.argv.slice(2);
const at = args.indexOf('--port');
const PORT = at >= 0 ? args[at + 1] : (process.env.ONSA_CDP_PORT ?? '9241');

/** Places in the sidebar: eight library pages, then the settings pages. */
const TRACKS_ITEM = 0;
const OUTPUT_ITEM = 8;
const APPEARANCE_ITEM = 13;
/** Anything that changes more often than this is a blink (WCAG 2.3.1 is
 *  three a second; the concept allows one). */
const SLOWEST_BLINK_MS = 1000;

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

const goTo = (item) => `(async () => {
	const entries = () => [...document.querySelectorAll('.sidebar .item:not(.playlist):not(.all)')];
	if (!entries()[${item}]) {
		document.querySelector('.head .icon-btn[aria-expanded]')?.click();
		await new Promise((r) => setTimeout(r, 400));
	}
	entries()[${item}]?.click();
})()`;
const toNowPlaying = `(() => {
	if (document.querySelector('.now')) return;
	window.dispatchEvent(new KeyboardEvent('keydown', { key: 'n', ctrlKey: true, bubbles: true }));
})()`;
const chips = (section) =>
	`[...document.querySelectorAll('.page section')[${section}]?.querySelectorAll('.chip') ?? []]`;

/** The CSS animations running now (transitions follow the sound, and are
 *  not decoration): what they move, how often, and what they change. */
const running = `(() => document.getAnimations()
	.filter((a) => a.constructor.name === 'CSSAnimation' && a.playState === 'running')
	.map((a) => {
		const el = a.effect.target;
		const timing = a.effect.getComputedTiming();
		const changes = [...new Set(a.effect.getKeyframes().flatMap((k) => Object.keys(k))
			.filter((k) => !['offset', 'computedOffset', 'easing', 'composite'].includes(k)))];
		const cls = typeof el.className === 'string' ? el.className.trim().split(/\\s+/)[0] : el.className?.baseVal;
		return {
			what: el.tagName.toLowerCase() + (cls ? '.' + cls : '') + ' @' + a.animationName,
			ms: Number(timing.duration),
			changes,
			drifts: Boolean(el.closest('.drifts'))
		};
	}))()`;

const onsa = await connect();
/** Power saving through the Output page's own checkbox — the last one
 *  with a note under it — and whether the interface took it up. */
const setPowerSave = async (on) => {
	await onsa.evaluate(goTo(OUTPUT_ITEM));
	await wait(700);
	const took = await onsa.evaluate(`(async () => {
		const box = [...document.querySelectorAll('.page label.check:has(+ p.note) input[type=checkbox]')].at(-1);
		if (!box) return false;
		if (box.checked !== ${on}) box.click();
		await new Promise((r) => setTimeout(r, 500));
		return document.documentElement.dataset.onsaStill === String(${on});
	})()`);
	if (!took) throw new Error(`power saving did not turn ${on ? 'on' : 'off'}`);
};

// What turns while a song plays only turns then: a run without one would
// pass for the wrong reason.
const state = await onsa.evaluate(
	`window.__TAURI_INTERNALS__.invoke('player_snapshot').then((s) => s?.state ?? null, () => null)`
);
if (state !== 'playing') {
	console.error(`nothing is playing (${state}); start a track and run again`);
	onsa.ws.close();
	process.exit(2);
}

const toAppearance = async () => {
	await onsa.evaluate(goTo(APPEARANCE_ITEM));
	await wait(600);
};
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

await onsa.send('Emulation.setDeviceMetricsOverride', { width: 1280, height: 800, deviceScaleFactor: 1, mobile: false });
const before = await remember();
const themes = JSON.parse(await onsa.evaluate(`JSON.stringify(${chips(0)}.map((c) => c.textContent.trim()))`));
let faults = 0;
for (const [themeAt, theme] of themes.entries()) {
	await onsa.evaluate(goTo(APPEARANCE_ITEM));
	await wait(600);
	await onsa.evaluate(`${chips(0)}[${themeAt}]?.click()`);
	await wait(800);
	for (const page of ['now-playing', 'tracks']) {
		await onsa.evaluate(page === 'now-playing' ? toNowPlaying : goTo(TRACKS_ITEM));
		await wait(1500);
		const moving = await onsa.evaluate(running);
		console.log(`\n${theme} · ${page}: ${moving.length} animation(s)`);
		for (const one of moving) {
			const blink = one.ms < SLOWEST_BLINK_MS;
			const loose = !one.drifts;
			if (blink || loose) faults += 1;
			console.log(
				`  ${blink || loose ? 'FAULT' : 'ok   '} ${one.what} every ${one.ms} ms, changes ${one.changes.join(', ')}` +
					(blink ? ' — faster than once a second' : '') +
					(loose ? ' — not marked as decoration, so nothing stills it' : '')
			);
		}
		// Reduced motion.
		await onsa.send('Emulation.setEmulatedMedia', { features: [{ name: 'prefers-reduced-motion', value: 'reduce' }] });
		await wait(500);
		const reduced = await onsa.evaluate(running);
		await onsa.send('Emulation.setEmulatedMedia', { features: [] });
		console.log(`  reduced motion: ${reduced.length ? 'STILL MOVING ' + reduced.map((one) => one.what).join(', ') : 'all still'}`);
		faults += reduced.length;
		// Power saving, switched on as a listener would, then back off.
		await setPowerSave(true);
		await onsa.evaluate(page === 'now-playing' ? toNowPlaying : goTo(TRACKS_ITEM));
		await wait(1200);
		const saving = await onsa.evaluate(running);
		await setPowerSave(false);
		console.log(`  power saving:   ${saving.length ? 'STILL MOVING ' + saving.map((one) => one.what).join(', ') : 'all still'}`);
		faults += saving.length;
	}
}
await giveBack(before);
await onsa.send('Emulation.clearDeviceMetricsOverride');
onsa.ws.close();
console.log(`\n${faults} fault(s).`);
process.exit(faults ? 1 : 0);
