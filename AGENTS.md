# AGENTS.md

Guidelines and hints for AI coding agents working in this repository.
For what lives where and how the pieces fit together, read [STRUCTURE.md](STRUCTURE.md) first.

## Workflow

- Run commands from the repo root through the Makefile (`make help` lists them). `make dev` starts backend
  (`:8080`) and Astro (`:4321`, hot reload, `/api` proxied).
- **Always run `make check` before declaring work done.** It runs `cargo fmt --check`, `clippy -D warnings`,
  `cargo test`, `astro check` and `svelte-check`, and must pass with zero warnings.
- Makefile recipes must work in both `sh` and `cmd.exe` (GNU make on Windows falls back to `cmd.exe`): only
  `cd dir && command`, no inline `VAR=x cmd`, no `rm`/`cp`/`mkdir -p` — use `node -e` for file operations.
- Every new env var goes in `backend/src/config/` **and** is documented in `.env.example` (and, if relevant
  for deployment, in `docker-compose.yml`).
- Don't commit or push unless asked. Never commit `.env`.

## Invariants — don't break these

- **Admin endpoints** (`/api/admin/...`) take the `AdminAuth` extractor (`api/auth.rs`) and stay
  disabled (404) without `AUTH_TOKEN`. Never log or return the token: keep it in `config::Secret` and
  compare it only through `AdminAuth`.
- **Redis must never fail a request.** Any Redis error or timeout degrades to a cache miss; the server must
  start without Redis.
- **Bump the cache key version** (`KEY_PREFIX` in `cache/mod.rs`, `timetable:v1:`) whenever a cached payload shape
  changes (fields added/renamed in `Lesson`, `SelectOption`, …).
- **Base path placeholder:** `base` in `frontend/astro.config.mjs` and `BASE_PLACEHOLDER` in
  `backend/src/static_files/mod.rs` must stay identical.
- **The frontend is schema-driven:** never hard-code university-specific fields or parameters in the frontend.
  If a university needs something new, extend the schema types (`backend/src/models/` + `lib/api.ts`) generically.
- **Cache keys only contain validated, declared params** (see `state/validation.rs`). Don't feed raw query strings into
  keys or upstream requests.
- **Lesson IDs must be stable** across fetches (they become ICS `UID`s): derive them from the source's IDs,
  prefixed with the university id.
- Astro stays **static only**: no SSR, no adapter, no Node server at runtime.
- When backend API models change, update the TypeScript types in `frontend/src/lib/api.ts` in the same change.

## Adding a university

1. Create `backend/src/universities/<id>/` with a `mod.rs` implementing `University` (`info`, `options`,
   `lessons`; override `timezone` if not `Europe/Rome`), the wizard schema in `wizard.rs` and the
   fetching/parsing split in submodules (see `unicam/`). Put a module doc comment at the top of `mod.rs` documenting the upstream endpoints
   and response formats.
2. Group related fields in the same step when they belong on one wizard screen.
3. Register it in `Registry::new` in `universities/registry.rs`.
4. Add unit tests for parsing with fixtures copied from real responses.

Validation, caching, API and wizard follow the schema: nothing else should need to change.

## Rust hints

- Return `AppError`, built from the error kinds in `errors/` (`NotFound::new(..).into()`, `?` converts
  automatically); use `Upstream::new(context, err)` for anything coming from a university site. A new
  error kind gets its own file in `errors/`, implements `ApiError` and becomes an `AppError` variant.
  No `unwrap()`/`expect()` on external data.
- Use the shared `reqwest::Client` in `AppState` (built in `startup/http_client.rs`); never build clients per request.
- Static regexes via `std::sync::LazyLock`.
- Dependencies are recent majors whose APIs differ from most online examples — check the docs for the
  installed version: **reqwest 0.13** (TLS feature is `rustls`, `.query()` needs the `query` feature),
  **redis 1.x**, **axum 0.8** (`{param}` path syntax, `nest()` doesn't match trailing slash),
  **tower-http 0.7**, **moka 0.12** (`future` feature).
- Log with `tracing`, not `println!`.
- Keep modules small and focused: one folder per area with a `mod.rs` that declares submodules and
  re-exports the public items; split a file when it grows several responsibilities.

## Frontend hints

- Svelte 5 runes only (`$state`, `$derived`, `$effect`, `$props`); no legacy stores or `export let`.
- Tailwind 4: reusable classes are `@utility` in `styles/global.css` (`@apply` can't reference classes defined
  in `@layer components`).
- Always build URLs with `BASE` from `lib/api.ts`; never hard-code `/api` or `/`.
- Every UI string goes in `lib/i18n.ts` in both languages (IT is the default); schema text comes localized
  from the backend as `Text`.
- **TypeScript is pinned to 6.x**: `astro check` doesn't support TypeScript 7 yet. Don't upgrade it.
- Static files are loaded by the backend once at startup: **restart the backend after rebuilding the frontend**
  when testing through `:8080`.

## Testing

- Backend unit tests live next to the code (`#[cfg(test)]`); test parsing against fixtures, not live requests.
- For end-to-end checks, run the backend with a built frontend and drive it with a headless browser
  (Playwright with the system Chrome/Edge; install `playwright-core` in a scratch dir, not in the repo).
- Docker isn't available on every dev machine: if you couldn't run `docker compose`, say so instead of
  claiming the image works.

## Windows gotchas

- The main dev machine is Windows 11; shells are PowerShell and Git Bash.
- **Git Bash rewrites env values that look like paths**: `BASE_PATH=/timetable ./timetable.exe` receives
  `C:/Program Files/Git/timetable`. Prefix with `MSYS2_ENV_CONV_EXCL='*'`, or set it in `.env`.
- A running `timetable.exe` locks the binary: stop it before `cargo build`.
- rust-analyzer must point at `backend/Cargo.toml` (`rust-analyzer.linkedProjects`), otherwise it creates a
  stray `target/` in the repo root.
