//! SQL on the short link table. Everything here is synchronous: callers run it on a blocking
//! thread.

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};

use super::code;

/// Creates the table if needed, then writes and drops a scratch table so that a database or
/// directory we can't write to fails now instead of on the first request.
pub fn init(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS short_links (
             id           TEXT PRIMARY KEY,
             canonical    TEXT NOT NULL UNIQUE,
             created_at   INTEGER NOT NULL,
             last_used_at INTEGER NOT NULL
         ) STRICT;
         BEGIN IMMEDIATE;
         CREATE TABLE write_check (x INTEGER);
         DROP TABLE write_check;
         COMMIT;",
    )
}

/// The code for `canonical`: the existing one, or a new one derived from its hash. On a
/// collision with another link the code grows one character at a time. `None` if every length
/// up to [`code::MAX_LEN`] collides (practically impossible).
pub fn insert_or_get(
    conn: &mut Connection,
    canonical: &str,
    now: i64,
) -> rusqlite::Result<Option<String>> {
    let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let existing: Option<String> = tx
        .query_row(
            "SELECT id FROM short_links WHERE canonical = ?1",
            [canonical],
            |row| row.get(0),
        )
        .optional()?;
    if existing.is_some() {
        return Ok(existing);
    }

    let hash = code::hash(canonical);
    for len in code::MIN_LEN..=code::MAX_LEN {
        let id = code::encode(&hash, len);
        let taken = tx
            .query_row("SELECT 1 FROM short_links WHERE id = ?1", [&id], |_| Ok(()))
            .optional()?
            .is_some();
        if !taken {
            tx.execute(
                "INSERT INTO short_links (id, canonical, created_at, last_used_at)
                 VALUES (?1, ?2, ?3, ?3)",
                params![id, canonical, now],
            )?;
            tx.commit()?;
            return Ok(Some(id));
        }
    }
    Ok(None)
}

/// The canonical query of a (normalized) code.
pub fn lookup(conn: &Connection, id: &str) -> rusqlite::Result<Option<String>> {
    conn.query_row(
        "SELECT canonical FROM short_links WHERE id = ?1",
        [id],
        |row| row.get(0),
    )
    .optional()
}

/// Number of stored links.
pub fn count(conn: &Connection) -> rusqlite::Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM short_links", [], |row| row.get(0))
}

pub fn touch(conn: &Connection, id: &str, now: i64) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE short_links SET last_used_at = ?2 WHERE id = ?1",
        params![id, now],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        conn
    }

    #[test]
    fn same_canonical_same_code() {
        let mut conn = db();
        let a = insert_or_get(&mut conn, "uni=unicam&weeks=4", 1)
            .unwrap()
            .unwrap();
        let b = insert_or_get(&mut conn, "uni=unicam&weeks=4", 2)
            .unwrap()
            .unwrap();
        assert_eq!(a, b);
        assert_eq!(
            a,
            code::encode(&code::hash("uni=unicam&weeks=4"), code::MIN_LEN)
        );
        assert_eq!(
            lookup(&conn, &a).unwrap().as_deref(),
            Some("uni=unicam&weeks=4")
        );
        assert_eq!(lookup(&conn, "00000000").unwrap(), None);
    }

    #[test]
    fn collision_grows_the_code() {
        let mut conn = db();
        let canonical = "uni=unicam&weeks=4";
        let taken = code::encode(&code::hash(canonical), code::MIN_LEN);
        conn.execute(
            "INSERT INTO short_links VALUES (?1, 'someone else', 0, 0)",
            [&taken],
        )
        .unwrap();

        let id = insert_or_get(&mut conn, canonical, 1).unwrap().unwrap();
        assert_eq!(id.len(), code::MIN_LEN + 1);
        assert!(id.starts_with(&taken));
        // Asking again returns the longer code, not a new one.
        assert_eq!(insert_or_get(&mut conn, canonical, 2).unwrap(), Some(id));
    }

    #[test]
    fn touch_updates_last_used() {
        let mut conn = db();
        let id = insert_or_get(&mut conn, "a=1", 10).unwrap().unwrap();
        touch(&conn, &id, 20).unwrap();
        let (created, used): (i64, i64) = conn
            .query_row(
                "SELECT created_at, last_used_at FROM short_links WHERE id = ?1",
                [&id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!((created, used), (10, 20));
    }

    #[test]
    fn init_is_idempotent() {
        let conn = db();
        init(&conn).unwrap();
    }
}
