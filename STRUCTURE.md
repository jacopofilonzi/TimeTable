# Project structure

TimeTable turns university lesson timetables into subscribable iCalendar (`.ics`) feeds. A step-by-step wizard
(frontend) helps users build the feed URL; the backend crawls the university website, caches the result and
serves both the API and the built frontend on a single port, over plain HTTP (HTTPS is a reverse proxy's job).

```text
repo/
├── backend/            Rust 2024, axum 0.8, tokio — API, crawlers, cache, static file server
├── frontend/           Astro 7 (static output only) + Svelte 5 (runes) + Tailwind 4, pnpm
├── Dockerfile          stage 1 frontend → /www, stage 2 backend → /app, stage 3 distroless runtime
├── docker-compose.yml  example deployment: service + Redis (internal network)
├── .env.example        every supported env var, documented; `.env` is git-ignored
└── Makefile            dev commands (run `make help`)
```

## Backend (`backend/src`)

Every folder has a `mod.rs` that declares its submodules and re-exports the public items.

```text
backend/src/
├── main.rs                   wiring: .env, config, `healthcheck` subcommand, state, server
├── startup/                  process setup used by main
│   ├── env_file.rs           loads `.env`; warns on a bad line (key only, never the value)
│   ├── logging.rs            tracing subscriber (RUST_LOG, LOG_FORMAT)
│   ├── http_client.rs        shared reqwest client (user agent, timeouts)
│   ├── frontend.rs           loads the built frontend, or API-only mode
│   ├── short_links.rs        opens the short link database (exits if it can't be written)
│   ├── healthcheck.rs        `timetable healthcheck` (Docker HEALTHCHECK, the image has no curl)
│   └── shutdown.rs           Ctrl+C / SIGTERM for graceful shutdown
├── config/                   `Config::from_env`, read once; empty env values count as unset
│   ├── env.rs                env var helpers
│   ├── base_path.rs          BASE_PATH normalization
│   ├── static_dir.rs         STATIC_DIR, or /www, then ../frontend/dist
│   ├── shortlink_db.rs       SHORTLINK_DB: path, `false`, or data/shortlinks.db
│   ├── log_format.rs         LogFormat
│   └── secret.rs             `Secret` (AUTH_TOKEN), redacted in `Debug`
├── errors/                   `AppError` + `ApiError` trait → JSON `{error, message}`
│   ├── not_found.rs          404
│   ├── bad_request.rs        400
│   ├── unauthorized.rs       401 (admin endpoints)
│   ├── upstream.rs           502, university site failed (details logged, not exposed)
│   └── internal.rs           500 (details logged, not exposed)
├── models/                   API data: `Lesson` + wizard schema
│   ├── lesson.rs
│   ├── text.rs               `Text`: plain or `{ it, en }`
│   ├── university_info.rs    `UniversityInfo`, `WeeksRange`
│   ├── step.rs
│   ├── field.rs              `Field`, `FieldKind`
│   └── select_option.rs
├── api/                      router (`/api/...` + static fallback), compression, tracing
│   ├── auth.rs               `AdminAuth` extractor: Bearer AUTH_TOKEN (404 when unset)
│   ├── base_path.rs          mounts everything under BASE_PATH
│   └── handlers/             health, universities, options, lessons, lessons_ics, short_links, admin
├── state/                    `AppState`: validation + caching around crawlers
│   ├── options.rs            `AppState::options`
│   ├── lessons.rs            `AppState::lessons` (+ query validation, `weeks` parsing)
│   ├── short_links.rs        create/resolve short links (validation, page query, cached lookups)
│   └── validation.rs         schema-driven checks, canonical cache-key params, reserved names test
├── cache/                    `Cache::get_or_fetch`
│   ├── entry.rs              in-memory entries with per-entry TTL (moka)
│   └── redis_layer.rs        optional Redis layer, background connection, prefix clear
├── shortlinks/               `ShortLinks`: SQLite store of short links (design in `mod.rs` doc comment)
│   ├── code.rs               codes: Crockford base32 of SHA-256, normalization
│   └── store.rs              SQL: table, insert-or-get with collision handling, lookup, touch
├── ics/                      `ics::render`, RFC 5545 calendar
│   └── format.rs             timestamps, escaping, 75-octet line folding
├── static_files/             built frontend served from memory
│   ├── file.rs               one file: placeholder replacement, ETag/304, cache headers
│   └── fs.rs                 directory walk, FNV-1a hash
└── universities/             crawlers
    ├── university.rs         `University` trait, `Params`
    ├── registry.rs           `Registry`: the list of supported universities
    ├── date_range.rs         `DateRange` (current weeks, in the university timezone)
    └── unicam/               Università di Camerino (upstream formats in `mod.rs` doc comment)
        ├── wizard.rs         wizard schema (steps, fields, year options, weeks range)
        ├── courses.rs        course list (scraped HTML)
        ├── lessons.rs        lessons (JSON endpoint)
        └── html.rs           description parsing, HTML entities
```

### API

All under `{BASE_PATH}/api`:

- `GET /health` → `{status, redis: disabled|connected|disconnected}`
- `DELETE /admin/cache` (header `Authorization: Bearer <AUTH_TOKEN>`) → `{memory, redis: {status, …}}`:
  empties the in-memory cache and deletes the `timetable:*` keys from Redis (never `FLUSHDB`). Without
  `AUTH_TOKEN` it answers 404; a wrong token answers 401 after a 500 ms delay.
- `GET /universities`, `GET /universities/{id}` → wizard schema
- `GET /universities/{id}/options/{field}?<deps>` → `SelectOption[]`
- `GET /universities/{id}/lessons?<params>&weeks=N` → `{timezone, from, to, lessons}`
- `GET /universities/{id}/lessons.ics?<params>&weeks=N&name=…` → iCalendar feed
- `POST /short` `{uni, params, weeks}` → `{code}`: validated like `lessons`; 404 when short links are disabled

Outside `/api` (still under `BASE_PATH`), also as `/S/…` for uppercase QR codes:

- `GET /s/{code}` → 307 to `/?<page query>&step=result` (`/en/…` when `Accept-Language` prefers English)
- `GET /s/{code}.ics?name=…` → the iCalendar feed of the link (not exposed in the UI yet)

Unknown codes (or short links disabled) get the site's 404 page.

### Wizard schema

Each university declares its own wizard in `info()`: a list of **steps**, each with one or more **fields**.
A field's `key` is the query parameter name. Kinds:

- `FieldKind::Select { options, default }` — options known up-front, embedded in the schema (e.g. unicam's year).
- `FieldKind::RemoteSelect { searchable }` — options fetched via `University::options()`; `depends_on` lists
  fields whose values are needed (sent as query params; the frontend reloads when they change). `pattern`
  is a server-side regex sanity check (not serialized).

`state/validation.rs` validates generically from the schema: every field is required; static values must be among the
options; remote values must match `pattern` and be among the (cached) remote options, so bogus values never
reach the upstream site nor create cache entries. Only declared params enter cache keys. The frontend renders
whatever the schema says.

User-facing strings in the schema are `Text`: either a plain string or `{ it, en }`.

### Cache

`Cache::get_or_fetch(key, ttl, fetch)`:

- moka layer: always on, coalesces concurrent misses for the same key (one upstream fetch), per-entry TTL.
- Redis layer: enabled by `REDIS_URL`; connects in the background and retries every 15 s, so the server starts
  even if Redis is down. Every Redis error/timeout (500 ms) degrades to a miss. On a Redis hit, the in-memory
  TTL is the remaining Redis TTL.
- Errors are never cached. Keys are prefixed `timetable:v1:`.
- Lessons keys include the Monday the window starts on (`DateRange`, computed in the university's timezone),
  so a cached week never leaks into the next one. TTLs: `CACHE_LESSONS_TTL` (6 h), `CACHE_OPTIONS_TTL` (24 h).
- Short link lookups (`short:{code}` → page query) are cached for 30 days: codes never change once created.

### Short links

`shortlinks/` stores `id → canonical page query` in SQLite (`SHORTLINK_DB`, default `data/shortlinks.db`
relative to the working dir, `/data/shortlinks.db` in the image; `false` disables them). Unlike Redis it is not a
cache: when enabled, the server creates the file and its directory and exits if it can't write there.

- Table `short_links(id, canonical UNIQUE, created_at, last_used_at)`, unix timestamps. No purge job (yet).
- The canonical query is `uni=…&<fields sorted by key>&weeks=N`, URL-encoded and validated like the lessons
  endpoints. The calendar name is not stored: the page regenerates the default one.
- The id is the first 40 bits of SHA-256(canonical) in Crockford base32 (8 chars, `0-9A-Z` without `ILOU`), so
  the same settings always get the same code. If it is taken by another query the code grows one character at a
  time. Lookups are case-insensitive and read `O` as `0`, `I`/`L` as `1`.
- `last_used_at` is updated in the background at most once a day per link.

### Base path

`BASE_PATH` is applied at runtime, so one build/image works under any prefix:

- The frontend production build uses `base: '/__TT_BASE__'` (`frontend/astro.config.mjs`); the backend
  replaces that placeholder in every text file (`BASE_PLACEHOLDER` in `static_files/mod.rs`) while loading them
  into memory.
- Routing strips the prefix in `api/base_path.rs` (axum 0.8's `nest()` does not match `/prefix/` with a
  trailing slash). `/` and `/prefix` redirect (308) to `/prefix/`.

## Frontend (`frontend/src`)

| File | Responsibility |
| --- | --- |
| `pages/index.astro`, `pages/en/index.astro` | IT (default) and EN pages, both mount `Wizard` with `client:load` |
| `pages/404.astro` | not-found page, served by the backend for unknown paths |
| `pages/admin.astro` | admin tools (`noindex`, not linked anywhere): cache reset with the `AUTH_TOKEN` |
| `components/AdminCache.svelte` | token form calling `DELETE /api/admin/cache`, shows what was removed |
| `layouts/Layout.astro` | page shell: grey page, white card, footer |
| `components/Wizard.svelte` | wizard state machine (university → schema steps → options → result) and two-column layout: sidebar with logo, steps, summary, language switch; content on the right. Mirrors its progress in the URL |
| `lib/wizard-url.ts` | wizard progress ↔ URL query: serialization, validation of a shared URL (labels from schema / remote options), furthest reachable step |
| `components/Stepper.svelte` | vertical step list in the sidebar; completed steps are clickable |
| `components/FieldInput.svelte` | renders a schema field: chips (≤ 8 static options), `OptionList`, or remote-loaded list; in a step mixing a list and chips, the chips become a side column |
| `components/OptionList.svelte` | searchable, grouped listbox (accent-insensitive search on label, hint, group) |
| `components/ResultStep.svelte` | Google / Apple Calendar buttons; other apps and formats (webcal, Outlook, ".ics URL" modal with copy + download); Share and preview in modals |
| `components/ShareLink.svelte` | Share modal: creates the short link (`POST /api/short`), shows it with copy + QR; on failure a generic error and the full page URL |
| `components/CopyField.svelte` | dark code block with a Copy button (falls back to selecting the text) |
| `components/Preview.svelte` | current-week calendar grid (Mon–Fri, weekend if used), overlapping lessons side by side, colour per subject, click for details; times in the university timezone |
| `components/Modal.svelte` | generic `<dialog>` wrapper; content is mounted only while open (lazy loading) |
| `components/QrCode.svelte` | QR code of a URL; `qrcode` is dynamically imported (separate chunk) |
| `components/Icon.svelte` | inline stroke icons (Lucide paths) |
| `lib/api.ts` | API types (mirror `backend/src/models/`), fetch helpers, `BASE`, `icsUrl()`, `createShortLink()`, `shortUrl()` / `shortUrlForQr()` |
| `lib/i18n.ts` | IT/EN dictionaries (`en` is typed against `it`: a missing key is a type error) |
| `styles/global.css` | Tailwind theme tokens and `@utility` classes (`btn-*`, `input`, `code-block`) |

Wizard progress lives only in the URL (no browser storage), so links are shareable and switching language or
reloading keeps it: `?uni=<id>&<field key>=<value>…&weeks=N&name=…&step=<step id>`. `name` is there only if the
user edited it; `step` is `university`, a schema step `id`, `options` or `result`. A shared URL lands on the
requested step, or on the furthest one whose previous steps are complete. Changing step adds a history entry
(`pushState`, Back works); other changes replace it. The short link popup's QR carries the link in uppercase
(`shortUrlForQr`: origin, `/S/` and code) so it fits the QR alphanumeric mode.

In dev, `astro.config.mjs` proxies `/api` and `/s/` (`/S/`) to the backend, reading `PORT` from the root `.env`.
