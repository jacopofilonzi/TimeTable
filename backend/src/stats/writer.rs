//! Background writer: requests are queued without blocking the handlers and written in batches
//! by a dedicated thread.

use std::sync::{
    Arc, Mutex,
    mpsc::{Receiver, Sender, SyncSender, TrySendError, sync_channel},
};

use rusqlite::Connection;

use super::{RequestEvent, store};
use crate::metrics::{self, DbLabels};

/// Requests waiting to be written; beyond this they are dropped (and counted).
const QUEUE: usize = 10_000;
/// Most requests written in one transaction.
const BATCH: usize = 500;

pub enum Message {
    Row(Box<RequestEvent>),
    /// Answered once every row queued before it is written.
    Flush(Sender<()>),
}

/// Starts the writer thread; it stops when every sender is dropped.
pub fn spawn(db: Arc<Mutex<Connection>>) -> std::io::Result<SyncSender<Message>> {
    let (tx, rx) = sync_channel(QUEUE);
    std::thread::Builder::new()
        .name("feed-stats-writer".into())
        .spawn(move || run(&db, &rx))?;
    Ok(tx)
}

/// Queues a message without blocking; a full queue drops rows.
pub fn send(tx: &SyncSender<Message>, message: Message) {
    match tx.try_send(message) {
        Ok(()) => {}
        Err(TrySendError::Full(_)) => {
            metrics::get().tracking_dropped.inc();
            tracing::warn!("usage log queue full, dropping a row");
        }
        Err(TrySendError::Disconnected(_)) => {
            tracing::warn!("usage log writer stopped, dropping a row");
        }
    }
}

fn run(db: &Mutex<Connection>, rx: &Receiver<Message>) {
    while let Ok(first) = rx.recv() {
        let mut rows = Vec::new();
        let mut flushes = Vec::new();
        let mut next = Some(first);
        while let Some(message) = next {
            match message {
                Message::Row(row) => rows.push(*row),
                Message::Flush(done) => flushes.push(done),
            }
            next = if rows.len() >= BATCH {
                None
            } else {
                rx.try_recv().ok()
            };
        }
        if !rows.is_empty() {
            let mut conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
            if let Err(err) = store::insert(&mut conn, &rows) {
                metrics::get()
                    .sqlite_errors
                    .get_or_create(&DbLabels { db: "feed_stats" })
                    .inc();
                tracing::error!(%err, rows = rows.len(), "could not write the usage log");
            }
        }
        for done in flushes {
            let _ = done.send(());
        }
    }
}
