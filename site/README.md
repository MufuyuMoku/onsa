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

## Images to replace

The files in `public/` are placeholders with their name and size written on them.
Replace each with a file of the **same name and size**:

| File | Size | What it shows |
|---|---|---|
| `public/screens/kokpit-tua.png` | 1280 × 800 | Kokpit Tua in its own layout, Panel Instrumen (also the picture at the top of the page) |
| `public/screens/laut-dalam.png` | 1280 × 800 | Laut Dalam, Menyelam |
| `public/screens/jendela-biru.png` | 1280 × 800 | Jendela Biru, Jendela |
| `public/screens/lorong-kuning.png` | 1280 × 800 | Lorong Kuning, Lorong |
| `public/screens/siang-mimpi.png` | 1280 × 800 | Siang Mimpi, Padang |
| `public/screens/krom-saku.png` | 1280 × 800 | Krom Saku, Pemutar Saku |
| `public/og.png` | 1200 × 630 | The picture shown when a link to the site is shared |
| `public/icon.svg` | square, any size (SVG) | The icon: a copy of the placeholder `assets/icon.svg` |

## Fonts

`public/fonts/` holds the two fonts of the Kokpit Tua theme, Saira Stencil One and B612
Mono, with their SIL Open Font Licence files, as the licence requires when the fonts are
passed on. Japanese text uses the reader's system fonts.
