//! SQL on the usage tables. Everything here is synchronous: callers run it on a blocking thread
//! (or the writer thread).

use rusqlite::{Connection, params};
use serde::Serialize;

use super::RequestEvent;

/// Enables WAL (readers such as `sqlite3` or backups don't block the writer), creates the tables,
/// then writes and drops a scratch table so that a database we can't write fails now.
pub fn init(conn: &Connection) -> rusqlite::Result<()> {
    // Returns the new mode (`memory` for in-memory test databases).
    conn.pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get::<_, String>(0))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS requests (
             id          INTEGER PRIMARY KEY,
             at          INTEGER NOT NULL,
             kind        TEXT NOT NULL,
             endpoint    TEXT NOT NULL,
             track_id    TEXT,
             short_code  TEXT,
             uni         TEXT,
             params      TEXT,
             weeks       INTEGER,
             name        TEXT,
             client      TEXT NOT NULL,
             user_agent  TEXT,
             status      INTEGER NOT NULL
         ) STRICT;
         CREATE INDEX IF NOT EXISTS requests_at ON requests (at);
         CREATE INDEX IF NOT EXISTS requests_kind_at ON requests (kind, at);
         CREATE INDEX IF NOT EXISTS requests_track_id ON requests (track_id, at);
         CREATE TABLE IF NOT EXISTS feed_subscribers (
             track_id      TEXT PRIMARY KEY,
             uni           TEXT NOT NULL,
             first_seen    INTEGER NOT NULL,
             last_seen     INTEGER NOT NULL,
             request_count INTEGER NOT NULL,
             last_client   TEXT NOT NULL,
             last_params   TEXT
         ) STRICT;
         CREATE INDEX IF NOT EXISTS feed_subscribers_last_seen ON feed_subscribers (last_seen);
         BEGIN IMMEDIATE;
         CREATE TABLE write_check (x INTEGER);
         DROP TABLE write_check;
         COMMIT;",
    )
}

/// Writes a batch of requests (and the subscriber updates they imply) in one transaction.
pub fn insert(conn: &mut Connection, events: &[RequestEvent]) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    {
        let mut request = tx.prepare_cached(
            "INSERT INTO requests (at, kind, endpoint, track_id, short_code, uni, params, weeks,
                                   name, client, user_agent, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        )?;
        let mut subscriber = tx.prepare_cached(
            "INSERT INTO feed_subscribers (track_id, uni, first_seen, last_seen, request_count,
                                           last_client, last_params)
             VALUES (?1, ?2, ?3, ?3, 1, ?4, ?5)
             ON CONFLICT (track_id) DO UPDATE SET
                 uni = excluded.uni,
                 last_seen = MAX(last_seen, excluded.last_seen),
                 request_count = request_count + 1,
                 last_client = excluded.last_client,
                 last_params = excluded.last_params",
        )?;
        for e in events {
            request.execute(params![
                e.at,
                e.kind.as_str(),
                e.endpoint,
                e.track_id,
                e.short_code,
                e.university,
                e.params,
                e.weeks,
                e.name,
                e.client.as_str(),
                e.user_agent,
                e.status,
            ])?;
            if let (true, Some(track_id), Some(uni)) =
                (e.is_subscriber_fetch(), &e.track_id, e.university)
            {
                subscriber.execute(params![track_id, uni, e.at, e.client.as_str(), e.params])?;
            }
        }
    }
    tx.commit()
}

/// Deletes requests older than `before` (unix seconds); returns how many.
pub fn purge(conn: &Connection, before: i64) -> rusqlite::Result<usize> {
    conn.execute("DELETE FROM requests WHERE at < ?1", [before])
}

