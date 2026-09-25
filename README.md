# TimeTable

Subscribe to your university's lesson timetable from any iCal-compatible calendar (Google, Apple, Outlook, …).
A step-by-step wizard builds the `.ics` link for you.

```
repo/
├── frontend/   Astro + Svelte 5 + Tailwind, built to static files
├── backend/    Rust (axum): API, crawlers, cache, serves the frontend
├── Dockerfile  frontend → /www, backend → /app, distroless runtime
└── docker-compose.yml  service + Redis
```

## Development

Requirements: Rust (stable), Node 22+, pnpm, GNU make.

```sh
make install   # creates .env, installs dependencies
make dev       # backend on :8080 + Astro on http://localhost:4321 (hot reload, /api proxied)
make check     # fmt, clippy, tests, astro/svelte type checks
make run       # production-like: build the frontend, run the release backend serving it
make help      # everything else
```

The backend reads `.env` from the repo root (see [.env.example](.env.example)). Without `REDIS_URL` it uses
an in-memory cache only; `make redis-up` starts a local Redis if you want to test with it.

## Deployment

```sh
cp .env.example .env   # set REDIS_PASSWORD, optionally BASE_PATH / PUBLISH_PORT
docker compose up -d --build
```

The service speaks plain HTTP: put a reverse proxy in front of it for HTTPS. `BASE_PATH` (e.g. `/timetable`)
is applied at startup, so the same image works under any prefix.

## API

| Endpoint | Description |
| --- | --- |
| `GET /api/universities` | Supported universities with their wizard schema (steps, fields, weeks range) |
| `GET /api/universities/{id}/options/{field}?…` | Options of a `remote_select` field (dependencies as query params) |
| `GET /api/universities/{id}/lessons?…&weeks=N` | Lessons as JSON, from Monday of the current week for `N` weeks |
| `GET /api/universities/{id}/lessons.ics?…&weeks=N&name=…` | Same, as an iCalendar feed |
| `GET /api/health` | Health and Redis status |
| `DELETE /api/admin/cache` | Clears the cache (memory + Redis); needs `Authorization: Bearer <AUTH_TOKEN>` |

The cache can also be cleared from the `/admin/` page (not linked): enter the `AUTH_TOKEN` set in `.env`.
Without `AUTH_TOKEN` the admin endpoints are disabled.

## Adding a university

1. Create `backend/src/universities/<id>/mod.rs` implementing the `University` trait (see `unicam/`):
   - `info()` describes the wizard: steps, and in each step the fields (static `Select` options or
     `RemoteSelect` fetched through `options()`), plus the allowed `weeks` range;
   - `options()` fetches remote options, `lessons()` fetches lessons for a date range.
2. Register it in `Registry::new` in [backend/src/universities/registry.rs](backend/src/universities/registry.rs).

Validation, caching and the frontend wizard are driven by the schema, so nothing else needs to change.
