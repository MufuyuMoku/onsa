# The Onsa site

The promotional page for Onsa, published at <https://mufuyumoku.github.io/onsa/> by
`.github/workflows/site.yml` whenever something under `site/` changes on `main`. It has
nothing to do with the application's own code.

Plain HTML and CSS, put together by a small Node script with no dependencies, and no
JavaScript in the pages themselves: no analytics, no trackers, no cookies, and nothing
loaded from anywhere else.

## Building it

```sh
node site/build.mjs     # writes site/dist/
node site/check.mjs     # links, images, languages, download button
```

To look at it, serve `site/dist/` with any static server, for example
`python -m http.server 4173 --directory site/dist`.

## Where things are

| What | Where |
|---|---|
| Text, one file per language | `i18n/en.json` (the root page), `i18n/id.json` (`/id/`), `i18n/ja.json` (`/ja/`) |
| The page | `src/page.html` |
| The look | `src/site.css` (colours and fonts of the Kokpit Tua theme) |
| Addresses: site, repository, download | `config.mjs` |
| Images, icon, fonts | `public/` |

Every language file must have the same keys; the build stops and names the missing one
otherwise. Text inside `{{{ }}}` in the template, and the `text`, `a` and `intro` fields
of the lists, may hold a little HTML (`<code>`, `<em>`, `<strong>`, links).

The download button always points at `releases/latest` on GitHub, so a new release needs
no change here. No version number is written in the pages; `check.mjs` fails if one is.

## Images

The six screenshots and `og.png` were taken from a test build of v1.5.0 with a fresh
profile holding only the synthetic sample library (no real songs, no real cover art), on
Now Playing, each theme in its own layout. The audio device's name was replaced on the page
with "Speakers" before each capture, because the real one belongs to the machine it was
taken on. Nothing else was changed. A new screenshot should follow the same rules: sample
library only, no folder path, account name, machine name or device name in the picture.

| File | Size | What it shows |
|---|---|---|
| `public/screens/kokpit-tua.png` | 1280 × 800 | Kokpit Tua in its own layout, Panel Instrumen (also the picture at the top of the page) |
| `public/screens/laut-dalam.png` | 1280 × 800 | Laut Dalam, Menyelam |
| `public/screens/jendela-biru.png` | 1280 × 800 | Jendela Biru, Jendela |
| `public/screens/lorong-kuning.png` | 1280 × 800 | Lorong Kuning, Lorong |
| `public/screens/siang-mimpi.png` | 1280 × 800 | Siang Mimpi, Padang |
| `public/screens/krom-saku.png` | 1280 × 800 | Krom Saku, Pemutar Saku |
| `public/og.png` | 1200 × 630 | The picture shown when a link to the site is shared: the Kokpit Tua screenshot, cropped |
| `public/icon.svg` | square, any size (SVG) | The icon: still a copy of the placeholder `assets/icon.svg`, until the final icon is chosen |

## Fonts

`public/fonts/` holds the two fonts of the Kokpit Tua theme, Saira Stencil One and B612
Mono, with their SIL Open Font Licence files, as the licence requires when the fonts are
passed on. Japanese text uses the reader's system fonts.