/// Unix seconds as an RFC 3339 UTC timestamp (`2026-09-29T08:14:00Z`).
fn rfc3339<S: serde::Serializer>(at: &i64, serializer: S) -> Result<S::Ok, S::Error> {
    match chrono::DateTime::from_timestamp(*at, 0) {
        Some(t) => serializer.serialize_str(&t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        None => serializer.serialize_i64(*at),
    }
}

/// Filters of [`requests`]; `None` means "any".
#[derive(Debug, Default)]
pub struct RequestFilter {
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub kind: Option<&'static str>,
    pub track_id: Option<String>,
    pub anonymous: Option<bool>,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct RequestRow {
    pub id: i64,
    #[serde(serialize_with = "rfc3339")]
    pub at: i64,
    pub kind: String,
    pub endpoint: String,
    pub track_id: Option<String>,
    pub short_code: Option<String>,
    pub university: Option<String>,
    pub params: Option<String>,
    pub weeks: Option<i64>,
    pub name: Option<String>,
    pub client: String,
    pub user_agent: Option<String>,
    pub status: i64,
}

/// Matching requests, newest first.
pub fn requests(conn: &Connection, f: &RequestFilter) -> rusqlite::Result<Vec<RequestRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, at, kind, endpoint, track_id, short_code, uni, params, weeks, name, client,
                user_agent, status
         FROM requests
         WHERE (?1 IS NULL OR at >= ?1)
           AND (?2 IS NULL OR at < ?2)
           AND (?3 IS NULL OR kind = ?3)
           AND (?4 IS NULL OR track_id = ?4)
           AND (?5 IS NULL OR (track_id IS NULL) = ?5)
         ORDER BY at DESC, id DESC
         LIMIT ?6",
    )?;
    let rows = stmt.query_map(
        params![f.from, f.to, f.kind, f.track_id, f.anonymous, f.limit],
        |r| {
            Ok(RequestRow {
                id: r.get(0)?,
                at: r.get(1)?,
                kind: r.get(2)?,
                endpoint: r.get(3)?,
                track_id: r.get(4)?,
                short_code: r.get(5)?,
                university: r.get(6)?,
                params: r.get(7)?,
                weeks: r.get(8)?,
                name: r.get(9)?,
                client: r.get(10)?,
                user_agent: r.get(11)?,
                status: r.get(12)?,
            })
        },
    )?;
    rows.collect()
}

#[derive(Debug, Serialize)]
pub struct SubscriberRow {
    pub track_id: String,
    pub university: String,
    #[serde(serialize_with = "rfc3339")]
    pub first_seen: i64,
    #[serde(serialize_with = "rfc3339")]
    pub last_seen: i64,
    pub request_count: i64,
    pub last_client: String,
    pub last_params: Option<String>,
}

/// Subscribers seen since `since` (any if `None`), most recently seen first.
pub fn subscribers(
    conn: &Connection,
    since: Option<i64>,
    limit: u32,
) -> rusqlite::Result<Vec<SubscriberRow>> {
    let mut stmt = conn.prepare_cached(
        "SELECT track_id, uni, first_seen, last_seen, request_count, last_client, last_params
         FROM feed_subscribers
         WHERE (?1 IS NULL OR last_seen >= ?1)
         ORDER BY last_seen DESC
         LIMIT ?2",
    )?;
    let rows = stmt.query_map(params![since, limit], |r| {
        Ok(SubscriberRow {
            track_id: r.get(0)?,
            university: r.get(1)?,
            first_seen: r.get(2)?,
            last_seen: r.get(3)?,
            request_count: r.get(4)?,
            last_client: r.get(5)?,
            last_params: r.get(6)?,
        })
    })?;
    rows.collect()
}

