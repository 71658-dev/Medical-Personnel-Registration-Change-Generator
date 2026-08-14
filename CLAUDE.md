# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

醫事人員執業異動文字產生器 — a single-page Rust/Yew WASM app (client-side only, no backend) that assembles a Traditional Chinese official-document subject line from three inputs: 申請人姓名 + 申請類別 (single select) + 申請項目 (multi select), then copies it to the clipboard. Deployed as a static PWA to GitHub Pages.

All UI strings are Traditional Chinese (zh-TW); keep new user-facing text in zh-TW.

## Commands

Build requires the Rust `wasm32-unknown-unknown` target and [Trunk](https://trunkrs.dev):

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk

trunk serve            # dev server with hot reload (http://localhost:8080)
trunk build            # debug build → dist/
trunk build --release  # what CI ships
cargo check --target wasm32-unknown-unknown   # fast type check without Trunk
```

Tests (Playwright, `testDir: ./tests` — note the directory does not exist yet):

```bash
npm install
npx playwright install       # first run only
trunk build                  # REQUIRED: server.js serves dist/, tests 404 without it
npm test                     # all browsers (chromium/firefox/webkit)
npm run test:ui              # interactive runner
npx playwright test tests/x.spec.js --project=chromium -g "test name"   # single test
npm run serve                # serve dist/ on :3000 standalone
```

`playwright.config.js` auto-starts `node server.js`; that server only serves `dist/`, so a stale or missing `dist/` silently means testing old code.

## Architecture

The entire app is `src/main.rs` (~900 lines): one Yew `Component` (`App`) with a flat `Msg` enum, plus `index.html` (Trunk asset manifest) and `style.css` (design tokens + component classes + all layout). `fonts.css` (2 MB of base64 Inter) is still on disk but **no longer linked** from `index.html` — see Design system below.

Key structural facts:

- **Domain data is compile-time constants.** `CATEGORIES` (20 醫事人員 types, each with a `group` used for the filter tab pills) and `ITEMS` (11 異動 types) are `&'static [..]` at the top of `main.rs`. Adding a profession or an application type means editing those arrays and nothing else — the tab list is derived from distinct `group` values at render time.
- **Text generation** lives in `get_generated_text()` → `format!("{}申辦{}{}", name, category, items)`. `clean_parentheses()` strips parenthetical suffixes from labels (`護理師(護士)` → `護理師`) but special-cases `(科別)變更`/`(姓名)變更`, which become `科別變更`/`姓名變更` rather than losing their prefix. Changing label text in the constants can silently change output — check both paths.
- **`placeholder_mode`** switches the same function between live preview (emits `（請輸入姓名）` placeholders) and actual clipboard output (emits empty strings). The copy path validates the three fields itself and toasts on failure.
- **There are two copy paths, and they behave differently on purpose.** The button/Ctrl+Enter path (`Msg::CopyText` → `CopySuccess`) validates, toasts `已複製：`, writes `medgen_history` + `medgen_names`, morphs the button, and refocuses the name input. The automatic path (`schedule_auto_copy` → `Msg::AutoCopy` → `AutoCopySuccess`) fires `AUTO_COPY_DELAY_MS` (600ms) after the last edit once all three fields are filled, toasts `已自動複製：`, and touches **neither** storage key — history is reserved for deliberate copies. `last_copied_text` tracks what is on the clipboard so the auto path never repeats itself or re-copies what the button just copied; every field-mutating `Msg` arm re-arms the debounce, and dropping the stored `Timeout` cancels the pending run. Auto-copy failures are swallowed silently (see the comment in `Msg::AutoCopy`) — browsers that demand a user gesture for `writeText` reject it from a timer callback, and the user never asked for that copy.
- **Single source of truth for state**: `App` holds everything; there is no router, no context, no child components. Every interaction is a `Msg`. Timeouts (`toast_timeout`, `morph_timeout`, `suggestions_timeout`) are stored on the struct so they are cancelled on drop.
- **Persistence** is `gloo_storage::LocalStorage` under two keys: `medgen_history` (last 20 copied strings, deduped, newest first) and `medgen_names` (last 8 names, powering the input's suggestion dropdown). Both are re-validated and truncated on load in `create()` — keep those guards when touching load logic.
- **JS interop** is confined to: async clipboard via `navigator.clipboard.writeText`, a window `keydown` listener for Ctrl/Cmd+Enter, and `beforeinstallprompt`/`appinstalled` for the PWA install button. Nothing touches the DOM outside Yew's vdom.
- **CSS is token-driven** (`:root` in `style.css`); the desktop two-column layout collapses at 800px, below which the sticky `.mobile-bar` provides preview + copy. Class names are string literals in `main.rs` — grep both files when renaming.
- **The 800px breakpoint is duplicated in Rust.** `is_desktop_viewport()` runs `match_media("(min-width: 800px)")` to gate the auto-focus in `rendered()` (focus the name input on first render and after a successful copy, but not on phones — it would pop the virtual keyboard and shove `.mobile-bar` up). If you move the breakpoint in `style.css`, move it there too. That focus call also arms `suppress_focus_suggestions`, since a programmatic `focus()` fires the same event a click does and would otherwise open the 最近使用 dropdown unprompted.

## Design system

`style.css` is a port of the **Modernist** design system from the Claude Design project `f7a5975d-1a4a-481c-ae0b-a8c1c2023ecc` (source of truth: `_ds/modernist-ea58a47d.../styles.css` there). The top of the file is that system's tokens verbatim — flat (`--radius-*: 0`), warm gray ground `#f3f2f2`, red accent ramp on `#ec3013`, 2px dividers, Archivo 800 headings — followed by the DS component classes actually used (`.btn`, `.input`, `.card`, `.tag`, `.nav`, `.hr`) and then app-specific layout classes. Pull design changes from that project rather than retuning tokens ad hoc.

Two deliberate deviations from the upstream DS file:

- Its `@import url('https://fonts.googleapis.com/css2?family=Archivo…')` is **dropped** — `index.html`'s CSP is same-origin only, so the import would be blocked. Archivo stays first in `--font-heading`/`--font-body` and falls back to system-ui plus a CJK stack; since the UI is almost entirely zh-TW, only Latin runs ("STEP 01", "Ctrl") are affected. Self-hosting an Archivo woff2 would restore it.
- Because the stack no longer names Inter, `fonts.css` is unlinked from `index.html` (and dropped from `LARGE_RESOURCES` in `sw.js`) rather than shipping 2 MB of unused base64.

## Deployment

`.github/workflows/gh-pages.yml` builds on push to `main`/`master` with a **hardcoded** `--public-url /Medical-Personnel-Registration-Change-Generator/`. This must match the GitHub repository name or all asset paths 404 in production.

## Offline / service worker

`register-sw.js` (loaded by `index.html`, copied into `dist/` by Trunk) registers `sw.js` on window load. Both are plain JS on purpose — registration should not wait on the WASM module, and the Cache API plumbing would only add interop noise to `main.rs`.

- **`sw.js` caching strategy is split by request type.** Navigations use Network First so a new deployment wins immediately (`index.html` is not fingerprinted, so Cache First would pin users to a stale shell); everything else uses Cache First, which is safe because Trunk fingerprints its output. Cross-origin GETs are passed straight through.
- **The hashed assets are cached via a warm-up message, not the precache list.** `PRECACHE_URLS` cannot name `index-<hash>.js` / `*_bg-<hash>.wasm` / `style-<hash>.css`. On first visit those requests are issued before the worker claims the page, so `register-sw.js` reads `performance.getEntriesByType('resource')` and posts a `WARM_CACHE` message with the same-origin URLs; the worker's `message` handler adds any that are missing. This is what makes the app work offline after one visit rather than two — if you change either side, keep the message contract (`{type: 'WARM_CACHE', urls: []}`) in sync.
- Bump `CACHE_VERSION` in `sw.js` when the precache list or strategy changes (currently `v3`). Old fingerprinted assets from previous deployments linger in the cache until that bump clears it.

`index.html` sets a strict CSP that allows only same-origin resources (plus `wasm-unsafe-eval` for the WASM module) and `data:` URIs for fonts/images — anything pulled from a CDN will be blocked.
