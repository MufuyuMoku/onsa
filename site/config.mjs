// Addresses and languages of the Onsa site, in one place.

/** Where the site itself is published (GitHub Pages for this repository). */
export const SITE = 'https://mufuyumoku.github.io/onsa/';

/** Where every download and source link points. No version number: the
 *  download button goes to whatever release is the latest. */
export const REPO = 'https://github.com/MufuyuMoku/onsa';
export const LINKS = {
	latest: `${REPO}/releases/latest`,
	releases: `${REPO}/releases`,
	repo: REPO,
	issues: `${REPO}/issues`
};

/** The languages, in the order the switcher shows them. The first is the
 *  site's root; the others live in a folder of their own. */
export const LANGUAGES = [
	{ code: 'en', path: '' },
	{ code: 'id', path: 'id/' },
	{ code: 'ja', path: 'ja/' }
];
