#!/usr/bin/env node
// Builds the Onsa site: one page per language, from one template and one
// text file per language. No dependencies, nothing fetched.
//
//   node site/build.mjs        writes site/dist/
//
// Text lives in site/i18n/<lang>.json; the page in site/src/page.html;
// images, fonts and the icon in site/public/. A key missing from any
// language stops the build, so no page ever ships half translated.

import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';
import { LANGUAGES, LINKS, SITE } from './config.mjs';

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, 'dist');

/** The six built-in themes, in the order Onsa lists them. */
const THEMES = ['kokpit-tua', 'laut-dalam', 'jendela-biru', 'lorong-kuning', 'siang-mimpi', 'krom-saku'];

const read = (file) => readFileSync(join(here, file), 'utf8');
const texts = Object.fromEntries(LANGUAGES.map(({ code }) => [code, JSON.parse(read(`i18n/${code}.json`))]));
const template = read('src/page.html');

const escape = (text) =>
	String(text).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' })[c]);

/** A value by dotted path, or a build error naming the language and key. */
function get(code, key) {
	let value = texts[code];
	for (const part of key.split('.')) value = value?.[part];
	if (value === undefined) throw new Error(`i18n/${code}.json has no "${key}"`);
	return value;
}

/** Every key the English text has must be in the others too, and the
 *  other way round. */
function sameKeys() {
	const keys = (value, prefix = '') =>
		value && typeof value === 'object' && !Array.isArray(value)
			? Object.entries(value).flatMap(([k, v]) => keys(v, prefix ? `${prefix}.${k}` : k))
			: [`${prefix}${Array.isArray(value) ? `[${value.length}]` : ''}`];
	const reference = new Set(keys(texts.en));
	for (const { code } of LANGUAGES) {
		const own = new Set(keys(texts[code]));
		const missing = [...reference].filter((k) => !own.has(k));
		const extra = [...own].filter((k) => !reference.has(k));
		if (missing.length || extra.length)
			throw new Error(`i18n/${code}.json differs from en.json: missing ${missing.join(', ') || '-'}; extra ${extra.join(', ') || '-'}`);
	}
}

// --- the blocks the template asks for ----------------------------------------

/** Paths from a page to the site's root. */
const rootOf = (path) => (path ? '../'.repeat(path.split('/').filter(Boolean).length) : '');

const blocks = {
	switcher(code, path) {
		return LANGUAGES.map((language) => {
			const href = `${rootOf(path)}${language.path}` || './';
			const name = escape(get(language.code, 'meta.languageName'));
			return language.code === code
				? `<li><a href="${href}" aria-current="page" lang="${language.code}" hreflang="${language.code}">${name}</a></li>`
				: `<li><a href="${href}" lang="${language.code}" hreflang="${language.code}">${name}</a></li>`;
		}).join('\n\t\t\t\t');
	},
	alternates(code, path) {
		return [
			`<link rel="canonical" href="${SITE}${path}">`,
			...LANGUAGES.map(
				(language) => `<link rel="alternate" hreflang="${language.code}" href="${SITE}${language.path}">`
			),
			`<link rel="alternate" hreflang="x-default" href="${SITE}">`
		].join('\n\t');
	},
	themes(code, path) {
		const root = rootOf(path);
		return THEMES.map((id) => {
			const theme = get(code, `themes.list.${id}`);
			return `<figure class="theme" id="theme-${id}">
					<a class="shot" href="${root}screens/${id}.png">
						<img src="${root}screens/${id}.png" width="1280" height="800" loading="lazy" decoding="async" alt="${escape(theme.alt)}">
					</a>
					<figcaption>
						<h3>${escape(theme.name)} <span class="layout">${escape(theme.layout)}</span></h3>
						<p>${theme.text}</p>
					</figcaption>
				</figure>`;
		}).join('\n\t\t\t\t');
	},
	features(code) {
		return get(code, 'features.list')
			.map(
				(feature) => `<li class="feature">
						<h3>${escape(feature.title)}</h3>
						<p>${feature.text}</p>
					</li>`
			)
			.join('\n\t\t\t\t\t');
	},
	soon(code) {
		return get(code, 'features.soon.list')
			.map((item) => `<li><span class="tag">${escape(item.when)}</span> ${item.text}</li>`)
			.join('\n\t\t\t\t\t');
	},
	programs(code) {
		return get(code, 'programs.list')
			.map(
				(program) => `<tr>
							<th scope="row"><code>${escape(program.name)}</code></th>
							<td>${program.text}</td>
						</tr>`
			)
			.join('\n\t\t\t\t\t\t');
	},
	faq(code) {
		return get(code, 'faq.list')
			.map(
				(item) => `<details>
					<summary>${escape(item.q)}</summary>
					<div>${item.a}</div>
				</details>`
			)
			.join('\n\t\t\t\t');
	}
};

/** Fills {{key}} with escaped text, {{{key}}} with text that may carry a
 *  little markup (written by us, in the JSON), {{@block}} with a block,
 *  and {{link.name}} / {{root}} with addresses. */
function render(code, path) {
	const links = LINKS;
	return template
		.replace(/\{\{@(\w+)\}\}/g, (_, name) => {
			if (!blocks[name]) throw new Error(`no block "${name}"`);
			return blocks[name](code, path);
		})
		.replace(/\{\{link\.(\w+)\}\}/g, (_, name) => {
			if (!links[name]) throw new Error(`no link "${name}"`);
			return links[name];
		})
		.replace(/\{\{root\}\}/g, rootOf(path))
		.replace(/\{\{site\}\}/g, SITE)
		.replace(/\{\{here\}\}/g, SITE + path)
		.replace(/\{\{lang\}\}/g, code)
		.replace(/\{\{\{([\w.]+)\}\}\}/g, (_, key) => get(code, key))
		.replace(/\{\{([\w.]+)\}\}/g, (_, key) => escape(get(code, key)));
}

sameKeys();
rmSync(dist, { recursive: true, force: true });
mkdirSync(dist, { recursive: true });
cpSync(join(here, 'public'), dist, { recursive: true });
cpSync(join(here, 'src', 'site.css'), join(dist, 'site.css'));
for (const { code, path } of LANGUAGES) {
	const html = render(code, path);
	const left = html.match(/\{\{[^}]*\}\}/);
	if (left) throw new Error(`${code}: "${left[0]}" was not filled`);
	const out = join(dist, path, 'index.html');
	mkdirSync(dirname(out), { recursive: true });
	writeFileSync(out, html);
	console.log(`wrote ${out.slice(here.length + 1)}`);
}
// GitHub Pages would otherwise run the files through Jekyll.
writeFileSync(join(dist, '.nojekyll'), '');
if (!existsSync(join(dist, 'icon.svg'))) throw new Error('public/icon.svg is missing');