/// Aggregates exported as Prometheus gauges.
#[derive(Debug, Clone, Default)]
pub struct Summary {
    /// `(university, count)` of every track id ever seen.
    pub subscribers: Vec<(String, i64)>,
    /// `(university, window, count)` of track ids seen within the window.
    pub active: Vec<(String, &'static str, i64)>,
    /// `(university, params, count)` of track ids seen in the last 7 days.
    pub by_params: Vec<(String, String, i64)>,
    /// `(university, window, count)` of distinct anonymous feeds within the window.
    pub anonymous: Vec<(String, &'static str, i64)>,
}

/// Windows of the "active" gauges, in seconds.
pub const WINDOWS: [(&str, i64); 3] = [("1d", 86_400), ("7d", 7 * 86_400), ("30d", 30 * 86_400)];

pub fn summary(conn: &Connection, now: i64) -> rusqlite::Result<Summary> {
    fn grouped(
        conn: &Connection,
        sql: &str,
        params: impl rusqlite::Params,
    ) -> rusqlite::Result<Vec<(String, i64)>> {
        let mut stmt = conn.prepare_cached(sql)?;
        let rows = stmt.query_map(params, |r| Ok((r.get(0)?, r.get(1)?)))?;
        rows.collect()
    }

    let mut summary = Summary {
        subscribers: grouped(
            conn,
            "SELECT uni, COUNT(*) FROM feed_subscribers GROUP BY uni",
            [],
        )?,
        ..Summary::default()
    };
    for (window, seconds) in WINDOWS {
        let since = [now - seconds];
        for (uni, count) in grouped(
            conn,
            "SELECT uni, COUNT(*) FROM feed_subscribers WHERE last_seen >= ?1 GROUP BY uni",
            since,
        )? {
            summary.active.push((uni, window, count));
        }
        // An anonymous feed is identified as well as we can: settings + custom name + client.
        // Two people with the same course, default name and app count once: a lower bound.
        for (uni, count) in grouped(
            conn,
            "SELECT uni, COUNT(*) FROM (
                 SELECT DISTINCT uni, params, weeks, name, client FROM requests
                 WHERE kind = 'feed' AND track_id IS NULL AND status < 400 AND uni IS NOT NULL
                   AND at >= ?1
             ) GROUP BY uni",
            since,
        )? {
            summary.anonymous.push((uni, window, count));
        }
    }

    let mut stmt = conn.prepare_cached(
        "SELECT uni, last_params, COUNT(*) FROM feed_subscribers
         WHERE last_seen >= ?1 AND last_params IS NOT NULL
         GROUP BY uni, last_params",
    )?;
    let rows = stmt.query_map([now - WINDOWS[1].1], |r| {
        Ok((r.get(0)?, r.get(1)?, r.get(2)?))
    })?;
    summary.by_params = rows.collect::<rusqlite::Result<_>>()?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::{Client, Kind};

    fn event(at: i64, kind: Kind, track_id: Option<&str>, status: u16) -> RequestEvent {
        RequestEvent {
            at,
            kind,
            endpoint: "lessons.ics".into(),
            track_id: track_id.map(str::to_string),
            short_code: None,
            university: Some("unicam"),
            params: Some("course=1&year=2".into()),
            weeks: Some(4),
            name: None,
            client: Client::Google,
            user_agent: Some("Google-Calendar-Importer".into()),
            status,
        }
    }

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        conn
    }

    #[test]
    fn records_requests_and_subscribers() {
        let mut conn = db();
        let now = 1_000_000;
        insert(
            &mut conn,
            &[
                event(now - 100, Kind::Feed, Some("aaaaaaaa"), 200),
                event(now - 50, Kind::Feed, Some("aaaaaaaa"), 200),
                // Failed feeds and API calls don't make subscribers.
                event(now - 40, Kind::Feed, Some("bbbbbbbb"), 502),
                event(now - 30, Kind::Api, Some("cccccccc"), 200),
                event(now - 20, Kind::Feed, None, 200),
            ],
        )
        .unwrap();

        let all = requests(
            &conn,
            &RequestFilter {
                limit: 100,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0].at, now - 20, "newest first");
        assert_eq!(all[0].track_id, None);

        let anonymous = requests(
            &conn,
            &RequestFilter {
                anonymous: Some(true),
                limit: 100,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(anonymous.len(), 1);

        let api = requests(
            &conn,
            &RequestFilter {
                kind: Some("api"),
                limit: 100,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(api.len(), 1);

        let subs = subscribers(&conn, None, 100).unwrap();
        assert_eq!(subs.len(), 1);
        assert_eq!(subs[0].track_id, "aaaaaaaa");
        assert_eq!(subs[0].request_count, 2);
        assert_eq!(subs[0].first_seen, now - 100);
        assert_eq!(subs[0].last_seen, now - 50);

        let summary = summary(&conn, now).unwrap();
        assert_eq!(summary.subscribers, vec![("unicam".to_string(), 1)]);
        assert!(summary.active.contains(&("unicam".to_string(), "1d", 1)));
        assert!(summary.anonymous.contains(&("unicam".to_string(), "7d", 1)));
        assert_eq!(
            summary.by_params,
            vec![("unicam".to_string(), "course=1&year=2".to_string(), 1)]
        );
    }

    #[test]
    fn purges_old_requests_only() {
        let mut conn = db();
        insert(
            &mut conn,
            &[
                event(100, Kind::Feed, Some("aaaaaaaa"), 200),
                event(200, Kind::Feed, None, 200),
            ],
        )
        .unwrap();
        assert_eq!(purge(&conn, 150).unwrap(), 1);
        let left = requests(
            &conn,
            &RequestFilter {
                limit: 10,
                ..Default::default()
            },
        )
        .unwrap();
        assert_eq!(left.len(), 1);
        // Subscribers are kept forever.
        assert_eq!(subscribers(&conn, None, 10).unwrap().len(), 1);
    }
}
