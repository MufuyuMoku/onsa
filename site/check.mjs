#!/usr/bin/env node
// Checks the built site (site/dist) before it is published:
//   - every relative link and image points at a file that is there;
//   - links that leave the site go only to Onsa's own GitHub pages;
//   - no script, no third-party resource, nothing that could track;
//   - every page offers all three languages, and the download button
//     points at the latest release, never at a version;
//   - the site never calls Onsa open source.
//
//   node site/build.mjs && node site/check.mjs

import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { LANGUAGES, LINKS, REPO, SITE } from './config.mjs';

const dist = join(dirname(fileURLToPath(import.meta.url)), 'dist');
const problems = [];
const say = (page, problem) => problems.push(`${relative(dist, page)}: ${problem}`);

const pages = LANGUAGES.map(({ path }) => join(dist, path, 'index.html'));
for (const page of pages) {
	if (!existsSync(page)) {
		say(page, 'missing');
		continue;
	}
	const html = readFileSync(page, 'utf8');
	const here = dirname(page);

	if (/<script\b/i.test(html)) say(page, 'has a script');
	if (/<iframe\b/i.test(html)) say(page, 'has an iframe');
	if (/open[\s-]?source|sumber terbuka|オープンソース/i.test(html)) say(page, 'calls Onsa open source');

	const addresses = [
		...[...html.matchAll(/\b(?:href|src)="([^"]+)"/g)].map((m) => m[1]),
		...[...html.matchAll(/\bsrcset="([^"]+)"/g)].flatMap((m) => m[1].split(',').map((c) => c.trim().split(/\s+/)[0]))
	];
	for (const address of addresses) {
		if (/^(mailto:|#)/.test(address)) continue;
		if (/^https?:\/\//.test(address)) {
			const ours = address.startsWith(`${REPO}/`) || address === REPO || address.startsWith(SITE);
			if (!ours) say(page, `links outside Onsa: ${address}`);
			continue;
		}
		const target = resolve(here, address.split('#')[0]);
		const file = existsSync(target) && statSync(target).isDirectory() ? join(target, 'index.html') : target;
		if (!existsSync(file)) say(page, `points at nothing: ${address}`);
	}

	// Stylesheets and fonts: our own only.
	if (/@import|fonts\.googleapis|fonts\.gstatic|cdn\./i.test(html)) say(page, 'loads something from elsewhere');

	for (const { code } of LANGUAGES) {
		if (!new RegExp(`hreflang="${code}"[^>]*>[^<]+</a>`).test(html) && !new RegExp(`<a [^>]*hreflang="${code}"`).test(html))
			say(page, `no link to the ${code} page`);
	}
	const downloads = [...html.matchAll(/class="button primary" href="([^"]+)"/g)].map((m) => m[1]);
	if (!downloads.length) say(page, 'no download button');
	for (const href of downloads) if (href !== LINKS.latest) say(page, `download button points at ${href}`);
	if (/\b\d+\.\d+\.\d+\b/.test(html.replace(/v1\.[67]/g, ''))) say(page, 'a version number is written in the page');
}

const css = readFileSync(join(dist, 'site.css'), 'utf8');
if (/url\(\s*['"]?https?:/i.test(css) || /@import/i.test(css)) problems.push('site.css loads something from elsewhere');
for (const [, file] of css.matchAll(/url\('([^']+)'\)/g)) {
	if (!existsSync(join(dist, file))) problems.push(`site.css points at nothing: ${file}`);
}
const everything = (dir) =>
	readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
		entry.isDirectory() ? everything(join(dir, entry.name)) : [join(dir, entry.name)]
	);
for (const file of everything(dist)) {
	if (/(^|[\\/])(LICEN[CS]E|COPYING)(\.\w+)?$/i.test(file)) problems.push(`a licence file is published: ${relative(dist, file)}`);
}

if (problems.length) {
	for (const problem of problems) console.error(`FAIL ${problem}`);
	process.exit(1);
}
console.log(`ok: ${pages.length} pages, every link and image checked`);
