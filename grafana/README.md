# Grafana dashboard

`dashboard.json` is a ready-made dashboard for TimeTable. It uses two datasources:

- **Prometheus**, scraping `GET /api/metrics`: usage counters and subscriber gauges, upstream, server, short
  links.
- **Infinity** (plugin `yesoreyeram-infinity-datasource`), reading the usage log JSON from
  `GET /api/stats/requests` and `GET /api/stats/feed-subscribers`: the tables at the bottom.

Both endpoints need `STATS_TOKEN` (read-only; see `.env.example`). Without it they answer 404.

## 1. TimeTable

Set a long random `STATS_TOKEN` in `.env` (e.g. `openssl rand -hex 32`), different from `AUTH_TOKEN`, and
restart. Keep `FEED_STATS_DB` enabled (the default) for the subscriber panels and the tables.

## 2. Prometheus

```yaml
scrape_configs:
  - job_name: timetable
    scheme: https                    # or http when scraping the container directly
    metrics_path: /api/metrics       # prefix with BASE_PATH if set, e.g. /timetable/api/metrics
    scrape_interval: 60s
    authorization:
      type: Bearer
      credentials: <STATS_TOKEN>     # or credentials_file: /etc/prometheus/timetable_token
    static_configs:
      - targets: ["timetable.example.com"]
```

## 3. Infinity datasource

Install the plugin (`grafana cli plugins install yesoreyeram-infinity-datasource`, or
`GF_INSTALL_PLUGINS=yesoreyeram-infinity-datasource` in Docker), then add a datasource:

- **URL** (base URL): `https://timetable.example.com` (plus `BASE_PATH`, if set). The dashboard queries
  use relative paths such as `/api/stats/requests`.
- **Authentication → Bearer token** (or a custom header `Authorization: Bearer <STATS_TOKEN>`).
- **Allowed hosts**: add the same base URL (Infinity refuses other hosts once a secret is set).

## 4. Import

Grafana → Dashboards → New → Import → upload `dashboard.json`, then choose the Prometheus and Infinity
datasources when asked.

## What you'll see

- **Usage**: feed requests tracked vs anonymous (pie), feed vs api, calendar apps, active subscribers
  (1/7/30 days), all-time and new subscribers, anonymous feeds (lower bound: distinct settings + name + app),
  subscribers by course settings.
- **API usage**: JSON calls from outside the wizard, by client.
- **Upstream**: fetches and errors towards the university sites, p95 duration, lessons in the last fetch
  (0 is a hint that the site changed format).
- **Server**: requests and p95 latency by route, errors, cache hit ratio, Redis status, SQLite file sizes,
  memory/CPU (Linux only), uptime and version.
- **Short links**: stored links, opens by QR vs link, creations.
- **Usage log**: the last 500 requests in the selected time range (empty `track_id` = anonymous; filter with
  the `Log: …` variables) and the subscriber list.

Pie charts and counters come from Prometheus counters, which reset when TimeTable restarts; `increase()`
handles that, but requests made while Prometheus was down only show up in totals, not in time. The
usage log tables read SQLite directly and are exact.
