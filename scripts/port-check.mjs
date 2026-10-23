#!/usr/bin/env node
// Starts an Onsa binary with a debugging port asked for in its environment,
// and says whether the port answered (see src-tauri/src/inspection.rs).
//
//   node scripts/port-check.mjs <onsa binary> closed   a release build: the
//        port must never answer, and the log must say it was refused
//   node scripts/port-check.mjs <onsa binary> open     a test-harness build:
//        the port must answer, which shows the check above can see a port
//
// Both ways are asked for at once: WebView2's argument on Windows, and
// WebKitGTK's inspector server on Linux. The binary runs on a throw-away
// profile (ONSA_DATA_DIR) and is stopped at the end. On Linux run it under
// a display, e.g. `xvfb-run -a node scripts/port-check.mjs …`.

import { execFileSync, spawn } from 'node:child_process';
import { existsSync, mkdtempSync, readdirSync, readFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';

const [binary, expect] = process.argv.slice(2);
if (!binary || !['open', 'closed'].includes(expect)) {
	console.error('usage: port-check.mjs <onsa binary> open|closed');
	process.exit(2);
}

const PORT = 9555;
const WAIT_MS = 40_000;
const profile = mkdtempSync(join(tmpdir(), 'onsa-port-check-'));
const child = spawn(binary, [], {
	env: {
		...process.env,
		ONSA_DATA_DIR: profile,
		// A web view browser left from an earlier run that shares the data
		// folder would be reused with its own arguments, and this run's would
		// be ignored: each run has a folder of its own.
		WEBVIEW2_USER_DATA_FOLDER: join(profile, 'webview'),
		WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${PORT}`,
		WEBKIT_INSPECTOR_HTTP_SERVER: `127.0.0.1:${PORT}`,
		// A test build opens behind whatever has the focus; a release build
		// ignores this.
		ONSA_TEST_BACKGROUND: '1'
	},
	stdio: 'ignore'
});
let exited = null;
child.on('exit', (code) => (exited = code ?? -1));

const wait = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const answers = async () => {
	for (const path of ['/json', '/']) {
		try {
			const response = await fetch(`http://127.0.0.1:${PORT}${path}`, { signal: AbortSignal.timeout(2000) });
			if (response.ok) return true;
		} catch {
			// Nothing there.
		}
	}
	return false;
};
const log = () => {
	const dir = join(profile, 'logs');
	if (!existsSync(dir)) return '';
	return readdirSync(dir)
		.map((file) => readFileSync(join(dir, file), 'utf8'))
		.join('\n');
};

let answered = false;
const until = Date.now() + WAIT_MS;
while (Date.now() < until && exited === null) {
	if (await answers()) {
		answered = true;
		break;
	}
	// A release build is given the whole time to open it, if it would.
	await wait(1000);
}
// Onsa writes its log without blocking, so a port that answers quickly can
// be ahead of the log on disk: give the log a moment to catch up.
for (let tries = 0; tries < 10 && exited === null && !log().includes('Onsa starting'); tries++) {
	await wait(1000);
}
const said = log();
// Whether it stopped on its own, before this script stops it.
const early = exited;
// What the machine looks like while it is still running, for a failure to
// be read by: is there a web view at all, and what is listening.
const look = (file, args) => {
	try {
		return execFileSync(file, args, { encoding: 'utf8', timeout: 15_000 });
	} catch (error) {
		return `(${file}: ${String(error.message).split('\n')[0]})`;
	}
};
const runtimeKey = [
	'HKLM',
	'SOFTWARE',
	'WOW6432Node',
	'Microsoft',
	'EdgeUpdate',
	'Clients',
	'{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
].join(String.fromCharCode(92));
const machine =
	process.platform === 'win32'
		? [
				look('netstat', ['-ano', '-p', 'tcp'])
					.split('\n')
					.filter((line) => line.includes('LISTENING'))
					.join('\n'),
				look('reg', ['query', runtimeKey, '/v', 'pv']),
				// Whether the argument reached the web view at all, and whether
				// a policy on the machine turns remote debugging off.
				look('powershell', [
					'-NoProfile',
					'-Command',
					"Get-CimInstance Win32_Process -Filter \"Name='msedgewebview2.exe'\" | Where-Object { $_.CommandLine -match 'webview-exe-name=onsa.exe' -and $_.CommandLine -notmatch '--type=' } | ForEach-Object { $_.CommandLine }"
				]),
				look('reg', ['query', ['HKLM', 'SOFTWARE', 'Policies', 'Microsoft', 'Edge'].join(String.fromCharCode(92)), '/s'])
			].join('\n')
		: look('sh', ['-c', 'ps -eo pid,comm | grep -i -E "webkit|onsa"; ss -ltn']);
// With the web view's own processes, which outlive Onsa otherwise; only
// this Onsa's tree, never anybody else's web views.
if (process.platform === 'win32') look('taskkill', ['/PID', String(child.pid), '/T', '/F']);
else child.kill();
await wait(1500);
try {
	rmSync(profile, { recursive: true, force: true });
} catch {
	// Still held by the process on its way out; it is a temporary folder.
}

const problems = [];
if (!said.includes('Onsa starting')) problems.push('Onsa never got as far as starting');
if (early !== null && !answered) problems.push(`Onsa exited on its own (code ${early})`);
if (expect === 'closed') {
	if (answered) problems.push(`a release build answered on port ${PORT}`);
	if (!said.includes('debugging port refused')) problems.push('the log does not say the port was refused');
} else if (!answered) {
	problems.push(`a test build did not answer on port ${PORT}, so a closed port would prove nothing`);
}

if (problems.length) {
	for (const problem of problems) console.error(`FAIL: ${problem}`);
	console.error(said.split('\n').slice(-30).join('\n'));
	console.error('--- the machine while Onsa ran');
	console.error(machine);
	process.exit(1);
}
console.log(expect === 'closed' ? `ok: port ${PORT} stayed closed, and the log says why` : `ok: port ${PORT} answered in the test build`);
